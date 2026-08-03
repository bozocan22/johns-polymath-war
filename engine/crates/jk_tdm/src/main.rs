//! jk_tdm - arena shooter on the deterministic 120 Hz core.
//!
//! v4: every weapon is a real multi-part model (barrel, stock, magazine,
//! scope, bow limbs + string, spear blade) - held in the hands, not floating.
//! Bows visibly draw, spears cock overhead javelin-style. The roster is
//! ROBOT COWBOYS: metal heads with glowing visors under proper hats, team
//! bandanas, belts with shining buckles, boots - per-man colors.
//!
//! v5 (Phantom-Forces feel):
//! - FIRST or THIRD person - V toggles; first person gets real HANDS and
//!   a full weapon viewmodel that bobs, kicks, and dips on reload.
//! - DODGE ROLL (Q, or tap crouch while sprinting): a duck-spin somersault,
//!   faster than a sprint, low to the ground, gun locked while tumbling.
//!   Hard landings breakfall into the same roll automatically - parkour.
//! - Legs got KNEES and ANKLES: thigh/shin/foot chains with a real gait,
//!   a full deep crouch, an air tuck, and the roll's tucked ball.
//! - THREE MAPS at the intro: the dust arena, a castle bailey (keep, drum
//!   towers, crenellated walls), and green castle gardens (hedges, ruins,
//!   trees). Recoil is HALF what it was, per the owner's request.
//! - Bow/spear aiming shows a red PREDICTED ARC with a landing marker -
//!   the exact flight the sim will fly.
//!
//! Controls: WASD move - SPACE jump - Q dodge roll - V first/third person -
//! mouse look - LEFT CLICK aim (zoom) - RIGHT CLICK / T fire - CTRL/C
//! crouch - SHIFT sprint - R reload - TAB scoreboard - ESC menu

mod branding;
mod sim;

use bevy::audio::Volume;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::math::Isometry3d;
use bevy::render::camera::ClearColorConfig;
use bevy::render::view::RenderLayers;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use jk_core::timestep::DT;
use sim::*;
use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Resource)]
struct Game {
    sim: TdmSim,
    accum: f32,
    rebuild: bool,
    last_t: f32,
    /// Edge-triggered inputs latched until a sim step consumes them -
    /// above 120 fps some frames run zero steps, and a raw `just_pressed`
    /// would be lost on exactly those frames.
    pending_jump: bool,
    pending_reload: bool,
    pending_dodge: bool,
    pending_shield: bool,
    pending_slot: Option<u8>,
    /// §5: G pressed - cycle the selected throwable (edge, latched).
    pending_cycle_throw: bool,
}

#[derive(Resource)]
struct CamCtl {
    yaw: f32,
    pitch: f32,
    grabbed: bool,
    ads: bool,
    recoil: f32,
    /// V toggles: first-person (hands + viewmodel) vs third-person.
    first_person: bool,
    /// §3.4 ADS progress 0..1, advanced framerate-independently. Drives
    /// the FOV blend, the sim's ADS gate (spread/speed at > 0.9), and the
    /// zoom-consistent sensitivity.
    ads_t: f32,
    /// The LIVE world FOV in radians (mid-transition included) - §3.2
    /// feeds this into the tangent sensitivity match.
    fov_now: f32,
    /// §5 person blend: 0 = first person, 1 = third person. `first_person`
    /// is the target; this eases toward it over PERSON_BLEND_S.
    person_t: f32,
    /// Smoothed third-person boom length after collision (§5.2):
    /// pulled in instantly on contact, pushed back out slowly.
    boom: f32,
    /// §5.3: the muzzle→aim-point segment is obstructed close-by - the
    /// crosshair shows a blocked state so near-cover hits aren't a mystery.
    blocked: bool,
    /// §10 (Brief II): smoothed shoulder side, +1 right / −1 left - the
    /// boom auto-mirrors when a wall sits within 1.2 m on the camera side.
    shoulder: f32,
    /// Landing camera dip (weight-absorb), decaying - set on the
    /// grounded edge proportional to impact speed.
    land_dip: f32,
    /// Task 3 rule 5 (MISSION doc): landings never fully damp in one
    /// frame - a small upward rebound (8% of impact) lifts the camera
    /// briefly before it settles, instead of a pure one-way decay.
    /// This is the rebound AMPLITUDE; `landing_offset` shapes it against
    /// `land_t` rather than decaying it per-frame.
    land_rebound: f32,
    /// Seconds since the last touchdown - the single clock both the
    /// landing dip and its delayed rebound are sampled from.
    land_t: f32,
    prev_vy: f32,
    prev_grounded: bool,
    /// §5.2 (Brief VI): scoped-class zoom stage - 0 unscoped, 1 = 40°,
    /// 2 = 10°. RMB cycles; every shot auto-unscopes.
    zoom_stage: u8,
    prev_fire_cd: f32,
    /// §5.1 (Brief VII v2): the hip boom's OWN eased length - chases
    /// TP_BOOM/TP_BOOM_SPRINT with a 0.12s lag, independent of the ADS
    /// pull-in which blends further on top of whatever this settles to.
    sprint_boom: f32,
    /// §2.5/§5.2: velocity state for the boom-collision RECOVERY spring
    /// (`SPRING_K_CAMERA_BOOM`) - zeroed on every instant pull-in so a
    /// recovery in progress never fights the next snap.
    boom_vel: f32,
    /// Whether the boom is currently shortened by cover. Explicit state
    /// because "boom < free-space target" is equally true while
    /// recovering from a wall and while an eased target is growing - the
    /// spring must govern only the former.
    boom_occluded: bool,
}

impl Default for CamCtl {
    fn default() -> Self {
        CamCtl {
            yaw: 0.0,
            pitch: 0.08,
            grabbed: false,
            ads: false,
            recoil: 0.0,
            first_person: false,
            ads_t: 0.0,
            fov_now: FOV_HIP_DEG.to_radians(),
            person_t: 1.0,
            boom: TP_BOOM,
            blocked: false,
            shoulder: 1.0,
            land_dip: 0.0,
            land_rebound: 0.0,
            land_t: 99.0, // no landing yet
            boom_occluded: false,
            prev_vy: 0.0,
            prev_grounded: true,
            zoom_stage: 0,
            prev_fire_cd: 0.0,
            sprint_boom: TP_BOOM,
            boom_vel: 0.0,
        }
    }
}

// ---- §3/§5 camera + aim tuning -------------------------------------------
/// Hip-fire world FOV (vertical, degrees). Gun `zoom_deg` values lerp
/// against this on ADS.
const FOV_HIP_DEG: f32 = 62.0;
/// One mouse sensitivity for both axes - raw, no dt, no smoothing (§3.1).
const MOUSE_SENS: f32 = 0.0026;
/// Zoom sensitivity ratio: 1.0 = full monitor-distance match (§3.2).
const ADS_SENS_RATIO: f32 = 1.0;
/// ADS transition time, ease-out (§3.4).
const ADS_TIME_S: f32 = 0.12;
/// First↔third person blend time, ease-out (§5.1).
const PERSON_BLEND_S: f32 = 0.18;
/// Third-person boom: back / up / screen-right of the head pivot (§5.1).
// §5.1 (Brief VII v2): hip 2.2m back / +0.45m right / +0.12m up.
const TP_BOOM: f32 = 2.2;
const TP_UP: f32 = 0.12;
const TP_RIGHT: f32 = 0.45;
/// §5.1: sprint eases the boom out to 2.5m with a 0.12s lag.
const TP_BOOM_SPRINT: f32 = 2.5;
const TP_SPRINT_LAG_S: f32 = 0.12;
/// §5.1: aim (RMB in third person) - 1.35m boom, +0.55m right, FOV -12deg.
const TP_BOOM_AIM: f32 = 1.35;
const TP_RIGHT_AIM: f32 = 0.55;
/// §5.2: upper-body additive aim before the legs turn-in-place catch up.
const TORSO_AIM_LIMIT_DEG: f32 = 60.0;

/// R4 (Brief VII v2 / MISSION doc): config externalization, first real
/// slice of it. Every field here mirrors one of the TP_* consts above -
/// those consts remain the shipped DEFAULT (and `CamCtl::default()`'s
/// one-frame fallback, corrected by `camera_system` on the very next
/// tick), while `camera_system` itself now reads these off a `Resource`
/// loaded once at startup from `config/camera_tuning.txt` if present.
/// A missing file, missing key, or unparseable value all fall back to
/// the exact same compiled-in number - zero behavior change out of the
/// box, real behavior change if a human edits the file. Deliberately
/// scoped to the camera-feel constants only (not `TORSO_AIM_LIMIT_DEG`,
/// which lives in a separate pure function with its own test, and not
/// `sim.rs`'s `MECH_SCALE`, which several OTHER consts derive from at
/// compile time - converting that one is a bigger, riskier job left for
/// its own pass).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
struct CameraTuning {
    tp_boom: f32,
    tp_up: f32,
    tp_right: f32,
    tp_boom_sprint: f32,
    tp_sprint_lag_s: f32,
    tp_boom_aim: f32,
    tp_right_aim: f32,
}

impl Default for CameraTuning {
    fn default() -> Self {
        CameraTuning {
            tp_boom: TP_BOOM,
            tp_up: TP_UP,
            tp_right: TP_RIGHT,
            tp_boom_sprint: TP_BOOM_SPRINT,
            tp_sprint_lag_s: TP_SPRINT_LAG_S,
            tp_boom_aim: TP_BOOM_AIM,
            tp_right_aim: TP_RIGHT_AIM,
        }
    }
}

fn camera_tuning_path() -> std::path::PathBuf {
    std::path::PathBuf::from("config/camera_tuning.txt")
}

// ---- settings persistence -------------------------------------------------
// The settings screen's five values were session-only: change your
// sensitivity, quit, and it was gone - the audit table called this out
// as an honest gap. Same hand-rolled `key = value` convention as
// camera_tuning and the Forge (no serde for five values), same rule:
// a missing/malformed file or key can never stop the game starting.

fn settings_path() -> std::path::PathBuf {
    std::path::PathBuf::from("config/settings.txt")
}

fn settings_to_text(s: &GameSettings) -> String {
    format!(
        "# jk_tdm player settings - rewritten on every change\n\
         swap_mouse = {}\nminimap = {}\nsens_idx = {}\nfov_idx = {}\ninvert_y = {}\n",
        s.swap_mouse as u8, s.minimap as u8, s.sens_idx, s.fov_idx, s.invert_y as u8
    )
}

/// Pure parse, directly testable. Indices from disk are CLAMPED to their
/// choice lists - a hand-edited or stale file must not index out of
/// bounds (the persistence sibling of the Forge's own bounds rule).
fn parse_settings(text: &str) -> GameSettings {
    let mut s = GameSettings::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let Ok(v) = val.trim().parse::<i64>() else {
            continue;
        };
        match key.trim() {
            "swap_mouse" => s.swap_mouse = v != 0,
            "minimap" => s.minimap = v != 0,
            "sens_idx" => s.sens_idx = (v.max(0) as usize).min(SENS_CHOICES.len() - 1),
            "fov_idx" => s.fov_idx = (v.max(0) as usize).min(FOV_CHOICES.len() - 1),
            "invert_y" => s.invert_y = v != 0,
            _ => {}
        }
    }
    s
}

fn load_settings() -> GameSettings {
    match std::fs::read_to_string(settings_path()) {
        Ok(text) => parse_settings(&text),
        Err(_) => GameSettings::default(),
    }
}

/// Save on change (the resource is only mutated by the settings screen
/// and the M minimap hotkey, so `is_changed` fires rarely). Write
/// failure is non-fatal: settings still work for the session.
fn persist_settings(settings: Res<GameSettings>) {
    if settings.is_changed() && !settings.is_added() {
        let _ = std::fs::create_dir_all("config");
        let _ = std::fs::write(settings_path(), settings_to_text(&settings));
    }
}

fn load_camera_tuning() -> CameraTuning {
    match std::fs::read_to_string(camera_tuning_path()) {
        Ok(text) => parse_camera_tuning(&text),
        Err(_) => CameraTuning::default(),
    }
}

/// `key = value` per line, `#` comments and blank lines ignored, same
/// hand-rolled-text convention as the Forge saves (no serde dependency
/// for seven numbers). Any key that's absent, misspelled, or fails to
/// parse as f32 just keeps its compiled-in default - this can never be
/// the reason the game fails to start. Pure/no I/O, so it's directly
/// testable without touching the filesystem.
fn parse_camera_tuning(text: &str) -> CameraTuning {
    let mut t = CameraTuning::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let Ok(v) = val.trim().parse::<f32>() else {
            continue;
        };
        match key.trim() {
            "tp_boom" => t.tp_boom = v,
            "tp_up" => t.tp_up = v,
            "tp_right" => t.tp_right = v,
            "tp_boom_sprint" => t.tp_boom_sprint = v,
            "tp_sprint_lag_s" => t.tp_sprint_lag_s = v,
            "tp_boom_aim" => t.tp_boom_aim = v,
            "tp_right_aim" => t.tp_right_aim = v,
            _ => {}
        }
    }
    t
}

/// §5.2 (Brief VII v2): the torso's additive aim yaw RELATIVE to the
/// legs' facing - clamped to +/-60deg. Beyond the clamp, the excess
/// is what should drive a turn-in-place (the legs catching up to face
/// where the torso already committed); this function only owns the
/// clamp itself, which is what the completion-gate test measures.
fn torso_aim_offset(desired_delta_deg: f32) -> f32 {
    desired_delta_deg.clamp(-TORSO_AIM_LIMIT_DEG, TORSO_AIM_LIMIT_DEG)
}

/// §2 (rig audit, MISSION doc): hip-shoulder SEPARATION, in radians.
/// The render root (legs attach here) carries exactly `f.yaw`; torso is
/// the root's CHILD with this as its own additional local Y-rotation -
/// so this return value IS "thorax yaw minus pelvis yaw" by construction
/// (composition of a child's local rotation onto a parent's), not an
/// approximation of it. Extracted verbatim from the pre-existing inline
/// `spear_yaw` block in `sync_fighters` so the audit's separation test
/// measures the exact render path, never a copy of it.
// ---- Task 3 (MISSION doc): the elastic load model ------------------------
// COSMETIC layer (R3) - the velocity/timing SHAPE feeds animation, never a
// sim hit/damage value. "Explosive motion is never muscle firing from
// rest - it is load, then release, and the release is faster than the
// load that produced it."

/// A stretch-shortening cycle: pre-activation -> loaded stretch
/// (eccentric) -> explosive shortening (concentric), scaled by how much
/// was stored.
///
/// SPEC FIXTURE, NOT A LIVE PATH. This doc used to claim "every power
/// move routes through" it; in fact `ElasticMove` and `chain_peak_tick`
/// have zero production call sites - they are exercised only by the
/// Task 3 tests. What IS wired from this module: `chain_segment_scale`
/// drives the spear follow-through, `landing_rebound_vy` drives the
/// camera's landing rebound, and `counter_movement_bonus` graduated to
/// sim.rs where it now scales the dodge burst via a trigger-time
/// velocity snapshot (`roll_boost`) - the sim-side field this doc once
/// said was the blocker.
#[derive(Clone, Copy, Debug)]
struct ElasticMove {
    load_s: f32,
    release_s: f32,
    stored_energy: f32,     // 0..1, accumulates during the load phase
    return_efficiency: f32, // 0.92 human tendon, 0.55 mech steel
}

impl ElasticMove {
    /// Rule 1: release must be 2-3x faster than load - the completion
    /// gate's exact test threshold (release_s <= load_s / 2).
    fn load_release_ok(&self) -> bool {
        self.release_s <= self.load_s / 2.0
    }
    /// Rule 2: stored energy scales output - a fully loaded move is
    /// measurably (not just numerically) stronger.
    fn release_velocity(&self, base: f32) -> f32 {
        base * (1.0 + self.stored_energy.clamp(0.0, 1.0) * 0.35)
    }
}

// Rule 3 (`counter_movement_bonus`) GRADUATED from spec fixture to real
// sim mechanic: it now lives in sim.rs, snapshotted at the dodge trigger
// and scaling the roll burst (`roll_boost`). Reachable here via
// `use sim::*` - the copy that sat here unwired for two briefs is gone.

/// Rule 5: landings never fully damp in one frame - ~8% of impact
/// velocity returns upward, eased over the caller's own 2-3 frame window.
/// `impact_vy` is negative (downward); the return is always positive.
fn landing_rebound_vy(impact_vy: f32) -> f32 {
    (-impact_vy).max(0.0) * 0.08
}

/// Task 3.3: the kinetic chain - one shared proximal-to-distal sequence
/// every power move routes through. `elapsed_s` since the move's own
/// onset; returns this segment's current velocity-scale multiplier (0
/// before its own onset, ramping toward `peak_scale` after).
/// §3 (BRIEF_VIII_B): the kinetic chain, DERIVED FROM MEASUREMENT -
/// this table was authored by feel until 2026-08-02.
///
/// Anchors 0/3/4/7 are Campos, Brizuela & Ramon (2004), New Studies in
/// Athletics 19:47-57, Table 3: n=7 elite male javelin finalists, 1999
/// World Championships (Seville), two cameras at 50 fps. Peak linear
/// velocity relative to release - hip -0.130 s, shoulder -0.090 s,
/// elbow -0.060 s, release 0.000 - re-based so the pelvis peak is the
/// chain's zero. A joint marker's linear velocity peaks when the
/// segment PROXIMAL to it is at peak angular velocity (the marker is
/// that segment's distal end), which is why the shoulder marker maps
/// to the clavicle and the elbow marker to the upper arm.
///
/// Indices 1,2 are inertia-weighted across the de Leva trunk
/// subsegments; 5,6 are a geometric compression seeded by the measured
/// 40->30 ms gap, ratio q = 0.81053571 (the root of q + q^2 + q^3 = 2),
/// which lands the arm's three hops on exactly 60 ms and the tip on the
/// measured 130 ms anchor. The spec printed q = 0.8107 for one commit,
/// which sums to 60.023 ms; the shipped values here are unaffected,
/// because both roots round to the same milliseconds. Full arithmetic in
/// research/body-rig/SPEC_20_SEGMENT_RIG.md §3.3, and re-solved from
/// scratch by `the_arm_onsets_reproduce_an_independently_solved_geometric_root`.
///
/// Do NOT read index 2's agreement with the old by-feel table as
/// corroboration of the trunk derivation - it is forced by the 5 ms floor
/// (0.040 - 0.005), not by the inertia split, so it corroborates nothing.
///
/// ONLY THE DIFFERENCES ARE LOAD-BEARING: `chain_peak_tick` adds a
/// SHARED `ramp_s` to every entry, so the absolute zero is arbitrary
/// and the shared ramp absorbs the onset-to-peak lag.
///
/// PRECISION CEILING: the source is 50 fps = 20 ms per frame. Do NOT
/// build per-fighter timing variance or anything finer than ~20 ms on
/// top of this table - the 5 ms floor used in the derivation is a
/// monotonicity device, not a claim of 5 ms accuracy.
const CHAIN_ONSET_OFFSETS: [f32; 8] =
    [0.000, 0.016, 0.035, 0.040, 0.070, 0.094, 0.114, 0.130];

/// The four MEASURED anchors, held separately so the test compares two
/// independent tables rather than a constant against itself.
const JAVELIN_ANCHOR_S: [(usize, f32); 4] =
    [(0, 0.000), (3, 0.040), (4, 0.070), (7, 0.130)];

/// Peak angular-velocity multiplier per segment. Indices 0..=2 are
/// MEASURED: the thorax/pelvis peak angular-velocity ratio of 1.43 is
/// the mean of four marker-based pitching datasets; lumbar = sqrt(1.43)
/// as the single intermediate hop. Indices 3..=7 are NOT derivable from
/// any source consulted - their per-hop gains are carried over from the
/// by-feel table and rescaled onto the new thorax value.
///
/// This entire table is currently INERT in production, verified by
/// algebra rather than assumed: its only consumer divides
/// `chain_segment_scale`'s output by `CHAIN_PEAK_SCALE[TIP]`, and that
/// function multiplies by exactly the same value.
const CHAIN_PEAK_SCALE: [f32; 8] =
    [1.000, 1.196, 1.430, 1.554, 1.679, 1.990, 2.300, 2.611];

/// The chain activation curve for ONE segment, written against that
/// segment's two table ROWS instead of its index. Production always goes
/// through `chain_segment_scale`; lifting the two lookups into parameters
/// is what lets a test substitute rows the consts do not contain and so
/// assert table-invariance by *calling the real code* rather than by
/// retyping it. (Spec §Step 1 asked for this refactor; it was skipped,
/// and the test that should have caught the resulting hole was itself the
/// retyped copy - see `spear_followthrough_is_invariant_to_the_chain_tables`.)
fn chain_scale_from(onset: f32, peak: f32, elapsed_s: f32, ramp_s: f32) -> f32 {
    if elapsed_s < onset {
        0.0
    } else {
        peak * ((elapsed_s - onset) / ramp_s.max(1e-4)).clamp(0.0, 1.0)
    }
}

fn chain_segment_scale(segment_index: usize, elapsed_s: f32, ramp_s: f32) -> f32 {
    chain_scale_from(
        CHAIN_ONSET_OFFSETS[segment_index],
        CHAIN_PEAK_SCALE[segment_index],
        elapsed_s,
        ramp_s,
    )
}

/// Task 3.3 test support: the tick at which segment `i` reaches its peak,
/// for asserting strict pelvis->lumbar->...->tip ordering.
fn chain_peak_tick(segment_index: usize, ramp_s: f32) -> f32 {
    CHAIN_ONSET_OFFSETS[segment_index] + ramp_s
}

/// Task 3.3, sprint-start consumer: the HEAD is the chain's last segment,
/// so it arrives at a new acceleration lean ~one TIP ONSET behind the
/// pelvis. The time constant IS `CHAIN_ONSET_OFFSETS[7]` - read the value
/// there, it is deliberately not restated here. (It was, as "0.125 s",
/// and went stale five lines below the table that moved the tip to 0.130;
/// a restated constant is a constant that will lie to the next reader.)
///
/// This chases the pelvis lean with that time constant; the difference
/// between the two is the head's transient counter-pitch - present only
/// while the lean is CHANGING, zero at steady state, so a held sprint
/// looks exactly as before. `head_lag_chase_pins_the_measured_tip_onset`
/// pins the resulting curve to hand-computed numbers, so a future edit to
/// index 7 has to be deliberate.
// ---- §B.2 (mech plan): idle life -----------------------------------
// The only idle motion a mech had was `mech_bob`, which is walk-only and
// returns to EXACTLY zero at a dead stop. A multi-ton machine going
// perfectly inert the instant it stops reads as a prop, not a machine.
//
// Both terms below are COSMETIC and pure functions of sim time, so they
// are directly unit-testable without standing up Bevy - the same
// extraction pattern this file already uses for `view_recoil_offset`.

/// Servo micro-tremor, as (pitch, roll) radians.
///
/// Two deliberately unrelated frequencies with an offset phase, so the
/// pair never resolves into a single readable oscillation. Both are
/// well away from `mech_bob`'s 0.9 Hz stride: sharing that cadence would
/// read as the walk cycle at low amplitude, which is exactly what this
/// is not.
fn mech_servo_tremor(t: f32) -> (f32, f32) {
    (
        (t * 3.1).sin() * 0.0025,
        (t * 2.3 + 1.7).sin() * 0.0020,
    )
}

/// Hull "breathing" - a slow plate-expansion cue at ~0.18 Hz, the
/// mechanical analogue of the human idle breath already in the rig.
/// Positive lifts the camera, so the caller subtracts it.
fn mech_hull_breath(t: f32) -> f32 {
    (t * std::f32::consts::TAU * 0.18).sin() * 0.008
}

fn chain_lag_chase(lag: f32, target: f32, dt: f32) -> f32 {
    lag + (target - lag) * (dt / CHAIN_ONSET_OFFSETS[7]).min(1.0)
}

// §3.2 the coil: named so the follow-through can start from the EXACT
// value the windup/thrust branch ends on. Both branches deliberately
// land on the same release yaw, so a throw and a thrust hand off to the
// same follow-through with no discontinuity.
const COIL_AWAY_RAD: f32 = -0.73; // windup: torso coils away from the target
const COIL_SWING_RAD: f32 = 1.08; // plant -> whip: hips fire open through it
const COIL_PLANT_FRAC: f32 = 0.68; // windup fraction where the plant blocks
const THRUST_AWAY_RAD: f32 = -0.45;
const THRUST_SWING_RAD: f32 = 0.80;
/// The yaw BOTH the throw windup and the thrust end on (COIL_AWAY +
/// COIL_SWING == THRUST_AWAY + THRUST_SWING == 0.35). The follow-through
/// must begin here or release visibly pops.
const SPEAR_RELEASE_YAW: f32 = COIL_AWAY_RAD + COIL_SWING_RAD;

/// Task 3.3 real consumer: the spear's post-action follow-through, for
/// BOTH a throw release and a thrust's recovery (torso_coil_yaw routes
/// either one here once `spear_wind_t`/`knife_phase` both hit zero).
///
/// The torso does not stop dead at the release angle: it CARRIES PAST it
/// (the tip segment is the chain's last and most amplified one,
/// `CHAIN_PEAK_SCALE[7]`), then relaxes back to neutral.
///
/// `release_t` is seconds since the action ended. NEGATIVE means no
/// release has happened yet (fresh spawn, or the spear was never
/// swung) - that returns 0, so a fighter who merely holds a spear is
/// not born mid-unwind.
///
/// The tip is sampled from its OWN onset rather than from zero: that
/// offset is the tip's delay behind the PELVIS when a chain starts, but
/// here the chain already ran during the windup and the release IS the
/// tip's moment. Sampling from zero made this silent for the whole tip
/// onset - a hard snap to neutral exactly when the motion should be at
/// its most alive.
///
/// ALGEBRAIC NOTE (verified 2026-08-02, corrected 2026-08-03 after Thor
/// measured it): passing `release_t + onset` means the chain curve
/// evaluates `elapsed - onset == release_t`, so the onset CANCELS; and
/// dividing by the tip's peak cancels the peak the same expression just
/// multiplied in. The drive term therefore reduces to
/// `(release_t / RAMP_S).clamp(0,1)`, and this curve is INVARIANT to both
/// chain tables.
///
/// "Exactly" ONLY in exact arithmetic. In f32 it is not exact: both
/// `(t + onset) - onset` and `peak * x / peak` round. Measured on a 10 us
/// grid: **8,290** samples carry a non-zero residual in the drive term,
/// worst case **1.788e-7** at `release_t` = 0.0936. Every one of them
/// falls while the ramp is still climbing (`release_t` < `RAMP_S`) -
/// 8,290 of the 12,000 samples there, 69% - and there are **exactly
/// zero** once the clamp saturates, because `peak * 1.0 / peak` is
/// exact. (Quote that count, not a percentage of a sweep: the same 8,290
/// reads as 20.7% of a 0..0.4 s sweep and 13.8% of a 0..0.6 s one.)
///
/// UNITS - the trap this block was written to correct, and then fell into
/// itself (F2, Thor, 2026-08-03). **1.788e-7 is a residual in the DRIVE
/// term**, which is dimensionless and runs 0..1. The invariance test's
/// **1e-6 tolerance is applied to YAW**, in radians. Dividing one by the
/// other produced the "~5.6x" this block used to quote, and that is not a
/// margin of anything - it is two different quantities put over each
/// other. Like for like:
///
/// - the drive residual expressed as yaw is `1.788e-7 * OVERSHOOT_RAD` =
///   **1.79e-8 rad**, which alone would be 56x inside the tolerance;
/// - but what the test actually measures is the END-TO-END divergence
///   between table variants, and that is **2.98e-8 rad** (worst case,
///   on the 1 ms grid the test itself sweeps, over 0..0.6 s) - larger
///   than the drive residual alone, because the divide, the add and the
///   decay multiply each round on top of it.
///
/// So the margin is **34x** (1e-6 / 2.98e-8), and that is the only figure
/// this file should quote for it. It is grid-dependent: off the test's
/// 1 ms grid it gets worse, reaching **5.96e-8 rad (17x)** on a 1 us
/// grid. Do NOT tighten the tolerance toward 1e-7 - at 1e-7 that
/// off-grid worst case leaves 1.7x, and the test becomes a coin flip the
/// moment anyone refines the sweep. And do NOT assert bit-equality across
/// substituted tables: the spec's Step 1 test table asked for `==` and
/// that assertion is simply false here.
/// The invariance is real; its precision is finite. Written down
/// because the sampling-from-onset reasoning above is still the right
/// INTENT - it is what keeps the code correct if the scale ever stops
/// cancelling - but a reader must not infer that retuning either table
/// changes this curve. It does not.
fn spear_followthrough_yaw(release_t: f32) -> f32 {
    const TIP: usize = 7;
    spear_followthrough_yaw_from(release_t, CHAIN_ONSET_OFFSETS[TIP], CHAIN_PEAK_SCALE[TIP])
}

/// `spear_followthrough_yaw` with the tip's two table rows lifted into
/// parameters. Production only ever calls the wrapper above; this exists
/// so the invariance claim can be tested by feeding the REAL function
/// rows the consts do not contain, instead of by retyping its body into
/// a test (which is how the missing `+ onset` shipped here once already -
/// see handback/AUDIT.md, "bugs I introduced this session" #1).
/// `tip_peak` must be non-zero.
fn spear_followthrough_yaw_from(release_t: f32, tip_onset: f32, tip_peak: f32) -> f32 {
    const RAMP_S: f32 = 0.12;
    const OVERSHOOT_RAD: f32 = 0.10; // carried PAST the release, not back through it
    const HOLD_S: f32 = 0.05; // the carry-past runs before the settle starts
    const SETTLE_RATE: f32 = 6.0;
    if release_t < 0.0 {
        return 0.0; // nothing has been thrown or thrust yet
    }
    let drive = chain_scale_from(tip_onset, tip_peak, release_t + tip_onset, RAMP_S) / tip_peak;
    let decay = (-SETTLE_RATE * (release_t - HOLD_S).max(0.0)).exp();
    (SPEAR_RELEASE_YAW + OVERSHOOT_RAD * drive) * decay
}

fn torso_coil_yaw(gun: GunKind, spear_wind_t: f32, knife_phase: f32, in_mech: bool, release_t: f32) -> f32 {
    if gun == GunKind::Spear {
        if spear_wind_t > 0.0 {
            let wp = 1.0 - spear_wind_t / SPEAR_WINDUP_S;
            if wp < COIL_PLANT_FRAC {
                COIL_AWAY_RAD * (wp / COIL_PLANT_FRAC) // windup: torso coils away
            } else {
                // plant -> whip: hips fire open, fast
                COIL_AWAY_RAD
                    + COIL_SWING_RAD * ((wp - COIL_PLANT_FRAC) / (1.0 - COIL_PLANT_FRAC))
            }
        } else if knife_phase > 0.0 {
            let tw = THRUST_WIND_S * if in_mech { MECH_THRUST_TIME_MULT } else { 1.0 };
            let ph = knife_phase;
            if ph < tw {
                THRUST_AWAY_RAD * ease_out((ph / tw).clamp(0.0, 1.0))
            } else {
                THRUST_AWAY_RAD
                    + THRUST_SWING_RAD * ease_out(((ph - tw) / 0.16).clamp(0.0, 1.0))
            }
        } else {
            spear_followthrough_yaw(release_t)
        }
    } else {
        0.0
    }
}
/// Camera collision pad (§5.2) - the push-back-out itself is now the
/// SPRING_K_CAMERA_BOOM critical spring, not a fixed time constant.
const CAM_PAD: f32 = 0.2;

/// §3.2 monitor-distance sensitivity match: how much to scale raw mouse
/// input at the current (live, mid-transition) FOV so tracking feels
/// identical hip-fired and zoomed. `ratio` 1.0 = full match, 0.0 = off.
fn ads_sens_mult(fov_hip: f32, fov_now: f32, ratio: f32) -> f32 {
    let raw = (fov_now * 0.5).tan() / (fov_hip * 0.5).tan();
    1.0 + (raw - 1.0) * ratio
}

/// §3.3/§5.3 two-stage aim, shared by the shot command and the arc
/// preview so they always agree: the CROSSHAIR ray (camera eye, camera
/// forward) finds the first cover/ground hit (or the 200 m far point);
/// the SHOT direction runs from the muzzle toward that point. Returns
/// the muzzle-space aim direction plus whether the muzzle→point segment
/// is obstructed within ~1.5 m (the "you're hugging your own cover"
/// crosshair state).
fn crosshair_aim_dir(sim: &TdmSim, cam_tf: &Transform) -> (Vec3, bool) {
    let fwd = cam_tf.forward().as_vec3();
    let o = cam_tf.translation;
    let mut t_hit = 200.0_f32;
    for c in &sim.cover {
        if let Some((t, _)) = c.ray_hit(o.to_array(), fwd.to_array(), t_hit) {
            // ignore sub-0.6 m hits: in third person the ray can start
            // shy of geometry the CAMERA is already tucked against
            if t > 0.6 && t < t_hit {
                t_hit = t;
            }
        }
    }
    if fwd.y < -1e-4 {
        let t = -o.y / fwd.y; // the ground plane is a real aim target
        if t > 0.6 && t < t_hit {
            t_hit = t;
        }
    }
    let aim_point = o + fwd * t_hit;
    let eye = Vec3::from_array(sim.muzzle_origin(sim.player));
    let to = aim_point - eye;
    let dist = to.length().max(1e-4);
    let dir = to / dist;
    let probe = (dist - 0.1).min(1.5);
    let blocked = probe > 0.05
        && sim
            .cover
            .iter()
            .any(|c| c.ray_hit(eye.to_array(), dir.to_array(), probe).is_some());
    (dir, blocked)
}

/// Ease-out cubic - snappy start, gentle landing (§3.4/§5.1).
fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Everything the intro/loadout screen configures for the next match.
#[derive(Resource, Clone)]
struct Selected {
    map: MapKind,
    difficulty: Difficulty,
    per_team: usize,
    loadout: Loadout,
    /// cosmetic only: hat + tunic colors picked before the match
    hat: usize,
    tunic: usize,
    /// §6/§8 (Brief IV): melee slot choice + grenade budget preset
    melee_axe: bool,
    grenade_preset: usize,
}

// ---- §7 (Brief VII v2): the Forge - saved appearance slots ---------------
// Scoped honestly: this engine has no glTF/texture pipeline, so "parts" are
// the cosmetic choices that already exist (hat/tunic color, melee, grenade
// preset) - a real, working save/load system over real content, not a
// placeholder over content that doesn't exist yet.
const FORGE_SLOTS: usize = 3;

/// Everything the Forge saves/restores - a plain snapshot of `Selected`'s
/// cosmetic+loadout fields. Pure serialize/deserialize, no Bevy needed.
#[derive(Clone, Copy, PartialEq, Debug)]
struct ForgeProfile {
    hat: usize,
    tunic: usize,
    melee_axe: bool,
    grenade_preset: usize,
}

impl ForgeProfile {
    fn from_selected(sel: &Selected) -> Self {
        ForgeProfile {
            hat: sel.hat,
            tunic: sel.tunic,
            melee_axe: sel.melee_axe,
            grenade_preset: sel.grenade_preset,
        }
    }
    fn apply_to(&self, sel: &mut Selected) {
        sel.hat = self.hat;
        sel.tunic = self.tunic;
        sel.melee_axe = self.melee_axe;
        sel.grenade_preset = self.grenade_preset;
    }
    /// A compact one-line format - no serde dependency needed for four
    /// small fields. `hat,tunic,melee_axe,grenade_preset`.
    fn to_line(&self) -> String {
        format!("{},{},{},{}", self.hat, self.tunic, self.melee_axe as u8, self.grenade_preset)
    }
    fn from_line(s: &str) -> Option<Self> {
        let mut it = s.trim().split(',');
        Some(ForgeProfile {
            hat: it.next()?.parse().ok()?,
            tunic: it.next()?.parse().ok()?,
            melee_axe: it.next()?.parse::<u8>().ok()? != 0,
            grenade_preset: it.next()?.parse().ok()?,
        })
    }
}

fn forge_slot_path(slot: usize) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("forge_slot_{slot}.txt"))
}

fn forge_save(slot: usize, profile: &ForgeProfile) -> std::io::Result<()> {
    std::fs::write(forge_slot_path(slot), profile.to_line())
}

fn forge_load(slot: usize) -> Option<ForgeProfile> {
    let s = std::fs::read_to_string(forge_slot_path(slot)).ok()?;
    ForgeProfile::from_line(&s)
}

impl Default for Selected {
    fn default() -> Self {
        Selected {
            map: MapKind::Arena,
            difficulty: Difficulty::Normal,
            per_team: 5,
            loadout: DEFAULT_LOADOUT,
            hat: 0,
            tunic: 0,
            melee_axe: false,
            grenade_preset: 0,
        }
    }
}

/// Player-facing options (the settings page).
#[derive(Resource)]
struct GameSettings {
    /// §8: swap the aim/fire mouse buttons back to convention.
    swap_mouse: bool,
    minimap: bool,
    /// Index into `SENS_CHOICES` - a multiplier on `MOUSE_SENS`. Mouse
    /// sensitivity was a hard-coded constant with no way to change it,
    /// which makes the game unplayable for anyone whose mouse DPI does
    /// not happen to match the one it was tuned on.
    sens_idx: usize,
    /// Index into `FOV_CHOICES` - the hip-fire vertical FOV in degrees,
    /// replacing the fixed `FOV_HIP_DEG`.
    fov_idx: usize,
    /// Invert the vertical look axis.
    invert_y: bool,
}

/// Sensitivity multipliers applied to `MOUSE_SENS`.
const SENS_CHOICES: [(&str, f32); 6] = [
    ("0.50x  (very low)", 0.50),
    ("0.75x  (low)", 0.75),
    ("1.00x  (default)", 1.00),
    ("1.50x  (high)", 1.50),
    ("2.00x  (very high)", 2.00),
    ("3.00x  (twitch)", 3.00),
];
/// Hip-fire vertical FOV, degrees. The old fixed value was 62, which is
/// narrow enough to read as claustrophobic on a wide monitor.
const FOV_CHOICES: [(&str, f32); 6] = [
    ("62  (original)", 62.0),
    ("70", 70.0),
    ("80", 80.0),
    ("90  (recommended)", 90.0),
    ("100", 100.0),
    ("110  (max)", 110.0),
];
const SENS_DEFAULT_IDX: usize = 2;
const FOV_DEFAULT_IDX: usize = 3;

/// THE mouse mapping: `(aim button, fire button)`. One function, so the
/// settings label, the manual and the actual input handler cannot
/// disagree - all three used to derive it independently and BOTH text
/// versions had it exactly backwards, on the very control that changes
/// it. Default (`swap == false`) is the conventional LEFT = fire.
fn mouse_map(swap: bool) -> (MouseButton, MouseButton) {
    if swap {
        (MouseButton::Left, MouseButton::Right)
    } else {
        (MouseButton::Right, MouseButton::Left)
    }
}

/// The same mapping as human-readable button names, in the same order.
fn mouse_map_names(swap: bool) -> (&'static str, &'static str) {
    if swap {
        ("LEFT CLICK", "RIGHT CLICK")
    } else {
        ("RIGHT CLICK", "LEFT CLICK")
    }
}

impl GameSettings {
    fn sens_mult(&self) -> f32 {
        SENS_CHOICES[self.sens_idx.min(SENS_CHOICES.len() - 1)].1
    }
    fn fov_deg(&self) -> f32 {
        FOV_CHOICES[self.fov_idx.min(FOV_CHOICES.len() - 1)].1
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            swap_mouse: false,
            minimap: true,
            sens_idx: SENS_DEFAULT_IDX,
            fov_idx: FOV_DEFAULT_IDX,
            invert_y: false,
        }
    }
}

const HAT_CHOICES: [(&str, (f32, f32, f32)); 4] = [
    ("WHITE", (0.92, 0.90, 0.85)),
    ("BLACK", (0.12, 0.11, 0.11)),
    ("BROWN", (0.38, 0.24, 0.12)),
    ("RED", (0.55, 0.15, 0.10)),
];
const TUNIC_CHOICES: [(&str, (f32, f32, f32)); 4] = [
    ("GOLD", (0.95, 0.78, 0.25)),
    ("TEAL", (0.20, 0.75, 0.70)),
    ("VIOLET", (0.60, 0.35, 0.85)),
    ("STEEL", (0.55, 0.58, 0.62)),
];

// ---- world-space visuals -------------------------------------------------

#[derive(Component)]
struct FighterVis {
    index: usize,
}

#[derive(Component)]
struct FighterRig {
    /// §1.4 gait phase in radians - driven by DISTANCE, never by time.
    phase: f32,
    /// last frame's planar speed, for the accel lean
    prev_speed: f32,
    /// smoothed accel lean (root pitch), ±0.07 rad
    accel_lean: f32,
    /// smoothed sprint low-ready blend 0..1 (in 220 ms, out 140 ms)
    sprint_t: f32,
    /// §2.2 carry→aim blend 0..1 (aim in 220 ms / out 140 ms)
    carry_t: f32,
    /// §2.3 weapon-mass settle: the gun lags the spine on turns
    prev_yaw_vis: f32,
    wr_lag_yaw: f32,
    wr_lag_v: f32,
    /// per side: [thigh (hip pivot), shin (knee pivot), foot (ankle pivot)]
    leg_l: [Entity; 3],
    leg_r: [Entity; 3],
    torso: Entity,
    /// the neck pivot - sits EXACTLY on the 0.82 band line (§1.1), so
    /// head rotation stays inside the head hit band
    neck: Entity,
    /// per arm: [upper (shoulder pivot), forearm (elbow pivot), hand (wrist)]
    arm_l: [Entity; 3],
    arm_r: [Entity; 3],
    /// §1.3: the gun is parented HERE on the spine - the gun leads, the
    /// hands follow via two-bone IK. Never parented to a hand.
    weapon_root: Entity,
    /// held weapon models, indexed by `weapon_slot`
    weapons: [Entity; N_WEAPONS],
    /// the always-carried shield, shown raised on the left arm
    shield: Entity,
    bow_arrow: Entity,
    armor_rig: Entity,
}

/// §1.4: metres per full step. Phase advances by planar distance / stride
/// - zero foot sliding at every speed falls out for free.
const STRIDE_M: f32 = 1.45;

// ---- §1 (Brief IV): rig connectivity ------------------------------------
// THE RULE: every parent–child pair overlaps or is bridged by joint
// geometry - zero daylight in any pose. Rotations preserve bone lengths,
// so overlaps asserted here hold across EVERY animation phase; these
// constants are shared by the spawn code and the gap unit test so they
// cannot drift apart.
/// Neck: a dark cylinder bridging yoke → head, in torso-local Y.
const NECK_R: f32 = 0.055;
const NECK_BOT: f32 = 0.64; // sunk ≥2 cm into the yoke (top 0.695)
const NECK_TOP: f32 = 0.89; // ≥1.5 cm past the head pivot (0.846)
/// Yoke half-width must reach past the shoulder pivots (±0.26).
const YOKE_HALF_W: f32 = 0.27;
const SHOULDER_X: f32 = 0.26;
/// Arm chain (all local Y, downward negative): shell spans must reach
/// their joint balls.
const UPPER_CENTER: f32 = -0.115;
const UPPER_HALF: f32 = 0.14 / 2.0 + 0.055; // capsule half-len + radius
const ELBOW_Y: f32 = -0.21;
const ELBOW_R: f32 = 0.05;
const FORE_CENTER: f32 = -0.09;
const FORE_HALF: f32 = 0.12 / 2.0 + 0.048;
const WRIST_Y: f32 = -0.19;
const WRIST_R: f32 = 0.04;

// ---- §5 (Brief IV): weapon–body clipping ---------------------------------
// The stock was piercing the chest: weapon_root sat too close to the
// torso centerline. These offsets hold the weapon's rear point OUTSIDE
// the chest ellipse (half-extents 0.20 × 0.15 at chest height) for the
// longest stock in the arsenal - asserted by the sweep test.
const WR_X: f32 = 0.14;
const WR_Z_HIP: f32 = 0.23;
const WR_Z_ADS: f32 = 0.19;

/// Rear extent (stock length behind the grip) per weapon, metres.
fn weapon_rear_extent(kind: GunKind) -> f32 {
    match kind {
        GunKind::Fists => 0.0,
        GunKind::Glock | GunKind::Deagle => 0.10,
        GunKind::Mp5 => 0.15, // folded stock - rear cap only
        GunKind::Shotgun => 0.34,
        GunKind::Ak47 | GunKind::M4 | GunKind::M249 => 0.36,
        GunKind::Awm => 0.39,
        GunKind::Bow => 0.12,
        GunKind::Spear => 0.63, // carried overhead - cleared vertically
        // §7: no stock at all - the rear grip and motor housing brace
        // AGAINST the torso; that contact is the carry, not a clip
        GunKind::Minigun => 0.15,
    }
}

/// §0.2: the head-band fraction the sim's hit-zone classifier uses.
/// Rendered head geometry must never fall below this fraction of the
/// fighter's height, or you can see a head you cannot shoot.
const HEAD_BAND_FRAC: f32 = 0.82;
/// The vertical offset from hip to head base at zero torso pitch.
const HEAD_OVER_HIP: f32 = 0.846;
/// §2 (Brief V): peak weight-absorb dip after a roll/side-step ends.
/// Requested depth - the band clamp below may allow less.
const ROLL_SETTLE_DIP: f32 = 0.055;

/// §1.4 pose core: (hip_y, torso_pitch) for a grounded gait at phase θ.
/// PURE - shared by `sync_fighters` and the §0.2 band test, so the test
/// can never drift from the render. Pelvis bob is 2× frequency with its
/// MINIMUM at double support, and only ever ADDS height; total torso
/// pitch is capped so the head can never dip below the 0.82 line.
///
/// `settle` (0..1) is the post-roll weight-absorb dip. It lives HERE
/// rather than at the call site because it MOVES THE WHOLE RIG DOWN:
/// applied afterwards, the renderer was writing a hip 5.5 cm below the
/// value `head_base_y` reported, which put the head base at ~0.79 of
/// height - outside the 0.82 band the test claims to guard, and
/// classified as Arms by the sim while looking like a head.
fn gait_pose(crouch: bool, theta: f32, amp: f32, accel_lean: f32, settle: f32) -> (f32, f32) {
    let (hip, pitch) = if crouch {
        (
            0.54 + 0.018 * (1.0 - (2.0 * theta).cos()) * amp,
            0.90, // ~52°: projects the 0.324 m head into the crouch band
        )
    } else {
        (
            0.63 + 0.0175 * (1.0 - (2.0 * theta).cos()) * amp,
            // stronger run lean - the band cap still rules (§0.2 test)
            (0.05 + amp * 0.09 + accel_lean).min(0.185),
        )
    };
    // The band is law, so the dip is clamped BY it rather than checked
    // against it afterwards: whatever depth still leaves the head base on
    // the 0.82 line is what the settle gets. At a hard run lean that is
    // nearly nothing; standing, it is ~1.5 cm. A deeper absorb would need
    // to raise the torso to compensate, which is a pose change, not a
    // constant - noted in the handback rather than guessed at here.
    let drop = if crouch { 0.12 } else { 0.0 };
    let height = if crouch { CROUCH_HEIGHT } else { BODY_HEIGHT };
    let min_hip = HEAD_BAND_FRAC * height + drop - HEAD_OVER_HIP * pitch.cos();
    let dipped = (hip - ROLL_SETTLE_DIP * settle.clamp(0.0, 1.0)).max(min_hip);
    (dipped, pitch)
}

/// §0.2: world-Y of the head's LOWEST geometry for a grounded pose (the
/// head pivot - geometry sits entirely above it).
fn head_base_y(crouch: bool, theta: f32, amp: f32, accel_lean: f32, settle: f32) -> f32 {
    let (hip, pitch) = gait_pose(crouch, theta, amp, accel_lean, settle);
    let drop = if crouch { 0.12 } else { 0.0 };
    hip - drop + HEAD_OVER_HIP * pitch.cos()
}

/// §2.3 (Brief III): weapon-mass settle - (lag seconds, damping). Heavy
/// weapons lag the spine and OVERSHOOT on direction changes; SMGs and
/// pistols snap. One constant per weapon differentiates the whole arsenal.
fn weapon_lag(kind: GunKind) -> (f32, f32) {
    match kind {
        GunKind::M249 | GunKind::Awm => (0.055, 0.60), // underdamped: 1.4° overshoot
        GunKind::Ak47 | GunKind::M4 | GunKind::Shotgun => (0.035, 0.85),
        GunKind::Spear | GunKind::Bow => (0.030, 0.85),
        _ => (0.020, 1.0), // MP5, pistols, fists: critically damped, no lag
    }
}

/// §1.3 analytic two-bone IK, solved in torso space: shoulder at `s`,
/// wrist target `t`, elbow steered toward `pole` (down-and-out ~35° -
/// what stops arms looking broken). Returns the shoulder rotation and
/// the elbow hinge flex; with the rig's −X elbow hinge the chain lands
/// EXACTLY on the target (within reach).
// ---- §2 (Brief VII v2): the joint-limit library --------------------------
// Every procedural pose passes through these before it reaches a
// Transform. Ranges are from measured human active range of motion.
// COSMETIC layer only (C3) - clamps rendering, never sim state.
const ELBOW_FLEX_MIN_DEG: f32 = -5.0; // hyperextension hard-clamp
const ELBOW_FLEX_MAX_DEG: f32 = 145.0;

/// §2.2: elbow flexion clamp, in radians in, radians out.
fn clamp_elbow_flex(flex_rad: f32) -> f32 {
    flex_rad.to_degrees().clamp(ELBOW_FLEX_MIN_DEG, ELBOW_FLEX_MAX_DEG).to_radians()
}

/// §2.2 coupling: DIP flexion tracks ~0.7x the driving (PIP-equivalent)
/// joint - tendon-linked in real hands. Independent DIP motion is the
/// single most common "robotic finger" tell; this is the fix for it.
const DIP_PIP_COUPLING: f32 = 0.7;
fn dip_from_driving_joint(driving_rot: f32) -> f32 {
    driving_rot * DIP_PIP_COUPLING
}

/// Task 3 rule 5 (MISSION doc): the landing camera offset `t` seconds
/// after touchdown. POSITIVE pushes the camera down.
///
/// The rebound is a DELAYED counter-push. Run simultaneously with the
/// dip, a rebound that starts smaller and decays faster can only ever
/// shrink the dip - it can never carry the camera past neutral, which
/// makes "landings never fully damp in one frame" just a slower one-way
/// decay. Delaying its onset until the dip has mostly decayed is what
/// actually produces the lift the rule asks for.
const LAND_DIP_DECAY: f32 = 11.0;
const LAND_REBOUND_DELAY_S: f32 = 0.085;
const LAND_REBOUND_WIN_S: f32 = 0.13;

fn landing_offset(dip_amp: f32, rebound_amp: f32, t: f32) -> f32 {
    let t = t.max(0.0);
    let dip = dip_amp * (-LAND_DIP_DECAY * t).exp();
    let reb = if t >= LAND_REBOUND_DELAY_S && t <= LAND_REBOUND_DELAY_S + LAND_REBOUND_WIN_S {
        rebound_amp * ((t - LAND_REBOUND_DELAY_S) / LAND_REBOUND_WIN_S * PI).sin()
    } else {
        0.0
    };
    dip - reb
}

/// §2.5 (Brief VII v2): the critically-damped spring - closed form, so
/// 60fps and 240fps agree bit-for-bit instead of drifting apart. This is
/// the ONE spring primitive behind every secondary-motion element in the
/// brief's table (hand follow k=120, elbow pole k=60, finger settle
/// k=220, shoulder/clavicle k=45, camera boom k=90) - previously each
/// use site hand-rolled its own copy (the viewmodel sway spring below
/// was the only one that existed); this is that same math, named and
/// reusable. `x'' = -k(x-target) - c*x'`, c = 2*sqrt(k) (critical).
fn damped_spring(x: Vec2, v: Vec2, target: Vec2, k: f32, dt: f32) -> (Vec2, Vec2) {
    let w = k.sqrt();
    let decay = (-w * dt).exp();
    let x0 = x - target;
    let new_x = target + (x0 + (v + x0 * w) * dt) * decay;
    let new_v = (v - (v + x0 * w) * w * dt) * decay;
    (new_x, new_v)
}

/// §2.5 named spring stiffnesses (k) from the brief's table - critical
/// damping is derived (c = 2*sqrt(k)), never tuned separately.
const SPRING_K_HAND_FOLLOW: f32 = 120.0;
const SPRING_K_ELBOW_POLE: f32 = 60.0;
const SPRING_K_FINGER_SETTLE: f32 = 220.0;
const SPRING_K_SHOULDER: f32 = 45.0;
const SPRING_K_CAMERA_BOOM: f32 = 90.0;

/// §2.5/§5.2: the boom-collision push-out - the actual SPRING_K_CAMERA_
/// BOOM consumer named in the brief's original table (it had been
/// documented as wired but was actually still a plain first-order
/// `CAM_RECOVER_S` chase; this closes that gap). Extracted to a scalar
/// helper (spring state lives on one axis - the boom LENGTH, not a
/// position) so the critical-damping behavior is directly testable
/// without a running camera_system.
fn boom_recover(boom: f32, boom_vel: f32, allowed: f32, dt: f32) -> (f32, f32) {
    let (nx, nv) = damped_spring(
        Vec2::new(boom, 0.0),
        Vec2::new(boom_vel, 0.0),
        Vec2::new(allowed, 0.0),
        SPRING_K_CAMERA_BOOM,
        dt,
    );
    (nx.x, nv.x)
}

/// §5.2: one boom update. `allowed` is the collision-limited distance
/// (== `free_len` when nothing is in the way); `free_len` is what the
/// boom would be with no cover at all.
///
/// The k=90 spring exists to stop the camera POPPING when it clears a
/// corner, so it must apply to exactly that case and nothing else.
/// Applying it to every increase in `allowed` also filtered ordinary
/// free-space boom changes - the 0.12 s sprint ease, the 0.12 s ADS
/// blend, and the length change from plain vertical mouse-look - putting
/// two filters in series and letting the heavier one silently win
/// (measured: the sprint boom-out reached 90% at ~0.55 s instead of the
/// documented ~0.25 s, and pitching down lagged ~18 cm while pitching up
/// snapped instantly).
/// Returns `(boom, boom_vel, still_occluded)`. `hit` is whether the
/// collision ray actually struck cover this frame; `was_occluded` is the
/// same flag from last frame.
///
/// The occlusion flag has to be explicit state: "boom is shorter than the
/// free-space target" is TRUE both while recovering from a wall and while
/// an eased target is simply growing, so a distance comparison alone
/// cannot tell them apart and ends up springing both.
fn boom_step(
    boom: f32,
    boom_vel: f32,
    was_occluded: bool,
    allowed: f32,
    free_len: f32,
    hit: bool,
    dt: f32,
) -> (f32, f32, bool) {
    const CLEAR_EPS: f32 = 0.01;
    if hit {
        if allowed < boom {
            // contact: pull in immediately, or the camera ends up inside
            // the very wall it is avoiding
            (allowed, 0.0, true)
        } else {
            // still occluded but the gap is opening
            let (nb, nv) = boom_recover(boom, boom_vel, allowed, dt);
            (nb, nv, true)
        }
    } else if was_occluded && boom < free_len - CLEAR_EPS {
        // just cleared the corner - THIS is the pop the k=90 spring
        // exists to smooth, and the only case it should govern
        let (nb, nv) = boom_recover(boom, boom_vel, free_len, dt);
        (nb, nv, true)
    } else {
        // free space: the sprint ease, the ADS blend and mouse-look all
        // own their own documented timings. Track them directly instead
        // of re-filtering through a heavier spring on top.
        (free_len, 0.0, false)
    }
}

/// §2.4 (Brief VII v2): trigger finger travel curve - out over 0.06s,
/// back over 0.10s, given seconds-since-last-shot. Pure so the exact
/// timing is testable without a running app.
const TRIGGER_OUT_S: f32 = 0.06;
const TRIGGER_BACK_S: f32 = 0.10;
fn trigger_finger_press(t_since: f32) -> f32 {
    if t_since < TRIGGER_OUT_S {
        t_since / TRIGGER_OUT_S
    } else {
        (1.0 - (t_since - TRIGGER_OUT_S) / TRIGGER_BACK_S).max(0.0)
    }
}

fn solve_arm_ik(s: Vec3, t: Vec3, pole: Vec3) -> (Quat, f32) {
    const L1: f32 = 0.21; // shoulder → elbow
    const L2: f32 = 0.21; // elbow → wrist (incl. mitten reach)
    let to = t - s;
    let d = to.length().clamp(0.08, L1 + L2 - 0.005);
    let dir = to.normalize_or(Vec3::NEG_Y);
    let cos_e = ((L1 * L1 + L2 * L2 - d * d) / (2.0 * L1 * L2)).clamp(-1.0, 1.0);
    // §2.2 (Brief VII v2): the solved flex is now HARD-CLAMPED to the
    // biomechanical range - a two-bone solver has no innate reason to
    // respect a human elbow, so the clamp is what actually enforces it.
    let flex = clamp_elbow_flex(PI - cos_e.acos());
    let a1 = ((L1 * L1 - L2 * L2 + d * d) / (2.0 * d)).clamp(-L1, L1);
    let r = (L1 * L1 - a1 * a1).max(0.0).sqrt();
    let side = (pole - dir * pole.dot(dir)).normalize_or(Vec3::NEG_Z);
    let elbow = s + dir * a1 + side * r;
    let up = (elbow - s).normalize_or(Vec3::NEG_Y);
    let fore = (t - elbow).normalize_or(up);
    let z = (fore - up * fore.dot(up)).normalize_or(Vec3::Z);
    let y = -up;
    let x = y.cross(z).normalize_or(Vec3::X);
    (Quat::from_mat3(&Mat3::from_cols(x, y, z)), flex)
}

/// First-person viewmodel: hands + weapon parented to the camera.
#[derive(Resource)]
struct VmRig {
    root: Entity,
    weapons: [Entity; N_WEAPONS],
    shield: Entity,
}

/// Extra weapon greebles that only show while aiming - the ADS detail pass.
#[derive(Component)]
struct AdsDetail;

/// The predicted-arc preview for bow/spear aiming (§4.2 Brief II):
/// arc-length-spaced dots, a landing ring + drop-line, and a ±spread
/// cone of two fainter arcs that widens as stability degrades.
#[derive(Resource)]
struct ArcVis {
    dots: Vec<Entity>,
    /// two fainter arcs at ±current spread - 8 dots each
    cone: Vec<Entity>,
    ring: Entity,
    drop_line: Entity,
}

/// §1 (Brief V): the grenade pre-aim preview - amber dots along the
/// predicted flight, a fainter run after the first bounce (less certain),
/// and a landing ring at the predicted end point.
#[derive(Resource)]
struct GrenadeArcVis {
    /// bright dots: launch → first bounce
    pre: Vec<Entity>,
    /// faint dots: after the first bounce
    post: Vec<Entity>,
    ring: Entity,
}

/// What the preview computed this frame - the HUD prints the range.
#[derive(Resource, Default)]
struct ArcState {
    range: Option<f32>,
}

/// Model-index for every carriable weapon (v6 roster).
const N_WEAPONS: usize = 11;

/// §2.3: render layer for the first-person viewmodel - seen only by the
/// dedicated fixed-FOV viewmodel camera, never by the world camera.
const VIEWMODEL_LAYER: usize = 1;
/// Viewmodel camera FOV. §1.2 (Brief VI): CS:GO Classic preset = 68°.
const VM_FOV_DEG: f32 = 68.0;
// ---- §1.3 (Brief VI): the no-bounce contract ------------------------------
/// Below this fraction of sprint speed the bob is EXACTLY zero - the
/// CS:GO property: the bob clock freezes when you stop.
const VM_BOB_DEADZONE: f32 = 0.05;
/// Airborne bob multiplier (CS:GO's exact ÷5).
const VM_AIR_BOB: f32 = 0.2;
/// Mouse-sway rotational cap, degrees (Brief VI: 0.3°).
const VM_SWAY_CAP_DEG: f32 = 0.3;
/// Fire back-slide: ≤ 1.5 cm along the barrel, returned in ≤ 120 ms.
const VM_KICK_SLIDE_M: f32 = 0.015;
const VM_KICK_RETURN_S: f32 = 0.12;
/// §1.4a screen-intrusion budgets: every weapon's geometry must fit a
/// two-part envelope around the vm root - a RECEIVER box (wide but low)
/// and a MAST (sights/scope: tall but narrow). The sweep test proves the
/// ROOT can never carry either part across the vertical midline or into
/// the central 12%-of-screen-height circle. Current widest receiver =
/// minigun cluster (0.069 left); tallest mast = AWM scope (0.085 up).
const VM_RECEIVER_LEFT: f32 = 0.09;
const VM_RECEIVER_UP: f32 = 0.05;
const VM_MAST_LEFT: f32 = 0.03;
const VM_MAST_UP: f32 = 0.09;

/// §1.3 (Brief VI): the carry-motion CORE - pure. The viewmodel root's
/// translation offset (camera space, meters) from bob/kick/sprint/dip.
/// Shared verbatim by `fp_viewmodel` and the §1.4 no-bounce tests, so
/// the tests measure the real motion, not a copy of it.
fn carry_offset(
    speed_frac: f32,
    theta: f32,
    grounded: bool,
    kick: f32,
    sprint_e: f32,
    dip: f32,
    wind: f32,
) -> Vec3 {
    let s = if speed_frac < VM_BOB_DEADZONE {
        0.0 // standing = frozen bob clock = zero positional motion
    } else {
        speed_frac
    };
    let air = if grounded { 1.0 } else { VM_AIR_BOB };
    let bob = Vec2::new(
        0.0065 * s * theta.sin(),
        0.004 * s * (2.0 * theta).sin(),
    ) * air;
    Vec3::new(
        bob.x + sprint_e * 0.02 + wind * 0.07,
        // the run-lower and the landing dip pull DOWN, never up
        bob.y - dip - sprint_e * 0.06 + wind * 0.09,
        kick * VM_KICK_SLIDE_M + wind * 0.30,
    )
}

/// §1.1 Rule 2 (Brief VI): while a scoped-class weapon is zoomed the
/// viewmodel is NOT RENDERED at all - pure predicate, shared by the
/// render path and the scope-hide test.
fn vm_hidden_while_scoped(gun_is_scoped: bool, ads: bool) -> bool {
    gun_is_scoped && ads
}

/// §1.2 (Brief VI): one segment of the on-weapon ammo bar - an emissive
/// tick on the LEFT receiver face, viewmodel only. Segments extinguish
/// as the magazine drains; all pulse once when a reload completes.
#[derive(Component)]
struct AmmoBarSeg {
    idx: usize,
}
const AMMO_BAR_SEGS: usize = 8;

/// Left-face standoff for the bar, per receiver width.
fn ammo_bar_x(kind: GunKind) -> f32 {
    match kind {
        GunKind::Glock | GunKind::Deagle => -0.030,
        GunKind::M249 => -0.044,
        GunKind::Minigun => -0.072,
        _ => -0.032,
    }
}

fn weapon_slot(kind: GunKind) -> Option<usize> {
    match kind {
        GunKind::Fists => None,
        GunKind::Glock => Some(0),
        GunKind::Deagle => Some(1),
        GunKind::Mp5 => Some(2),
        GunKind::Shotgun => Some(3),
        GunKind::Ak47 => Some(4),
        GunKind::M4 => Some(5),
        GunKind::Awm => Some(6),
        GunKind::M249 => Some(7),
        GunKind::Bow => Some(8),
        GunKind::Spear => Some(9),
        GunKind::Minigun => Some(10),
    }
}

const ALL_WEAPONS: [GunKind; N_WEAPONS] = [
    GunKind::Glock,
    GunKind::Deagle,
    GunKind::Mp5,
    GunKind::Shotgun,
    GunKind::Ak47,
    GunKind::M4,
    GunKind::Awm,
    GunKind::M249,
    GunKind::Bow,
    GunKind::Spear,
    GunKind::Minigun,
];

#[allow(dead_code)] // §7 dropped the caliber flavor line; kept for tooling
fn ammo_kind(kind: GunKind) -> &'static str {
    match kind {
        GunKind::Fists => "",
        GunKind::Glock => "9MM ROUNDS",
        GunKind::Deagle => ".50 AE",
        GunKind::Mp5 => "9MM ROUNDS",
        GunKind::Shotgun => "12 GAUGE",
        GunKind::Ak47 => "7.62 ROUNDS",
        GunKind::M4 => "5.56 ROUNDS",
        GunKind::Awm => ".338 LAPUA",
        GunKind::M249 => "5.56 BELT",
        GunKind::Bow => "ARROWS",
        GunKind::Spear => "WAR SPEARS",
        GunKind::Minigun => "7.62 BELT",
    }
}

// ---- §3 (Brief VI): the four-corner HUD, data-driven ----------------------
// Each corner carries exactly ONE info cluster; the center stays empty
// except the crosshair and transients. Anchors are normalized [0..1]
// screen fractions + pixel offsets, consumed by the spawn code and
// asserted by the layout test at three resolutions.
/// 5% safe area: nothing sits closer than this to any screen edge.
const HUD_SAFE_FRAC: f32 = 0.05;
/// (name, anchor [x,y] in 0..1, offset as SCREEN FRACTIONS - fractional
/// offsets keep the 5% safe area at every resolution by construction)
const HUD_ANCHORS: &[(&str, [f32; 2], [f32; 2])] = &[
    ("vitals", [0.0, 1.0], [0.06, -0.09]),     // bottom-left
    ("ammo", [1.0, 1.0], [-0.06, -0.09]),      // bottom-right
    ("minimap-kd", [0.0, 0.0], [0.06, 0.06]),  // top-left
    ("timer-score", [0.5, 0.0], [0.0, 0.055]), // top-center
    ("killfeed", [1.0, 0.0], [-0.06, 0.06]),   // top-right
];

/// §3.1: vitals color - white nominal, RED at ≤25, pulsing at ≤20.
/// Pure, shared by the HUD system and the threshold test.
fn vitals_color(hp: f32, t: f32) -> Color {
    if hp <= 20.0 {
        let a = 0.75 + 0.25 * (t * 6.0).sin().abs();
        Color::srgba(1.0, 0.18, 0.15, a)
    } else if hp <= 25.0 {
        Color::srgb(1.0, 0.18, 0.15)
    } else {
        Color::srgb(0.95, 0.96, 0.98)
    }
}

/// §3.2: the magazine number turns red at ≤25% of the mag.
fn ammo_is_low(ammo: u32, mag: u32) -> bool {
    mag > 0 && (ammo as f32) <= mag as f32 * 0.25
}

/// §3.5: killfeed modifier glyphs. Implemented: headshot (*). The other
/// CS:GO glyphs (wallbang, noscope, through-smoke, blind, flash-assist)
/// need sim events this game does not track yet - deferred, documented.
/// §0 (Brief VII): ASCII only - the bundled font has no glyph for U+271B.
fn feed_glyphs(headshot: bool) -> &'static str {
    if headshot {
        " * "
    } else {
        "  "
    }
}

#[derive(Component)]
struct CoverVis;

#[derive(Component)]
struct HillVis;

#[derive(Component)]
struct PickupVis {
    index: usize,
    item: Entity,
}

/// The glowing ring over a respawn checkpoint; tinted by its owner.
#[derive(Component)]
struct CheckpointVis {
    index: usize,
}

#[derive(Component)]
struct HealthBarVis {
    index: usize,
    fill: Entity,
    afill: Entity,
}

#[derive(Component)]
struct BarFill;

#[derive(Component)]
struct TracerMarker;

/// §3: a recoverable arrow/spear pile lying on the ground.
#[derive(Component)]
struct DroppedMarker;

#[derive(Resource, Default)]
struct DroppedPool(Vec<Entity>);

/// §5 throwable visuals: grenades in flight, smoke spheres, fire pools,
/// detonation flashes - one pooled entity set each.
#[derive(Component)]
struct GrenadeMarker;
#[derive(Component)]
struct SmokeMarker;
#[derive(Component)]
struct FireMarker;
#[derive(Component)]
struct BoomMarker;
/// Full-screen §5.3 flash whiteout (quantised steps, not an alpha fade).
#[derive(Component)]
struct FlashOverlay;

/// §8: pooled horde visuals - one body + one head per zombie, plus the
/// extraction beacon pillar.
#[derive(Component)]
struct ZombieMarker;

#[derive(Resource, Default)]
struct ZombiePool {
    bodies: Vec<Entity>,
    heads: Vec<Entity>,
    beacon: Option<Entity>,
}

#[derive(Resource)]
struct ZombieAssets {
    moss: Handle<StandardMaterial>,
    pale: Handle<StandardMaterial>,
    beacon: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
struct ThrowPools {
    grenades: Vec<Entity>,
    smokes: Vec<Entity>,
    fires: Vec<Entity>,
    booms: Vec<Entity>,
}

#[derive(Resource)]
struct ThrowAssets {
    ball: Handle<Mesh>,
    body: Handle<StandardMaterial>,
    smoke: Handle<StandardMaterial>,
    fire_mesh: Handle<Mesh>,
    fire: Handle<StandardMaterial>,
    flashband: Handle<StandardMaterial>,
}

#[derive(Component)]
struct MissileMarker;

#[derive(Component)]
struct DecalMarker;

#[derive(Component)]
struct MainCam;

#[derive(Resource, Default)]
struct TracerPool(Vec<Entity>);

#[derive(Resource, Default)]
struct MissilePool(Vec<Entity>);

#[derive(Resource, Default)]
struct DecalPool(Vec<Entity>);

#[derive(Resource)]
struct FxAssets {
    tracer_mesh: Handle<Mesh>,
    tracer_blue: Handle<StandardMaterial>,
    tracer_red: Handle<StandardMaterial>,
    missile_mesh: Handle<Mesh>,
    arrow_mat: Handle<StandardMaterial>,
    spear_mat: Handle<StandardMaterial>,
    decal_mesh: Handle<Mesh>,
    decal_mat: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct BarAssets {
    green: Handle<StandardMaterial>,
    orange: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
}

/// Shared meshes + materials for building weapon / gear models.
#[derive(Resource, Clone)]
struct ModelKit {
    cube: Handle<Mesh>,
    cyl: Handle<Mesh>, // unit cylinder: radius 0.5, height 1, axis Y
    ball: Handle<Mesh>, // unit sphere: radius 0.5
    gunmetal: Handle<StandardMaterial>,
    steel: Handle<StandardMaterial>,
    wood: Handle<StandardMaterial>,
    string: Handle<StandardMaterial>,
    lens: Handle<StandardMaterial>,
    olive: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    white: Handle<StandardMaterial>,
    med_glow: Handle<StandardMaterial>,
    armor_dark: Handle<StandardMaterial>,
    core_glow: Handle<StandardMaterial>,
    /// the gripping mitt - §1 restyles this to the robot's shell
    hand: Handle<StandardMaterial>,
    // §2.1 weapon palette: four flat greys, tone changes - not geometry -
    // suggest complexity (the reference's rail notches are tone stripes)
    grey_light: Handle<StandardMaterial>,
    grey_mid: Handle<StandardMaterial>,
    grey_dark: Handle<StandardMaterial>,
    grey_black: Handle<StandardMaterial>,
    // §11 (Brief IV): the Mech Armada palette - khaki faceted plates
    // over shadowed joints, one red sensor slit
    mech_khaki: Handle<StandardMaterial>,
    mech_khaki_dk: Handle<StandardMaterial>,
    mech_khaki_lt: Handle<StandardMaterial>,
    mech_shadow: Handle<StandardMaterial>,
    mech_metal: Handle<StandardMaterial>,
    mech_red: Handle<StandardMaterial>,
    /// §4.2 (Brief VI): yellow-black hazard accents.
    mech_hazard: Handle<StandardMaterial>,
}

/// §2.1 tone slots of the weapon palette.
#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Light,
    Mid,
    Dark,
    Black,
}

impl ModelKit {
    fn tone(&self, t: Tone) -> Handle<StandardMaterial> {
        match t {
            Tone::Light => self.grey_light.clone(),
            Tone::Mid => self.grey_mid.clone(),
            Tone::Dark => self.grey_dark.clone(),
            Tone::Black => self.grey_black.clone(),
        }
    }
}

/// §2.1 shared part vocabulary: every gun is blocks + cylinders in the
/// four-grey palette. Local frame: root at the grip, muzzle-forward = +Z
/// (the bow stands along +Y with its string toward −Z).
struct WPart {
    cyl: bool,
    tone: Tone,
    pos: Vec3,
    /// radians about X - magazines and stocks are raked, never square
    tilt: f32,
    size: Vec3,
    /// true → ADS-only greeble (rides the existing detail-LOD path)
    detail: bool,
}

fn wp(cyl: bool, tone: Tone, pos: (f32, f32, f32), tilt: f32, size: (f32, f32, f32)) -> WPart {
    WPart {
        cyl,
        tone,
        pos: Vec3::new(pos.0, pos.1, pos.2),
        tilt,
        size: Vec3::new(size.0, size.1, size.2),
        detail: false,
    }
}

fn wd(cyl: bool, tone: Tone, pos: (f32, f32, f32), tilt: f32, size: (f32, f32, f32)) -> WPart {
    WPart {
        detail: true,
        ..wp(cyl, tone, pos, tilt, size)
    }
}

/// The reference's signature top rail: a repeated notch pattern -
/// alternating light/mid blocks over a dark base bar.
fn push_rail(parts: &mut Vec<WPart>, y: f32, z0: f32, z1: f32, n: usize) {
    parts.push(wp(false, Tone::Dark, (0.0, y, (z0 + z1) * 0.5), 0.0, (0.030, 0.018, z1 - z0)));
    let step = (z1 - z0) / n as f32;
    for i in 0..n {
        let tone = if i % 2 == 0 { Tone::Light } else { Tone::Mid };
        parts.push(wp(
            false,
            tone,
            (0.0, y + 0.014, z0 + step * (i as f32 + 0.5)),
            0.0,
            (0.034, 0.012, step * 0.55),
        ));
    }
}

/// Skeletal stock: an OUTLINE of thin bars with a hole through it, never
/// a solid block - top bar, bottom bar, rear vertical, shoulder pad.
fn push_stock(parts: &mut Vec<WPart>, z_rear: f32, drop: f32) {
    let len = -z_rear - 0.04; // grip → rear
    let zc = z_rear * 0.5 - 0.02;
    parts.push(wp(false, Tone::Dark, (0.0, 0.035, zc), 0.06, (0.024, 0.024, len)));
    parts.push(wp(false, Tone::Dark, (0.0, -drop, zc), -0.10, (0.024, 0.024, len)));
    parts.push(wp(false, Tone::Dark, (0.0, (0.035 - drop) * 0.5, z_rear), 0.0, (0.026, drop + 0.10, 0.028)));
    parts.push(wp(false, Tone::Mid, (0.0, (0.035 - drop) * 0.5, z_rear - 0.018), 0.0, (0.030, drop + 0.13, 0.014)));
}

/// Muzzle device: a slightly wider dark block with a visible black bore
/// recess poking out of it.
fn push_muzzle(parts: &mut Vec<WPart>, y: f32, z: f32, w: f32) {
    parts.push(wp(false, Tone::Dark, (0.0, y, z), 0.0, (w, w, 0.07)));
    parts.push(wp(true, Tone::Black, (0.0, y, z + 0.028), FRAC_PI_2, (w * 0.45, 0.03, w * 0.45)));
}

/// §1.3: hand placements per weapon - (position, yaw, curl, mirrored) in
/// weapon-local space. Entry 0 is the GRIP socket, entry 1 (if any) the
/// FOREGRIP socket. One table serves both the viewmodel's posed hands and
/// the third-person two-bone IK, so they can never disagree.
fn weapon_hand_specs(kind: GunKind) -> Vec<(Vec3, f32, f32, bool)> {
    match kind {
        GunKind::Fists => vec![],
        // §2.2 (image 2): pistols get the two-handed CUP grip - support
        // hand wraps the firing hand from the left
        GunKind::Glock => vec![
            (Vec3::new(0.0, -0.05, -0.015), 0.0, 0.9, false),
            (Vec3::new(-0.028, -0.065, 0.005), 0.35, 0.8, true),
        ],
        GunKind::Deagle => vec![
            (Vec3::new(0.0, -0.055, -0.015), 0.0, 0.9, false),
            (Vec3::new(-0.030, -0.070, 0.005), 0.35, 0.8, true),
        ],
        GunKind::Mp5 => vec![
            (Vec3::new(0.0, -0.07, -0.055), 0.0, 1.0, false),
            (Vec3::new(0.0, -0.045, 0.16), 0.15, 0.55, true),
        ],
        GunKind::Shotgun => vec![
            (Vec3::new(0.0, -0.045, -0.06), 0.0, 1.0, false),
            (Vec3::new(0.0, -0.05, 0.30), 0.0, 0.55, true),
        ],
        GunKind::Ak47 => vec![
            (Vec3::new(0.0, -0.08, -0.06), 0.0, 1.0, false),
            (Vec3::new(0.0, -0.035, 0.32), 0.1, 0.55, true),
        ],
        GunKind::M4 => vec![
            (Vec3::new(0.0, -0.08, -0.06), 0.0, 1.0, false),
            (Vec3::new(0.0, -0.045, 0.24), 0.1, 0.55, true),
        ],
        GunKind::Awm => vec![
            (Vec3::new(0.0, -0.075, -0.10), 0.0, 1.0, false),
            (Vec3::new(0.0, -0.05, 0.26), 0.1, 0.55, true),
        ],
        GunKind::M249 => vec![
            (Vec3::new(0.0, -0.07, -0.08), 0.0, 1.0, false),
            (Vec3::new(0.0, -0.10, 0.22), 0.1, 0.55, true),
        ],
        GunKind::Bow => vec![(Vec3::new(0.0, 0.0, 0.03), 0.0, 0.8, true)],
        GunKind::Spear => vec![(Vec3::new(0.0, -0.02, 0.0), 0.0, 1.0, false)],
        // §7: rear spade grip + the side support handle - the hip carry
        GunKind::Minigun => vec![
            (Vec3::new(0.0, -0.06, -0.09), 0.0, 1.0, false),
            (Vec3::new(-0.02, -0.11, 0.18), 0.1, 0.6, true),
        ],
    }
}

// ---- audio ---------------------------------------------------------------

#[derive(Resource)]
struct Sfx {
    shot_glock: Handle<AudioSource>,
    shot_deagle: Handle<AudioSource>,
    shot_mp5: Handle<AudioSource>,
    shot_shotgun: Handle<AudioSource>,
    shot_ak: Handle<AudioSource>,
    shot_rifle: Handle<AudioSource>, // M4
    shot_mg: Handle<AudioSource>,   // M249
    shot_sniper: Handle<AudioSource>, // AWM
    bow: Handle<AudioSource>,
    spear: Handle<AudioSource>,
    click: Handle<AudioSource>, // dry fire on an empty magazine
    shield: Handle<AudioSource>, // the plate takes a round
    hit: Handle<AudioSource>,
    headshot: Handle<AudioSource>,
    hurt: Handle<AudioSource>,
    pickup: Handle<AudioSource>,
    reload: Handle<AudioSource>,
    jump: Handle<AudioSource>,
    roll: Handle<AudioSource>,
    kill: Handle<AudioSource>,
    win: Handle<AudioSource>,
}

fn shot_sound<'a>(sfx: &'a Sfx, kind: GunKind) -> &'a Handle<AudioSource> {
    match kind {
        GunKind::Glock => &sfx.shot_glock,
        GunKind::Deagle => &sfx.shot_deagle,
        GunKind::Mp5 => &sfx.shot_mp5,
        GunKind::Shotgun => &sfx.shot_shotgun,
        GunKind::Ak47 => &sfx.shot_ak,
        GunKind::M4 => &sfx.shot_rifle,
        GunKind::Awm => &sfx.shot_sniper,
        GunKind::M249 => &sfx.shot_mg,
        GunKind::Bow => &sfx.bow,
        GunKind::Spear => &sfx.spear,
        GunKind::Minigun => &sfx.shot_mg,
        GunKind::Fists => &sfx.hit,
    }
}

#[derive(Resource)]
struct SfxState {
    last_t: f32,
    prev_gun: GunKind,
    prev_fire_cd: f32,
    prev_reserve: u32,
    prev_reload: f32,
    prev_vy: f32,
    prev_roll: f32,
    prev_armor: f32,
    prev_health: f32,
    prev_alive: bool,
    prev_over: bool,
    prev_shield: bool,
    click_cd: f32,
    max_missile: u32,
}

impl Default for SfxState {
    fn default() -> Self {
        SfxState {
            last_t: 0.0,
            prev_gun: GunKind::Fists,
            prev_fire_cd: 0.0,
            prev_reserve: 0,
            prev_reload: 0.0,
            prev_vy: 0.0,
            prev_roll: 0.0,
            prev_armor: 0.0,
            prev_health: MAX_HEALTH,
            prev_alive: true,
            prev_over: false,
            prev_shield: false,
            click_cd: 0.0,
            max_missile: 0,
        }
    }
}

// ---- minimap -------------------------------------------------------------

#[derive(Component)]
struct MinimapRoot;

#[derive(Component)]
struct MinimapDot(usize);

/// §4.3 (BRIEF VIII): "spotted enemies = red dots ghost-fading to last
/// known." Index into `SpottedEnemies.slots`, same fixed-slot pattern
/// as `MinimapDot` for teammates - pre-spawned, hidden until an enemy
/// occupies the slot.
#[derive(Component)]
struct MinimapEnemyDot(usize);

/// One tracked enemy's last-seen minimap state. `fade` is 1.0 while
/// currently in LOS, decays toward 0 once LOS breaks - the dot stays
/// pinned at `pos` (the LAST known position, not the enemy's live
/// position) while fading, which is the "ghosting" the brief names.
/// At fade<=0 the slot is free for a different enemy to claim.
#[derive(Clone, Copy, Default)]
struct SpotSlot {
    fighter: Option<usize>,
    pos: Vec2,
    fade: f32,
}

/// §4.3: client-side-only presentational state - what the LOCAL
/// player currently sees on their own minimap. Never read by sim.rs,
/// never affects a hit or an outcome, so it has no business being
/// replay-authoritative; it is derived fresh each frame from a real
/// `los_clear` query against sim state, same as any other visibility
/// effect in this file.
#[derive(Resource, Default)]
struct SpottedEnemies {
    slots: [SpotSlot; MINIMAP_ENEMY_SLOTS],
}
const MINIMAP_ENEMY_SLOTS: usize = 8;
/// How long a lost-LOS dot keeps fading before the slot frees up.
const MINIMAP_GHOST_FADE_S: f32 = 3.0;

#[derive(Component)]
struct MinimapCp(usize);

#[derive(Component)]
struct MinimapHill;

#[derive(Component)]
struct MinimapPlayer;

const MINIMAP_PX: f32 = 170.0;

// ---- AWM scope overlay ---------------------------------------------------

#[derive(Component)]
struct ScopeRoot;

// ---- UI ------------------------------------------------------------------

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct ScoreTimerText;

#[derive(Component)]
struct FeedText;

#[derive(Component)]
struct HitFeedText;

#[derive(Component)]
struct BannerText;

#[derive(Component)]
struct CrosshairText;

/// §4.2: numeric range beside the trajectory impact marker.
#[derive(Component)]
struct RangeText;

/// §7: compass strip at the top - cardinal window over the view yaw.
#[derive(Component)]
struct CompassText;

/// §9.1 (Brief IV): one cell of the vertical weapon strip on the right
/// screen edge - active slot at full opacity with an accent, inactive 40%.
#[derive(Component)]
struct WeaponStripCell(usize);

/// §9.1: the strip - updates names/opacity, fades after 4 s idle.
fn weapon_strip(
    time: Res<Time>,
    game: Res<Game>,
    mut last_active: Local<usize>,
    mut idle_t: Local<f32>,
    mut q: Query<(&WeaponStripCell, &mut Text, &mut TextColor)>,
) {
    let p = &game.sim.fighters[game.sim.player];
    if p.active != *last_active {
        *last_active = p.active;
        *idle_t = 0.0;
    } else {
        *idle_t += time.delta_secs();
    }
    let strip_fade = if *idle_t > 4.0 { 0.45 } else { 1.0 };
    for (cell, mut t, mut tc) in &mut q {
        let g = p.inventory[cell.0];
        let name = if g == GunKind::Fists { "-" } else { gun(g).name };
        let active = cell.0 == p.active;
        **t = if active {
            // §0 (Brief VII): ASCII only - U+25B8 had no font glyph.
            format!("> {}  [{}]", name, cell.0 + 1)
        } else {
            format!("  {}  [{}]", name, cell.0 + 1)
        };
        let a = if active { 1.0 } else { 0.40 } * strip_fade;
        *tc = TextColor(Color::srgba(0.92, 0.93, 0.95, a));
    }
}

/// §7: the stance/stability bracket around the crosshair - widens with
/// the CURRENT spread (bloom + movement + stance), so the player watches
/// their own accuracy in real time. 0 = left, 1 = right.
#[derive(Component)]
struct StabilityBracket(u8);

#[derive(Component)]
struct PanelInfoText;

#[derive(Component)]
struct PanelAmmoText;

#[derive(Component)]
struct ScoreboardRoot;

#[derive(Component)]
struct ScoreboardText;

/// 0 top, 1 right, 2 bottom, 3 left - damage-direction flash strips.
#[derive(Component)]
struct DmgEdge(u8);

// (§7 Brief III: the old OwnHpFill/OwnArmorFill bars are gone - numerals only.)

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Intro,
    Playing,
    Paused,
    Settings,
    Manual,
    /// §1.2 (Brief III): the controls screen, generated from the registry.
    Controls,
}

// ---- §1.2 (Brief III): the keybind registry ------------------------------
// ONE table owns every action's bind, display name, and grouping. The
// controls screen and the first-run card are GENERATED from it, so they
// can never drift from reality. Every new action registers here.

struct Bind {
    key: &'static str,
    action: &'static str,
    /// Shown on the one-time first-run card (the non-obvious binds).
    essential: bool,
}

const BIND_REGISTRY: &[Bind] = &[
    Bind { key: "W A S D", action: "Move", essential: false },
    Bind { key: "MOUSE", action: "Look", essential: false },
    Bind { key: "LMB", action: "Fire", essential: false },
    Bind { key: "RMB", action: "Alt: scope zoom (heavy rifle) / draw (bow, spear) - rifles have NO aim-down-sights", essential: false },
    Bind { key: "T", action: "Inspect weapon", essential: false },
    Bind { key: "SHIFT", action: "Sprint", essential: false },
    // §4.3 (Brief VI): mech FLIGHT IS DELETED in the sim - the chassis
    // never leaves the ground. These strings promised thrusters the
    // simulation has not had for two briefs.
    Bind { key: "SPACE", action: "Jump", essential: true },
    Bind { key: "CTRL", action: "Crouch", essential: false },
    Bind { key: "Q", action: "Ground: dodge roll - Air + direction: FLIP (no firing)", essential: true },
    Bind { key: "V or O", action: "Camera: first <-> third person", essential: true },
    Bind { key: "E", action: "Shield stance (throwables only while up)", essential: true },
    Bind { key: "F", action: "Knife - tap: slash, hold: lunge (backstab kills)", essential: true },
    Bind { key: "C (hold)", action: "Armor ability (brace / flame / repulsor)", essential: true },
    Bind { key: "G", action: "Cycle throwable (frag/flash/smoke/molotov)", essential: true },
    Bind { key: "H / Mouse4", action: "HOLD: aim throw (arc previews, power charges) - release: throw", essential: true },
    Bind { key: "B", action: "Cancel aimed throw (keeps the grenade)", essential: false },
    Bind { key: "U", action: "Dismount the mech (chassis is scrapped; the pad respawns)", essential: false },
    Bind { key: "Y (hold)", action: "Mech missile pod: hold to LOCK a mech (1.3s), release to fire - tap: dumb-fire. Never locks infantry", essential: false },
    Bind { key: "1 2 3", action: "Weapon slots", essential: false },
    Bind { key: "R", action: "Reload", essential: false },
    Bind { key: "Z / X", action: "Lean left / right", essential: false },
    Bind { key: "TAB", action: "Scoreboard", essential: false },
    Bind { key: "M", action: "Minimap on/off", essential: false },
    Bind { key: "F3", action: "Hit-zone debug rings", essential: false },
    Bind { key: "F4", action: "Rig joint markers (gap view)", essential: false },
    Bind { key: "ESC", action: "Menu", essential: false },
];

/// §1.2: armor pads must announce themselves - the sets were shipped and
/// nobody could find them.
fn pickup_prompt(kind: PickupKind) -> &'static str {
    match kind {
        PickupKind::Health => "HEALTH PACK",
        PickupKind::Ammo => "AMMO CACHE",
        PickupKind::RobotArmor => "MECH CHASSIS - walk over to board  (Q: side-step, C: repulsor - armored front, soft rear)",
        PickupKind::FolkArmor => "FOLK ARMOR - walk over to equip  (hold C: shieldwall brace)",
        PickupKind::PyroArmor => "PYRO ARMOR - walk over to equip  (hold C: flame projector)",
        PickupKind::ReconWeave => "RECON WEAVE - walk over to equip  (fast, quiet, self-healing)",
        PickupKind::Minigun => "M134 MINIGUN - walk over to heft it  (hold fire: spin-up - R: vent heat - lost on death)",
    }
}

/// The 4-second ability hint shown the moment a set is equipped.
fn equip_hint(set: ArmorSet) -> &'static str {
    match set {
        ArmorSet::None => "",
        ArmorSet::Folk => "FOLK ARMOR EQUIPPED - hold C to BRACE the shieldwall",
        ArmorSet::Pyro => "PYRO ARMOR EQUIPPED - hold C: FLAME PROJECTOR - fireproof",
        ArmorSet::RobotSuit => "MECH BOARDED - Q: SIDE-STEP - C: REPULSOR - protect your REAR",
        ArmorSet::Recon => "RECON WEAVE EQUIPPED - faster, silent, regenerates",
    }
}

/// §1.2 first-run card: shown once on the first spawn, dismissed by any
/// key - the build finally teaches its own controls.
#[derive(Resource)]
struct FirstRunCard {
    shown: bool,
    dismissed: bool,
}

impl Default for FirstRunCard {
    fn default() -> Self {
        FirstRunCard {
            shown: false,
            dismissed: false,
        }
    }
}

#[derive(Component)]
struct FirstRunRoot;

#[derive(Component)]
struct ControlsRoot;

/// Bottom-center contextual prompt ("[pad] - walk over to equip").
#[derive(Component)]
struct PromptText;

/// A short on-screen confirmation ("FIRST PERSON") - actions the player
/// can't otherwise verify announce themselves here.
#[derive(Resource, Default)]
struct Toast {
    text: String,
    t: f32,
}

// ---- §0 (Brief VII): the capture helper -----------------------------------
// A "player" who can only issue tool calls has no hands on a keyboard to
// verify a claim like "the recoil pattern is visible now". This drives a
// SCRIPTED PlayerCmd-equivalent through the real client (real render, real
// HUD, real viewmodel) and snapshots it to disk, so a claim can be checked
// against a PNG instead of taken on faith. Set env var JK_CAPTURE=<script>
// to activate; the process exits itself once the script finishes.
#[derive(Resource, Default)]
struct CaptureMode {
    script: Option<String>,
    t: f32,
    cursor: usize,
    /// Snaps that BECAME DUE this frame, queued by the input driver and
    /// drained exactly once by the screenshot driver.
    ///
    /// The two drivers used to communicate through `cursor - 1`, which
    /// was wrong in both directions: the input driver advances through
    /// EVERY beat whose time has passed (a long frame can pass several),
    /// so all but the last had their snap silently skipped - and the
    /// screenshot driver re-read that same index every frame, so the
    /// surviving snap was written over and over (one 4-second run wrote
    /// its files 157 times). A queue makes each beat fire exactly once.
    pending_snaps: Vec<&'static str>,
    pending_end: bool,
    /// Frames to wait after the final beat before exiting, so queued
    /// screenshots reach disk.
    exit_in: u32,
}

enum CapKey {
    K(KeyCode),
    M(MouseButton),
}

/// One scripted beat: at `t` seconds into the match, hold/release keys,
/// snap the camera to a yaw/pitch, save a screenshot, or end the capture.
struct CapBeat {
    t: f32,
    press: &'static [CapKey],
    release: &'static [CapKey],
    look: Option<(f32, f32)>,
    snap: Option<&'static str>,
    end: bool,
}
const NO_KEYS: &[CapKey] = &[];
const fn beat(t: f32) -> CapBeat {
    CapBeat { t, press: NO_KEYS, release: NO_KEYS, look: None, snap: None, end: false }
}

// §0: prove third-person-by-default, then what first person actually
// looks like at rest, aiming, and mid-spray. Top-level `const` - a
// `..beat(t)` struct update inside a match-arm array literal isn't
// reliably 'static-promotable; a real const item always is.
const BASELINE_BEATS: &[CapBeat] = &[
    CapBeat { snap: Some("01-third-person-default"), ..beat(0.8) },
    CapBeat { press: &[CapKey::K(KeyCode::KeyV)], ..beat(0.9) },
    CapBeat { snap: Some("02-first-person-rest"), ..beat(1.4) },
    CapBeat {
        press: &[CapKey::M(MouseButton::Left)],
        snap: Some("03-first-person-fire-start"),
        ..beat(1.6)
    },
    CapBeat { snap: Some("04-first-person-mid-spray"), ..beat(2.4) },
    CapBeat {
        release: &[CapKey::M(MouseButton::Left)],
        snap: Some("05-first-person-recovered"),
        ..beat(3.2)
    },
    CapBeat { end: true, ..beat(3.6) },
];

/// The scripted beat table per capture script - the closest thing this
/// project has to a recorded playtest.
// §1.4 (Brief VII): the idle-life capture - stationary, out of combat, no
// input at all, for 16s. Several stills across the window are the closest
// thing this screenshot-only harness can give to "a clip": if breathing/
// weight-shift/head-glance are working, consecutive stills won't be
// identical poses.
const IDLE_LIFE_BEATS: &[CapBeat] = &[
    CapBeat { snap: Some("00s"), ..beat(1.0) },
    CapBeat { snap: Some("04s"), ..beat(4.0) },
    CapBeat { snap: Some("08s"), ..beat(8.0) },
    CapBeat { snap: Some("12s"), ..beat(12.0) },
    CapBeat { snap: Some("16s"), ..beat(16.0) },
    CapBeat { end: true, ..beat(16.5) },
];

// §4 (Brief VII v2): bow draw -> hold -> sway onset -> release. Switches
// to the bow, then holds fire for ~1s (past full draw) before releasing.
const BOW_DRAW_BEATS: &[CapBeat] = &[
    CapBeat { press: &[CapKey::K(KeyCode::Digit3)], ..beat(0.6) },
    CapBeat { snap: Some("01-bow-equipped"), ..beat(1.0) },
    CapBeat { press: &[CapKey::M(MouseButton::Left)], ..beat(1.1) },
    CapBeat { snap: Some("02-bow-draw-start"), ..beat(1.3) },
    CapBeat { snap: Some("03-bow-full-draw"), ..beat(1.9) },
    CapBeat {
        release: &[CapKey::M(MouseButton::Left)],
        snap: Some("04-bow-release"),
        ..beat(2.0)
    },
    CapBeat { snap: Some("05-bow-after-shot"), ..beat(2.4) },
    CapBeat { end: true, ..beat(2.8) },
];

// Task 5.7 (MISSION doc): the mech at its new scale/palette, held
// stationary at a known-clear spot (Arena center, set in
// capture_quick_deploy) with the camera aimed level and slightly down -
// no WASD movement, so unknown nearby cover can't confound the shot.
const MECH_CAPTURE_BEATS: &[CapBeat] = &[
    CapBeat { look: Some((0.0, 0.15)), ..beat(0.3) },
    CapBeat { snap: Some("01-mech-third-person"), ..beat(1.0) },
    CapBeat { look: Some((1.2, 0.15)), ..beat(1.2) },
    CapBeat { snap: Some("02-mech-side-on"), ..beat(2.0) },
    // Task 5.4 verification: pitch steeply down to bring the knee/waist
    // exposed-mechanism plates (added this pass) into frame - the
    // standing eye-level shots above crop them below the HUD.
    CapBeat { look: Some((0.5, 0.55)), ..beat(2.2) },
    CapBeat { snap: Some("03-mech-knee-waist-detail"), ..beat(3.0) },
    CapBeat { end: true, ..beat(3.4) },
];

// §7 audit: hold the trigger through spin-up, sustained fire (barrels
// full speed, heat climbing), and release - all from a fixed, known-
// clear spot so barrel spin / tracers / heat-driven spread are all
// checkable against a real capture rather than taken on faith.
const MINIGUN_CHECK_BEATS: &[CapBeat] = &[
    CapBeat { look: Some((0.0, 0.08)), ..beat(0.2) },
    CapBeat { press: &[CapKey::M(MouseButton::Left)], ..beat(0.3) },
    CapBeat { snap: Some("01-minigun-spinup"), ..beat(0.55) },
    CapBeat { snap: Some("02-minigun-sustained-fire"), ..beat(1.6) },
    CapBeat { snap: Some("03-minigun-hot"), ..beat(3.6) },
    CapBeat {
        release: &[CapKey::M(MouseButton::Left)],
        snap: Some("04-minigun-release"),
        ..beat(4.0)
    },
    CapBeat { snap: Some("05-minigun-spindown"), ..beat(4.5) },
    CapBeat { end: true, ..beat(4.9) },
];

// Task 0 before-clip (c): every traversal move that exists - jump,
// dodge roll (W held so it launches forward), and the air-flip (dodge
// while airborne). Third person so the whole body reads.
const TRAVERSAL_BEATS: &[CapBeat] = &[
    CapBeat { look: Some((0.0, 0.10)), ..beat(0.2) },
    CapBeat { press: &[CapKey::K(KeyCode::KeyW)], ..beat(0.3) },
    CapBeat { press: &[CapKey::K(KeyCode::Space)], ..beat(0.8) },
    CapBeat { snap: Some("01-jump-apex"), ..beat(1.0) },
    CapBeat { release: &[CapKey::K(KeyCode::Space)], ..beat(1.1) },
    CapBeat { press: &[CapKey::K(KeyCode::KeyQ)], ..beat(1.8) },
    CapBeat { snap: Some("02-roll-mid"), ..beat(2.05) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyQ)], ..beat(2.2) },
    // airborne flip: jump, then dodge mid-air
    CapBeat { press: &[CapKey::K(KeyCode::Space)], ..beat(3.2) },
    CapBeat { release: &[CapKey::K(KeyCode::Space)], ..beat(3.35) },
    CapBeat { press: &[CapKey::K(KeyCode::KeyQ)], ..beat(3.45) },
    CapBeat { snap: Some("03-air-flip"), ..beat(3.6) },
    CapBeat {
        release: &[CapKey::K(KeyCode::KeyQ), CapKey::K(KeyCode::KeyW)],
        snap: Some("04-landing-recovery"),
        ..beat(4.1)
    },
    CapBeat { end: true, ..beat(4.6) },
];

// Task 0 before-clip (e): one lap of the map's elevation story - four
// compass looks from the stage point, then two sprint legs with the
// camera held on the horizon so plateaus/stairs/towers cross frame.
const MAP_LAP_BEATS: &[CapBeat] = &[
    CapBeat { look: Some((0.0, 0.05)), snap: Some("01-north"), ..beat(0.6) },
    CapBeat { look: Some((1.5708, 0.05)), snap: Some("02-east"), ..beat(1.2) },
    CapBeat { look: Some((3.1416, 0.05)), snap: Some("03-south"), ..beat(1.8) },
    CapBeat { look: Some((4.7124, 0.05)), snap: Some("04-west"), ..beat(2.4) },
    CapBeat {
        look: Some((0.7854, 0.02)),
        press: &[CapKey::K(KeyCode::KeyW), CapKey::K(KeyCode::ShiftLeft)],
        ..beat(2.8)
    },
    CapBeat { snap: Some("05-sprint-leg-1"), ..beat(4.2) },
    CapBeat { look: Some((2.3562, 0.02)), ..beat(4.4) },
    CapBeat { snap: Some("06-sprint-leg-2"), ..beat(5.8) },
    CapBeat {
        release: &[CapKey::K(KeyCode::KeyW), CapKey::K(KeyCode::ShiftLeft)],
        snap: Some("07-lap-end"),
        ..beat(6.2)
    },
    CapBeat { end: true, ..beat(6.7) },
];

fn capture_script(name: &str) -> &'static [CapBeat] {
    match name {
        "baseline" => BASELINE_BEATS,
        "idle_life" => IDLE_LIFE_BEATS,
        "bow_draw" => BOW_DRAW_BEATS,
        "mech_scale" => MECH_CAPTURE_BEATS,
        "minigun_check" => MINIGUN_CHECK_BEATS,
        "traversal" => TRAVERSAL_BEATS,
        "map_lap" => MAP_LAP_BEATS,
        _ => &[],
    }
}

/// Populated once at Startup from `JK_CAPTURE`; if unset, every capture
/// system below is a no-op and the game behaves exactly as launched by a
/// human.
const CAPTURE_SCRIPTS: [&str; 8] = [
    "baseline",
    "idle_life",
    "bow_draw",
    "mech_scale",
    "minigun_check",
    "menus",
    "traversal",
    "map_lap",
];

fn init_capture_mode(mut cap: ResMut<CaptureMode>) {
    if let Ok(script) = std::env::var("JK_CAPTURE") {
        // A name with no beat table produced an EMPTY script, which meant
        // no beat ever fired, no snap was taken, and the `end` that exits
        // the process never came - the run just hung forever with a
        // window open, looking like a slow capture rather than a typo.
        if !CAPTURE_SCRIPTS.contains(&script.as_str()) {
            eprintln!(
                "JK_CAPTURE={script:?} is not a known capture script.\nValid scripts: {}",
                CAPTURE_SCRIPTS.join(", ")
            );
            std::process::exit(2);
        }
        cap.script = Some(script);
    }
}

/// Intro-state: skip the menu entirely and deploy straight into a match -
/// there is no way to synthesize a Bevy UI button click cheaply, so the
/// capture harness starts the match directly instead.
fn capture_quick_deploy(
    cap: Res<CaptureMode>,
    mut started: Local<bool>,
    mut sel: ResMut<Selected>,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<GameState>>,
    mut card: ResMut<FirstRunCard>,
) {
    if *started || cap.script.is_none() {
        return;
    }
    // "menus" stays in Intro on purpose - it is capturing the loadout
    // and settings SCREENS, which never enter Playing at all, so the
    // Playing-gated drivers below can never see them.
    if cap.script.as_deref() == Some("menus") {
        return;
    }
    *started = true;
    // The §1.2 first-run "GOOD TO KNOW" card is dismissed by any keypress,
    // but scripts like mech_scale only synthesize `look`, never a key -
    // so it sat on screen through every snap. Capture scripts test game
    // mechanics, not tutorial UX; skip spawning it entirely instead of
    // relying on some other beat's keypress to incidentally clear it.
    card.shown = true;
    card.dismissed = true;
    match cap.script.as_deref() {
        // (spear_throw / bow_pierce arms lived here with no beat table
        // behind them, so naming either just hung the process. Validated
        // against CAPTURE_SCRIPTS at startup now.)
        Some("bow_draw") => sel.loadout[2] = GunKind::Bow,
        _ => {}
    }
    start_match(&sel, Mode::Tdm, &mut game, &mut next);
    if cap.script.as_deref() == Some("mech_scale") {
        // Task 5.7: board the mech directly - no need to walk to a pad
        // just to prove the scale/palette read. Also plant it at a KNOWN
        // clear spot (Arena center) - the default spawn's proximity to
        // cover was a confound in earlier capture attempts, independent
        // of the camera-scaling fix itself.
        let stage = capture_stage_pos(&game.sim);
        let f = &mut game.sim.fighters[0];
        f.armor_set = ArmorSet::RobotSuit;
        f.armor = POWER_MAX;
        f.hull = MECH_HULL;
        f.mech_transition_t = 0.0; // skip the seal-up window for the capture
        f.pos = stage;
        f.yaw = 0.0;
    }
    if cap.script.as_deref() == Some("minigun_check") {
        // §7 audit: minigun is pickup-only (no `Selected.loadout` slot),
        // so hand it over the same way a pad pickup does (mirrors
        // `PickupKind::Minigun` in sim.rs exactly) rather than faking a
        // loadout path that doesn't exist for this weapon.
        let stage = capture_stage_pos(&game.sim);
        let f = &mut game.sim.fighters[0];
        f.inventory[0] = GunKind::Minigun;
        f.slot_ammo[0] = (gun(GunKind::Minigun).mag, 0);
        f.active = 0;
        f.gun = GunKind::Minigun;
        f.ammo = f.slot_ammo[0].0;
        f.reserve = 0;
        f.reload_t = 0.0;
        f.pos = stage;
        f.yaw = 0.0;
    }
    // Task 0 before-clips: both need a clear stage so the moves and the
    // lap read on camera instead of clipping into whatever cover the
    // default spawn abuts.
    if matches!(cap.script.as_deref(), Some("traversal") | Some("map_lap")) {
        let stage = capture_stage_pos(&game.sim);
        let f = &mut game.sim.fighters[0];
        f.pos = stage;
        f.yaw = 0.0;
    }
}

/// A spot on the CURRENT map that is provably outside every cover block,
/// for capture scripts that need a clean, unobstructed stage.
///
/// The two scripts that needed one both hard-coded `[0,0,0]` and called
/// it "a KNOWN clear spot (Arena center)". It is not: Arena builds a
/// central 8x8x3 m stone block spanning [-4,-4] to [4,4], so [0,0,0] is
/// its exact centroid - every mech and minigun capture was staged INSIDE
/// a rock, relying on the collision push-out to shove it somewhere
/// arbitrary. Search a ring of candidates and take the first that is
/// genuinely clear, so this cannot silently rot when a map changes.
fn capture_stage_pos(sim: &TdmSim) -> [f32; 3] {
    let clear = |x: f32, z: f32| {
        const PAD: f32 = 1.5; // room for a 3m mech's radius
        !sim.cover.iter().any(|a| {
            x >= a.min[0] - PAD
                && x <= a.max[0] + PAD
                && z >= a.min[2] - PAD
                && z <= a.max[2] + PAD
        })
    };
    for ring in [12.0_f32, 16.0, 20.0, 8.0, 24.0] {
        for k in 0..12 {
            let ang = k as f32 / 12.0 * std::f32::consts::TAU;
            let (x, z) = (ang.cos() * ring, ang.sin() * ring);
            if x.abs() < sim.half - 3.0 && z.abs() < sim.half - 3.0 && clear(x, z) {
                return [x, 0.0, z];
            }
        }
    }
    [0.0, 0.0, 0.0] // nothing clear found - better than looping forever
}

/// Scripts that capture a WEAPON being fired for several seconds in the
/// open. Firing deliberately clears spawn protection (sim.rs, the
/// `protect_t = 0.0` in the fire path), so a subject holding the trigger
/// in a live match is a legitimate target and simply dies mid-script -
/// which is correct game behavior, but destroys the capture. These
/// scripts pin the subject's health so the weapon-feel frames actually
/// get taken. Capture-harness only: inert without `JK_CAPTURE`, and it
/// never runs for a human-launched game.
const CAPTURE_KEEP_ALIVE: [&str; 3] = ["minigun_check", "traversal", "map_lap"];

fn capture_keep_subject_alive(cap: Res<CaptureMode>, mut game: ResMut<Game>) {
    let Some(name) = cap.script.as_deref() else { return };
    if !CAPTURE_KEEP_ALIVE.contains(&name) {
        return;
    }
    let p = game.sim.player;
    if let Some(f) = game.sim.fighters.get_mut(p) {
        // `alive()` is `respawn_t <= 0.0 && health > 0.0` - restoring
        // health alone leaves the death already latched, so the subject
        // still reads as DOWN. Clearing respawn_t directly (rather than
        // letting it tick down) also skips the respawn reposition, so
        // the subject stays planted where the script put it.
        f.health = MAX_HEALTH;
        f.respawn_t = 0.0;
    }
}

/// Playing-state, runs BEFORE `input_and_step`: synthesizes exactly the
/// key/mouse-button holds a human would produce, and sets camera yaw/pitch
/// directly (there is no cheap way to fake a MouseMotion delta that lands
/// on an exact aim angle). `input_and_step` is untouched - it just reads
/// these same `ButtonInput` resources like it would for a real player.
fn capture_input_driver(
    mut cap: ResMut<CaptureMode>,
    time: Res<Time>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    mut cam: ResMut<CamCtl>,
) {
    let Some(name) = cap.script.clone() else { return };
    cap.t += time.delta_secs();
    let script = capture_script(&name);
    while cap.cursor < script.len() && script[cap.cursor].t <= cap.t {
        let b = &script[cap.cursor];
        for k in b.press {
            match k {
                CapKey::K(k) => keys.press(*k),
                CapKey::M(m) => buttons.press(*m),
            }
        }
        for k in b.release {
            match k {
                CapKey::K(k) => keys.release(*k),
                CapKey::M(m) => buttons.release(*m),
            }
        }
        if let Some((yaw, pitch)) = b.look {
            cam.yaw = yaw;
            cam.pitch = pitch;
        }
        // queue this beat's snap/end for the screenshot driver instead of
        // letting it infer them from the cursor - a frame that passes
        // several beats must fire ALL their snaps, not just the last
        let (snap, end) = (b.snap, b.end);
        cap.cursor += 1;
        if let Some(label) = snap {
            cap.pending_snaps.push(label);
        }
        if end {
            cap.pending_end = true;
            cap.exit_in = 12; // frames of flush grace
        }
    }
}

/// Fires the snapshot(s) due this frame and, on `end`, exits the process -
/// there is no human to close the window once the script is done.
fn capture_screenshot_driver(
    mut cap: ResMut<CaptureMode>,
    mut shots: ResMut<Assets<Image>>,
    mut commands: Commands,
    window: Query<Entity, With<PrimaryWindow>>,
) {
    let Some(name) = cap.script.clone() else { return };
    // drain the queue: every beat that came due fires EXACTLY once, and
    // a frame that passed several fires all of them
    let due: Vec<&'static str> = cap.pending_snaps.drain(..).collect();
    for label in due {
        let dir = format!("handback/brief-vii/{name}");
        let _ = std::fs::create_dir_all(&dir);
        let path = format!("{dir}/{label}.png");
        if let Ok(win) = window.get_single() {
            commands
                .spawn(Screenshot::window(win))
                .observe(bevy::render::view::screenshot::save_to_disk(path));
        }
    }
    if cap.pending_end {
        // Grace frames before exiting: `save_to_disk` is an observer that
        // runs after the render, so exiting the same frame the last snap
        // is queued would lose it. The old code got away with this only
        // because it re-fired the same snap every frame until exit.
        if cap.exit_in > 0 {
            cap.exit_in -= 1;
        } else {
            std::process::exit(0);
        }
    }
    let _ = &mut shots; // reserved: kept for API-version fallback
}

/// The loadout (Intro) and Settings screens never enter `Playing`, so the
/// Playing-gated drivers above can never photograph them - which meant UI
/// work was the one part of this codebase exempt from "visible or it
/// didn't happen". This walks Intro -> Settings on a timer and snaps
/// each. Inert unless `JK_CAPTURE=menus`.
fn capture_menus(
    cap: Res<CaptureMode>,
    time: Res<Time>,
    mut commands: Commands,
    mut t: Local<f32>,
    mut stage: Local<usize>,
    mut next: ResMut<NextState<GameState>>,
    window: Query<Entity, With<PrimaryWindow>>,
) {
    if cap.script.as_deref() != Some("menus") {
        return;
    }
    *t += time.delta_secs();
    let snap = |commands: &mut Commands, label: &str| {
        let dir = "handback/brief-vii/menus".to_string();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(win) = window.get_single() {
            commands
                .spawn(Screenshot::window(win))
                .observe(bevy::render::view::screenshot::save_to_disk(format!(
                    "{dir}/{label}.png"
                )));
        }
    };
    match *stage {
        0 if *t > 1.2 => {
            snap(&mut commands, "01-loadout-intro");
            *stage = 1;
        }
        1 if *t > 1.8 => {
            next.set(GameState::Settings);
            *stage = 2;
        }
        2 if *t > 2.8 => {
            snap(&mut commands, "02-settings");
            *stage = 3;
        }
        3 if *t > 3.6 => std::process::exit(0),
        _ => {}
    }
}

/// The full-screen low-health tint overlay. §3.7 (Brief VI) turned this
/// OFF - it is held fully transparent by `health_vignette`, and danger
/// reads through the vitals colour instead. Kept as an entity so the
/// decision lives in one place. (Its doc used to describe a pulsing
/// regen tint as though it were live; it has not been since Brief VI.)
#[derive(Component)]
struct HealthVignette;

/// The lobby's toast line. The Forge (Ctrl+1/2/3 save, 1/2/3 load) runs
/// ONLY in the Intro state and its sole feedback is the shared `Toast`
/// resource - but the only system that rendered or decayed a toast was
/// `contextual_prompts`, which is Playing-gated. So every Forge
/// confirmation was invisible where the Forge lives, AND never decayed,
/// so it surfaced stale on the first frame of the next match instead.
#[derive(Component)]
struct LobbyToast;

fn lobby_toast(
    time: Res<Time>,
    mut toast: ResMut<Toast>,
    mut q: Query<&mut Text, With<LobbyToast>>,
) {
    let Ok(mut t) = q.get_single_mut() else {
        return;
    };
    if toast.t > 0.0 {
        toast.t -= time.delta_secs();
        **t = toast.text.clone();
    } else {
        **t = String::new();
    }
}

/// §14: the loadout tech readout - real numbers from the live weapon
/// table. "spear - 26 m/s, 4.70 m drop at 30 m" tells the player the
/// game's premise before they press Play.
#[derive(Component)]
struct TechReadout;

fn tech_readout(sel: Res<Selected>, mut q: Query<&mut Text, With<TechReadout>>) {
    let Ok(mut t) = q.get_single_mut() else {
        return;
    };
    let mut s = String::from("- LOADOUT SPEC -\n");
    for g in sel.loadout.iter() {
        let spec = gun(*g);
        s += &match spec.projectile {
            Some((v0, _)) => {
                let g_eff = missile_g(*g == GunKind::Spear);
                let t30 = 30.0 / v0;
                let drop = 0.5 * g_eff * t30 * t30;
                format!(
                    "{:<14} {:>3.0} m/s   drop {:.2} m @30 m   dmg {:.0}\n",
                    spec.name, v0, drop, spec.damage
                )
            }
            None => format!(
                "{:<14} hitscan   {:>4.0} rpm   dmg {:.1}   mag {}\n",
                spec.name,
                60.0 / spec.fire_period,
                spec.damage,
                spec.mag
            ),
        };
    }
    **t = s;
}

#[derive(Component)]
struct IntroRoot;

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct SettingsRoot;

#[derive(Component)]
struct ManualRoot;

#[derive(Component, Clone, Copy)]
enum ModeButton {
    Tdm,
    Koth,
    /// §8: co-op zombie extraction.
    Extraction,
}

#[derive(Component, Clone, Copy)]
struct MapButton(MapKind);

#[derive(Component, Clone, Copy)]
struct DiffButton(Difficulty);

#[derive(Component, Clone, Copy)]
struct SizeButton(usize);

/// Loadout pick: (slot index, weapon).
#[derive(Component, Clone, Copy)]
struct LoadoutButton(usize, GunKind);

/// Cosmetics: (0 = hat, 1 = tunic; choice index).
#[derive(Component, Clone, Copy)]
struct CosmeticButton(usize, usize);

/// §6 (Brief IV): melee slot pick - false = knife, true = axe.
#[derive(Component, Clone, Copy)]
struct MeleeButton(bool);

/// §8 (Brief IV): grenade budget preset pick (GRENADE_PRESETS index).
#[derive(Component, Clone, Copy)]
struct NadeButton(usize);

#[derive(Component, Clone, Copy)]
enum MenuButton {
    Resume,
    Restart,
    Settings,
    Manual,
    /// §1.2: the registry-generated controls screen.
    Controls,
    BackToLoadout,
    Quit,
}

#[derive(Component, Clone, Copy)]
enum SettingsButton {
    SwapMouse,
    Minimap,
    Sens,
    Fov,
    InvertY,
    Back,
}

/// Live text labels on the settings page.
#[derive(Component, Clone, Copy)]
struct SettingsLabel(SettingsButtonKind);

#[derive(Clone, Copy, PartialEq)]
enum SettingsButtonKind {
    SwapMouse,
    Minimap,
    Sens,
    Fov,
    InvertY,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "John Kingdom Game - Arena".into(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // shared workspace assets dir (engine/assets), whether
                    // launched via cargo or the raw target/release binary
                    file_path: "../../assets".into(),
                    ..default()
                }),
        )
        // The key art / wordmark / emblem. Self-contained: owns its own
        // splash state, systems and teardown, and skips itself entirely
        // when JK_CAPTURE is set so it never lands in a scripted capture.
        .add_plugins(branding::BrandingPlugin)
        .insert_resource(ClearColor(Color::srgb(0.58, 0.63, 0.72)))
        .init_resource::<CamCtl>()
        // R4: loaded once here, held fixed for the whole run - same
        // lifetime as the consts it can override.
        .insert_resource(load_camera_tuning())
        .init_state::<GameState>()
        // §0 (Brief VII): the capture helper - inert unless JK_CAPTURE is set
        .init_resource::<CaptureMode>()
        .add_systems(Startup, init_capture_mode)
        .add_systems(Update, capture_quick_deploy.run_if(in_state(GameState::Intro)))
        // menu capture runs in the MENU states, not Playing
        .add_systems(Update, capture_menus)
        // PreUpdate, not Update: a synthetic `.press()` only sets
        // just_pressed for one frame, same as real OS input — but real
        // input arrives in PreUpdate (guaranteed before every Update
        // system), where a synthetic press in Update would race other
        // Update systems for that single frame non-deterministically.
        .add_systems(
            PreUpdate,
            (
                capture_keep_subject_alive,
                capture_input_driver,
                capture_screenshot_driver,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, input_and_step.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            (
                rebuild_world,
                sync_fighters,
                sync_tracers,
                sync_missiles,
                sync_dropped,
                sync_throwables,
                sync_zombies,
                sync_decals,
                sync_pickups,
                camera_system,
                fp_viewmodel,
                arc_preview,
                sync_health_bars,
                mech_stage_presentation,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                hud_system,
                hud_fade,
                scoreboard_system,
                damage_indicator,
                sfx_system,
                distant_gunfire,
                scope_overlay,
                ads_detail,
                checkpoint_rings,
                minimap_system,
                zone_overlay,
                tag_viewmodel_layer,
                compass_system,
                stability_bracket,
                health_vignette,
                weapon_strip,
            ),
        )
        .init_resource::<DebugZones>()
        .init_resource::<DistantShots>()
        .init_resource::<Toast>()
        .add_systems(Update, esc_toggle)
        .add_systems(OnEnter(GameState::Playing), grab_cursor)
        .add_systems(OnEnter(GameState::Intro), open_intro)
        .add_systems(OnExit(GameState::Intro), close_intro)
        .add_systems(
            Update,
            (
                intro_buttons,
                intro_map_buttons,
                intro_loadout_buttons,
                intro_cosmetic_buttons,
                intro_melee_buttons,
                intro_nade_buttons,
                intro_diff_buttons,
                intro_size_buttons,
                tech_readout, // §14: live weapon numbers on the loadout
                forge_keybinds, // §7.2 (Brief VII v2): Ctrl+1/2/3 save, 1/2/3 load
                lobby_toast,    // ...and where its confirmations appear
            )
                .run_if(in_state(GameState::Intro)),
        )
        .add_systems(OnEnter(GameState::Paused), open_menu)
        .add_systems(OnExit(GameState::Paused), close_menu)
        .add_systems(Update, menu_buttons.run_if(in_state(GameState::Paused)))
        .add_systems(OnEnter(GameState::Settings), open_settings)
        .add_systems(OnExit(GameState::Settings), close_settings)
        .add_systems(Update, settings_buttons.run_if(in_state(GameState::Settings)))
        // state-agnostic: the M minimap hotkey mutates settings during
        // Playing too, and that change must survive a restart as well
        .add_systems(Update, persist_settings)
        .add_systems(OnEnter(GameState::Manual), open_manual)
        .add_systems(OnExit(GameState::Manual), close_manual)
        // §1.2 (Brief III): discoverability - the controls screen, the
        // first-run card, and contextual pickup/equip prompts
        .init_resource::<FirstRunCard>()
        .add_systems(OnEnter(GameState::Controls), open_controls)
        .add_systems(OnExit(GameState::Controls), close_controls)
        .add_systems(
            Update,
            (first_run_card, contextual_prompts).run_if(in_state(GameState::Playing)),
        )
        // §12 (Brief IV): ejected shell casings - pooled, physical, brief
        .add_systems(
            Update,
            (
                spawn_casings,
                update_casings,
                spin_minigun_barrels,
                grenade_arc,
                crosshair_kill_pop,
                ammo_bar_sync,
                hud_colors,
                sync_rockets,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

/// §1 (Brief V): the grenade pre-aim arc. While the throw is held this
/// calls the sim's OWN `throw_release_velocity` + `predict_grenade` -
/// never a reimplementation - so the dots trace exactly the flight the
/// grenade will take, live, as the camera moves and the power charges.
/// Dots after the first bounce render faint: less certain, by design.
fn grenade_arc(
    game: Res<Game>,
    arc: Res<GrenadeArcVis>,
    cam_q: Query<&Transform, With<MainCam>>,
    mut q: Query<(&mut Transform, &mut Visibility), Without<MainCam>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let show = p.alive() && p.cook_t > 0.0;
    if !show {
        for e in arc.pre.iter().chain(arc.post.iter()).chain([&arc.ring]) {
            if let Ok((_, mut v)) = q.get_mut(*e) {
                *v = Visibility::Hidden;
            }
        }
        return;
    }
    let Ok(cam_tf) = cam_q.get_single() else {
        return;
    };
    let (d, _) = crosshair_aim_dir(&game.sim, cam_tf);
    let kind = ThrowKind::ALL[p.throw_sel as usize];
    let (o, vel) = game.sim.throw_release_velocity(
        game.sim.player,
        [d.x, d.y, d.z],
        p.cook_t,
    );
    let spec = throw_spec(kind);
    let fuse = if spec.fuse_s.is_finite() {
        (spec.fuse_s - if kind == ThrowKind::Frag { p.cook_t } else { 0.0 }).max(0.15)
    } else {
        f32::INFINITY
    };
    let (pts, end, first_bounce) = game.sim.predict_grenade(kind, o, vel, fuse, 8.0);
    let fb = first_bounce.unwrap_or(pts.len()).min(pts.len());
    let mut place = |ents: &[Entity], seg: &[[f32; 3]]| {
        for (i, e) in ents.iter().enumerate() {
            let Ok((mut t, mut v)) = q.get_mut(*e) else {
                continue;
            };
            if seg.len() < 2 {
                *v = Visibility::Hidden;
                continue;
            }
            let idx = (i * (seg.len() - 1)) / (ents.len() - 1).max(1);
            t.translation = Vec3::from_array(seg[idx]);
            *v = Visibility::Visible;
        }
    };
    place(&arc.pre, &pts[..fb]);
    place(&arc.post, &pts[fb..]);
    if let Ok((mut t, mut v)) = q.get_mut(arc.ring) {
        t.translation = Vec3::from_array(end) + Vec3::Y * 0.04;
        t.rotation = Quat::IDENTITY;
        *v = Visibility::Visible;
    }
}

/// §7: the viewmodel barrels turn with the sim's spin state - idle
/// crawl at rest, a blur at full spin, frozen mid-vent. All three states
/// are distinct on screen: the rest crawl is what stops "holding a
/// minigun" from looking identical to "the gun is seized mid-vent",
/// which is exactly what a purely spin_t-proportional rate produced
/// (spin_t is pinned to 0 at rest, so the cluster was bolt-still).
const MINIGUN_IDLE_CRAWL_RAD_S: f32 = 0.55;
const MINIGUN_SPIN_FULL_RAD_S: f32 = 46.0;

fn spin_minigun_barrels(
    game: Res<Game>,
    time: Res<Time>,
    mut q: Query<&mut Transform, With<MinigunSpinner>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let rate = if p.vent_t > 0.0 {
        0.0 // vent locks the cluster while it dumps heat
    } else {
        MINIGUN_IDLE_CRAWL_RAD_S
            + (p.spin_t / MINIGUN_SPINUP_S) * MINIGUN_SPIN_FULL_RAD_S
    };
    if rate <= 0.0 {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    for mut tf in &mut q {
        tf.rotation = Quat::from_rotation_z(rate * dt) * tf.rotation;
    }
}

// ------------------------------------------------------------- casings ---

/// §12: one ejected casing per shot - a tiny brass slug that arcs right,
/// bounces once, and fades from the world after 8 s. Render-side only.
#[derive(Component)]
struct Casing {
    vel: Vec3,
    spin: Vec3,
    ttl: f32,
    bounced: bool,
    /// world-Y the casing lands on - the SHOOTER's floor, so brass on a
    /// plateau doesn't sink to the arena floor below it
    floor_y: f32,
}

const CASING_CAP: usize = 96;
/// §5.1 (Brief VI): casings persist ≥ 10 s.
const CASING_TTL_S: f32 = 10.0;

/// Detect fresh shots by each fighter's fire_cd jumping UP, and eject a
/// casing from beside the action. Bows, spears, and fists leave none.
fn spawn_casings(
    mut commands: Commands,
    game: Res<Game>,
    kit: Res<ModelKit>,
    live: Query<(), With<Casing>>,
    mut prev_cd: Local<Vec<f32>>,
) {
    let simr = &game.sim;
    prev_cd.resize(simr.fighters.len(), 0.0);
    let mut budget = CASING_CAP.saturating_sub(live.iter().count());
    for (i, f) in simr.fighters.iter().enumerate() {
        let fresh = f.fire_cd > prev_cd[i] + 1e-6;
        prev_cd[i] = f.fire_cd;
        if !fresh || budget == 0 {
            continue;
        }
        if matches!(f.gun, GunKind::Bow | GunKind::Spear | GunKind::Fists) {
            continue;
        }
        budget -= 1;
        // eject to screen-right of the muzzle line
        let right = Vec3::new(-f.yaw.cos(), 0.0, f.yaw.sin());
        let fwd = Vec3::new(f.yaw.sin(), 0.0, f.yaw.cos());
        let at = Vec3::new(f.pos[0], f.pos[1] + 1.32, f.pos[2]) + right * 0.22 + fwd * 0.30;
        // small per-shot variety from a render-side hash (never sim RNG)
        let h = ((i as f32 * 12.9898 + simr.t * 78.233).sin() * 43758.55).fract();
        commands.spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.gold.clone()),
            Transform::from_translation(at)
                .with_scale(Vec3::new(0.014, 0.014, 0.034)),
            Casing {
                vel: right * (1.3 + h * 0.8) + Vec3::Y * (1.6 + h * 0.5) - fwd * 0.2,
                spin: Vec3::new(7.0 + h * 6.0, 5.0, 3.0 + h * 4.0),
                ttl: CASING_TTL_S,
                bounced: false,
                floor_y: f.pos[1] + 0.02,
            },
        ));
    }
}

fn update_casings(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Casing, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (e, mut c, mut tf) in &mut q {
        c.ttl -= dt;
        if c.ttl <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        if c.vel.length_squared() > 1e-4 {
            c.vel.y -= 9.8 * dt;
            let step = c.vel * dt;
            tf.translation += step;
            tf.rotation = Quat::from_scaled_axis(c.spin * dt) * tf.rotation;
            if tf.translation.y < c.floor_y {
                tf.translation.y = c.floor_y;
                if c.bounced {
                    // at rest: lie flat where it landed
                    c.vel = Vec3::ZERO;
                    c.spin = Vec3::ZERO;
                } else {
                    c.bounced = true;
                    c.vel.y = -c.vel.y * 0.35;
                    c.vel.x *= 0.55;
                    c.vel.z *= 0.55;
                    c.spin *= 0.5;
                }
            }
        }
    }
}

// ------------------------------------------------------------------ colors

/// Per-man hat color - no two robots dress alike; the player rides in white.
fn hat_color(slot: usize, is_player: bool) -> Color {
    if is_player {
        return Color::srgb(0.92, 0.90, 0.85);
    }
    const H: [(f32, f32, f32); 5] = [
        (0.38, 0.24, 0.12), // saddle brown
        (0.12, 0.11, 0.11), // black
        (0.72, 0.60, 0.40), // tan
        (0.30, 0.30, 0.33), // slate
        (0.45, 0.18, 0.12), // oxblood
    ];
    let (r, g, b) = H[slot % 5];
    Color::srgb(r, g, b)
}

// --------------------------------------------------------------- models ---

/// §2.1: the index-finger pivot on a first-person hand - the trigger
/// finger. `rest` is its rest curl; `fp_viewmodel` drives it to full
/// flex over 40 ms on fire, returning over 90 ms, LEADING the shot.
#[derive(Component)]
struct TriggerFinger {
    rest: f32,
}

/// §7: the viewmodel minigun's barrel-cluster pivot - spun by the sim's
/// spin_t (the player's own barrels; world models stay still).
#[derive(Component)]
struct MinigunSpinner;

/// §5.3 (Brief VI): one pooled pod-missile visual.
#[derive(Component)]
struct RocketVis(usize);

/// §5.3: place the pooled missile visuals on the sim's live rockets.
fn sync_rockets(game: Res<Game>, mut q: Query<(&RocketVis, &mut Transform, &mut Visibility)>) {
    for (rv, mut tf, mut vis) in &mut q {
        if let Some(r) = game.sim.rockets.get(rv.0) {
            tf.translation = Vec3::from_array(r.pos);
            let v = Vec3::from_array(r.vel);
            if v.length_squared() > 1e-3 {
                tf.look_to(v.normalize(), Vec3::Y);
            }
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// §2.1 (Brief II): a fully articulated fingered hand - palm, four
/// two-jointed fingers, two-jointed thumb. VIEWMODEL ONLY: it lives on
/// the viewmodel render layer, is never instanced per-bot, and therefore
/// has no bearing on the per-fighter cost §0.3 protects.
fn spawn_hand_fingered(commands: &mut Commands, kit: &ModelKit, curl: f32, mirror: bool) -> Entity {
    let m = if mirror { -1.0_f32 } else { 1.0 };
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // §2.2 (Brief IV, image 4): white shell segments, DARK ball joints,
    // individually segmented fingers with dark knuckle lines - sized to
    // READ in frame, not to hide behind the receiver.
    // wrist ball
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(kit.grey_black.clone()),
            Transform::from_xyz(0.0, 0.0, -0.045).with_scale(Vec3::splat(0.052)),
        ))
        .set_parent(root);
    // palm
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_scale(Vec3::new(0.105, 0.040, 0.115)),
        ))
        .set_parent(root);
    // four fingers, two segments each, dark knuckle balls at both joints
    for (fi, fx) in [-0.038_f32, -0.013, 0.013, 0.038].into_iter().enumerate() {
        let len = if fi == 0 || fi == 3 { 0.044 } else { 0.052 };
        let rest = -1.15 * curl;
        let mut base_cmd = commands.spawn((
            Transform::from_xyz(fx * m, 0.0, 0.058).with_rotation(Quat::from_rotation_x(rest)),
            Visibility::default(),
        ));
        if fi == 0 && !mirror {
            base_cmd.insert(TriggerFinger { rest });
        }
        let base = base_cmd.set_parent(root).id();
        // knuckle line at the palm edge
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(kit.grey_black.clone()),
                Transform::from_scale(Vec3::splat(0.024)),
            ))
            .set_parent(base);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.hand.clone()),
                Transform::from_xyz(0.0, 0.0, len * 0.5).with_scale(Vec3::new(0.022, 0.023, len)),
            ))
            .set_parent(base);
        // §2.2 (Brief VII v2): the tip (DIP-equivalent) joint COUPLES to
        // the base joint's curl at ~0.7x, not an independent coefficient
        // - real fingertips can't out-curl the knuckle behind them, and
        // the old -1.3x (curling the TIP harder than the base) was
        // exactly backwards from how a hand actually closes.
        let tip = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, len)
                    .with_rotation(Quat::from_rotation_x(dip_from_driving_joint(rest))),
                Visibility::default(),
            ))
            .set_parent(base)
            .id();
        // mid-knuckle
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(kit.grey_black.clone()),
                Transform::from_scale(Vec3::splat(0.021)),
            ))
            .set_parent(tip);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.hand.clone()),
                Transform::from_xyz(0.0, 0.0, len * 0.4)
                    .with_scale(Vec3::new(0.020, 0.021, len * 0.8)),
            ))
            .set_parent(tip);
    }
    // thumb: two joints, wrapping from the side (flipped when mirrored)
    let tbase = commands
        .spawn((
            Transform::from_xyz(-0.056 * m, 0.0, 0.018).with_rotation(
                Quat::from_rotation_y(0.9 * curl * m) * Quat::from_rotation_x(-0.5 * curl),
            ),
            Visibility::default(),
        ))
        .set_parent(root)
        .id();
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(kit.grey_black.clone()),
            Transform::from_scale(Vec3::splat(0.024)),
        ))
        .set_parent(tbase);
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_xyz(0.0, 0.0, 0.023).with_scale(Vec3::new(0.024, 0.025, 0.046)),
        ))
        .set_parent(tbase);
    let ttip = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.046)
                .with_rotation(Quat::from_rotation_y(0.9 * curl * m)),
            Visibility::default(),
        ))
        .set_parent(tbase)
        .id();
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_xyz(0.0, 0.0, 0.019).with_scale(Vec3::new(0.021, 0.022, 0.038)),
        ))
        .set_parent(ttip);
    root
}

/// §1.1: hands are MITTENS - a rounded mitt curled to the grip plus a
/// separate thumb. `curl` is the grip strength (1.0 = fist around a rifle
/// grip, 0.55 = the open cradle of a forend); `mirror` flips chirality so
/// the thumb sits on the correct side. Three entities where the fingered
/// hand spent nineteen - a straight perf win on top of the art change.
fn spawn_hand(commands: &mut Commands, kit: &ModelKit, curl: f32, mirror: bool) -> Entity {
    let m = if mirror { -1.0_f32 } else { 1.0 };
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // the mitt: a rounded shell wrapped over the grip
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_xyz(0.0, -0.005, 0.020)
                .with_rotation(Quat::from_rotation_x(-0.55 * curl))
                .with_scale(Vec3::new(0.085, 0.062, 0.115)),
        ))
        .set_parent(root);
    // the thumb: one rounded capsule-ish lobe hooking from the side
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_xyz(-0.042 * m, 0.004, 0.028)
                .with_rotation(
                    Quat::from_rotation_y(0.8 * curl * m) * Quat::from_rotation_x(-0.4 * curl),
                )
                .with_scale(Vec3::new(0.034, 0.034, 0.062)),
        ))
        .set_parent(root);
    root
}

/// Spawn a weapon model from the §2.1 shared part vocabulary. Root sits
/// at the grip; +Z is the muzzle. (The bow stands along +Y with its
/// string toward -Z.) Per-gun differentiation is PROPORTION, not new part
/// types; tone changes - not geometry - suggest complexity. Every gun
/// `with_detail` adds the ADS-only greebles (sights, scope rings) - the
/// detail LOD belongs to YOUR weapons; bots skip the parts entirely.
/// `with_hands` bakes posed mitten hands onto the model (viewmodel only -
/// §1.3 third-person hands are the BODY's, IK'd onto the sockets).
fn spawn_weapon_model(
    commands: &mut Commands,
    kit: &ModelKit,
    kind: GunKind,
    with_detail: bool,
    with_hands: bool,
) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    let mut parts: Vec<WPart> = Vec::new();
    match kind {
        GunKind::Fists => {}
        GunKind::Glock => {
            // compact service pistol: squared mid slide over a dark frame
            parts.push(wp(false, Tone::Mid, (0.0, 0.05, 0.07), 0.0, (0.046, 0.058, 0.22)));
            parts.push(wp(false, Tone::Light, (0.0, 0.05, -0.025), 0.0, (0.048, 0.044, 0.045)));
            parts.push(wp(true, Tone::Black, (0.0, 0.052, 0.185), FRAC_PI_2, (0.026, 0.05, 0.026)));
            parts.push(wp(false, Tone::Dark, (0.0, 0.0, 0.05), 0.0, (0.042, 0.045, 0.20)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.05, -0.01), 0.18, (0.042, 0.13, 0.06)));
            parts.push(wp(false, Tone::Black, (0.0, -0.018, 0.048), 0.0, (0.012, 0.012, 0.05)));
            parts.push(wd(false, Tone::Light, (0.0, 0.085, 0.15), 0.0, (0.008, 0.012, 0.01)));
            parts.push(wd(false, Tone::Light, (0.0, 0.085, -0.03), 0.0, (0.014, 0.012, 0.01)));
        }
        GunKind::Deagle => {
            // the hand cannon: long light slide, heavy dark frame
            parts.push(wp(false, Tone::Light, (0.0, 0.055, 0.10), 0.0, (0.052, 0.075, 0.30)));
            parts.push(wp(false, Tone::Mid, (0.0, 0.096, 0.10), 0.0, (0.030, 0.012, 0.28)));
            parts.push(wp(false, Tone::Dark, (0.0, 0.0, 0.07), 0.0, (0.048, 0.05, 0.24)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.055, -0.01), 0.20, (0.046, 0.14, 0.065)));
            push_muzzle(&mut parts, 0.055, 0.27, 0.055);
            parts.push(wd(false, Tone::Light, (0.0, 0.10, 0.22), 0.0, (0.008, 0.014, 0.01)));
            parts.push(wd(false, Tone::Light, (0.0, 0.10, -0.04), 0.0, (0.016, 0.012, 0.01)));
        }
        GunKind::Mp5 => {
            // compact SMG: short everything - slab receiver, raked mag
            parts.push(wp(false, Tone::Mid, (0.0, 0.02, 0.04), 0.0, (0.05, 0.09, 0.34)));
            push_rail(&mut parts, 0.075, -0.10, 0.18, 6);
            parts.push(wp(true, Tone::Dark, (0.0, 0.03, 0.28), FRAC_PI_2, (0.024, 0.18, 0.024)));
            push_muzzle(&mut parts, 0.03, 0.385, 0.036);
            parts.push(wp(false, Tone::Light, (0.0, -0.012, 0.16), 0.0, (0.052, 0.058, 0.14)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.10, 0.10), 0.35, (0.032, 0.17, 0.06)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.07, -0.05), 0.2, (0.04, 0.11, 0.05)));
            // folded stock: flat end cap + side-folded strut hugging the
            // receiver - nothing protrudes past the grip line
            parts.push(wp(false, Tone::Mid, (0.0, 0.02, -0.135), 0.0, (0.046, 0.075, 0.018)));
            parts.push(wp(false, Tone::Dark, (0.055, 0.025, 0.02), 0.0, (0.016, 0.02, 0.26)));
        }
        GunKind::Shotgun => {
            // pump gun: barrel + tube pair over a light pump
            parts.push(wp(false, Tone::Dark, (0.0, 0.02, 0.02), 0.0, (0.05, 0.085, 0.30)));
            parts.push(wp(true, Tone::Mid, (0.0, 0.045, 0.38), FRAC_PI_2, (0.028, 0.48, 0.028)));
            parts.push(wp(true, Tone::Dark, (0.0, -0.005, 0.36), FRAC_PI_2, (0.024, 0.42, 0.024)));
            parts.push(wp(false, Tone::Light, (0.0, -0.015, 0.30), 0.0, (0.054, 0.05, 0.16)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.035, -0.20), 0.12, (0.045, 0.10, 0.26)));
            parts.push(wp(false, Tone::Mid, (0.0, -0.035, -0.325), 0.12, (0.05, 0.11, 0.02)));
            parts.push(wd(false, Tone::Light, (0.0, 0.09, 0.55), 0.0, (0.008, 0.016, 0.01)));
        }
        GunKind::Ak47 => {
            // the classic: long gas tube, big two-segment raked magazine
            parts.push(wp(false, Tone::Mid, (0.0, 0.02, 0.06), 0.0, (0.05, 0.085, 0.38)));
            parts.push(wp(false, Tone::Light, (0.0, 0.068, 0.0), 0.0, (0.048, 0.02, 0.22)));
            parts.push(wp(true, Tone::Dark, (0.0, 0.045, 0.44), FRAC_PI_2, (0.026, 0.36, 0.026)));
            parts.push(wp(true, Tone::Light, (0.0, 0.078, 0.34), FRAC_PI_2, (0.020, 0.18, 0.020)));
            parts.push(wp(false, Tone::Dark, (0.0, 0.01, 0.32), 0.0, (0.05, 0.055, 0.18)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.10, 0.09), 0.35, (0.036, 0.14, 0.075)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.185, 0.14), 0.75, (0.034, 0.11, 0.07)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.08, -0.05), 0.30, (0.04, 0.10, 0.05)));
            push_stock(&mut parts, -0.30, 0.045);
            push_muzzle(&mut parts, 0.045, 0.635, 0.038);
            parts.push(wd(false, Tone::Light, (0.0, 0.085, 0.10), 0.0, (0.014, 0.02, 0.012)));
            parts.push(wd(false, Tone::Light, (0.0, 0.09, 0.58), 0.0, (0.008, 0.018, 0.01)));
        }
        GunKind::M4 => {
            // modern carbine: notched top rail, straight raked mag
            parts.push(wp(false, Tone::Mid, (0.0, 0.02, 0.08), 0.0, (0.05, 0.09, 0.42)));
            push_rail(&mut parts, 0.078, -0.08, 0.30, 10);
            parts.push(wp(true, Tone::Dark, (0.0, 0.03, 0.45), FRAC_PI_2, (0.028, 0.34, 0.028)));
            push_muzzle(&mut parts, 0.03, 0.635, 0.04);
            parts.push(wp(false, Tone::Dark, (0.0, -0.005, 0.30), 0.0, (0.055, 0.055, 0.20)));
            parts.push(wp(false, Tone::Light, (0.032, -0.005, 0.30), 0.0, (0.008, 0.02, 0.16)));
            parts.push(wp(false, Tone::Light, (-0.032, -0.005, 0.30), 0.0, (0.008, 0.02, 0.16)));
            parts.push(wp(false, Tone::Mid, (0.0, -0.09, 0.06), 0.15, (0.038, 0.16, 0.08)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.08, -0.06), 0.35, (0.04, 0.10, 0.05)));
            push_stock(&mut parts, -0.30, 0.05);
            parts.push(wd(false, Tone::Light, (0.0, 0.105, 0.24), 0.0, (0.008, 0.018, 0.01)));
            parts.push(wd(false, Tone::Light, (0.0, 0.105, -0.02), 0.0, (0.014, 0.016, 0.01)));
        }
        GunKind::Awm => {
            // the AWM: long barrel, big scope block, solid cheek stock
            parts.push(wp(false, Tone::Light, (0.0, 0.01, 0.05), 0.0, (0.045, 0.08, 0.46)));
            parts.push(wp(true, Tone::Mid, (0.0, 0.03, 0.55), FRAC_PI_2, (0.024, 0.55, 0.024)));
            push_muzzle(&mut parts, 0.03, 0.85, 0.036);
            parts.push(wp(true, Tone::Dark, (0.0, 0.10, 0.08), FRAC_PI_2, (0.055, 0.20, 0.055)));
            parts.push(wp(true, Tone::Black, (0.0, 0.10, 0.185), FRAC_PI_2, (0.045, 0.012, 0.045)));
            parts.push(wp(false, Tone::Light, (0.0, -0.02, -0.22), 0.0, (0.045, 0.11, 0.30)));
            parts.push(wp(false, Tone::Black, (0.0, -0.02, -0.375), 0.0, (0.048, 0.12, 0.02)));
            parts.push(wp(false, Tone::Dark, (0.045, -0.08, 0.42), 0.0, (0.012, 0.14, 0.012)));
            parts.push(wp(false, Tone::Dark, (-0.045, -0.08, 0.42), 0.0, (0.012, 0.14, 0.012)));
            parts.push(wd(true, Tone::Light, (0.0, 0.10, 0.02), FRAC_PI_2, (0.060, 0.02, 0.060)));
            parts.push(wd(true, Tone::Light, (0.0, 0.10, 0.14), FRAC_PI_2, (0.060, 0.02, 0.060)));
        }
        GunKind::M249 => {
            // belt-fed support gun: deep receiver, box mag, thick barrel
            parts.push(wp(false, Tone::Dark, (0.0, 0.02, 0.05), 0.0, (0.075, 0.12, 0.50)));
            parts.push(wp(false, Tone::Light, (0.0, 0.088, 0.05), 0.0, (0.07, 0.02, 0.30)));
            parts.push(wp(true, Tone::Dark, (0.0, 0.04, 0.50), FRAC_PI_2, (0.045, 0.40, 0.045)));
            push_muzzle(&mut parts, 0.04, 0.73, 0.06);
            parts.push(wp(false, Tone::Mid, (0.0, -0.13, 0.02), 0.0, (0.09, 0.16, 0.13)));
            parts.push(wp(false, Tone::Light, (0.0, 0.12, 0.08), 0.0, (0.02, 0.06, 0.16)));
            push_stock(&mut parts, -0.30, 0.05);
            parts.push(wp(false, Tone::Dark, (0.03, -0.10, 0.44), 0.0, (0.014, 0.16, 0.014)));
            parts.push(wp(false, Tone::Dark, (-0.03, -0.10, 0.44), 0.0, (0.014, 0.16, 0.014)));
            parts.push(wd(false, Tone::Light, (0.0, 0.10, 0.30), 0.0, (0.01, 0.02, 0.012)));
        }
        GunKind::Bow => {
            // hard-surface war bow: dark blocky limbs, mid riser, light tips
            parts.push(wp(false, Tone::Dark, (0.0, 0.24, 0.03), -0.30, (0.028, 0.46, 0.045)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.24, 0.03), 0.30, (0.028, 0.46, 0.045)));
            parts.push(wp(false, Tone::Mid, (0.0, 0.0, 0.01), 0.0, (0.035, 0.15, 0.055)));
            parts.push(wp(false, Tone::Light, (0.0, 0.455, -0.04), -0.30, (0.032, 0.05, 0.05)));
            parts.push(wp(false, Tone::Light, (0.0, -0.455, -0.04), 0.30, (0.032, 0.05, 0.05)));
            parts.push(wp(false, Tone::Light, (0.0, 0.0, -0.09), 0.0, (0.008, 0.82, 0.008)));
            parts.push(wd(false, Tone::Light, (0.0, 0.10, 0.02), 0.0, (0.038, 0.02, 0.058)));
            parts.push(wd(false, Tone::Light, (0.0, -0.10, 0.02), 0.0, (0.038, 0.02, 0.058)));
        }
        GunKind::Spear => {
            // war spear: dark shaft, light flat blade, black collar + butt
            parts.push(wp(true, Tone::Dark, (0.0, 0.0, 0.35), FRAC_PI_2, (0.032, 1.85, 0.032)));
            parts.push(wp(false, Tone::Light, (0.0, 0.0, 1.32), 0.0, (0.055, 0.018, 0.22)));
            parts.push(wp(true, Tone::Black, (0.0, 0.0, 1.18), FRAC_PI_2, (0.042, 0.09, 0.042)));
            parts.push(wp(true, Tone::Black, (0.0, 0.0, -0.56), FRAC_PI_2, (0.038, 0.07, 0.038)));
            parts.push(wd(false, Tone::Light, (0.0, 0.0, 1.10), 0.0, (0.045, 0.045, 0.02)));
        }
        GunKind::Minigun => {
            // §7: six-barrel cluster round a spine, deep motor housing at
            // the rear, spade grip below - no stock, the housing IS the
            // brace. Reads as MASS from every angle. The whole barrel
            // cluster lives on its own SPINNER child so the viewmodel can
            // rotate it with the sim's spin_t.
            let spinner = commands
                .spawn((Transform::IDENTITY, Visibility::default()))
                .id();
            if with_hands {
                commands.entity(spinner).insert(MinigunSpinner);
            }
            commands.entity(spinner).set_parent(root);
            let mut cluster: Vec<(Tone, Vec3, Vec3)> = vec![
                (Tone::Mid, Vec3::new(0.0, 0.0, 0.24), Vec3::new(0.024, 0.52, 0.024)),
                (Tone::Black, Vec3::new(0.0, 0.0, 0.50), Vec3::new(0.082, 0.05, 0.082)),
                (Tone::Black, Vec3::new(0.0, 0.0, 0.16), Vec3::new(0.086, 0.05, 0.086)),
            ];
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.0;
                let (bx, by) = (a.cos() * 0.052, a.sin() * 0.052);
                cluster.push((
                    Tone::Dark,
                    Vec3::new(bx, by, 0.26),
                    Vec3::new(0.017, 0.56, 0.017),
                ));
            }
            for (tone, pos, size) in cluster {
                commands
                    .spawn((
                        Mesh3d(kit.cyl.clone()),
                        MeshMaterial3d(kit.tone(tone)),
                        Transform {
                            translation: pos,
                            rotation: Quat::from_rotation_x(FRAC_PI_2),
                            scale: size,
                        },
                    ))
                    .set_parent(spinner);
            }
            // motor housing + rear cap (the torso-brace face)
            parts.push(wp(false, Tone::Dark, (0.0, 0.0, -0.05), 0.0, (0.13, 0.15, 0.18)));
            parts.push(wp(false, Tone::Mid, (0.0, 0.0, -0.135), 0.0, (0.11, 0.13, 0.02)));
            // spade grip under the rear + side support handle
            parts.push(wp(false, Tone::Black, (0.0, -0.115, -0.07), 0.15, (0.03, 0.09, 0.04)));
            parts.push(wp(false, Tone::Black, (-0.02, -0.10, 0.18), 0.0, (0.026, 0.10, 0.035)));
            // feed chute hint on the right flank
            parts.push(wd(false, Tone::Light, (0.075, -0.02, -0.02), 0.0, (0.02, 0.06, 0.10)));
        }
    }
    for p in parts {
        if p.detail && !with_detail {
            continue; // bots carry the plain silhouette - the LOD working
        }
        let mesh = if p.cyl { kit.cyl.clone() } else { kit.cube.clone() };
        let mut e = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(kit.tone(p.tone)),
            Transform {
                translation: p.pos,
                rotation: Quat::from_rotation_x(p.tilt),
                scale: p.size,
            },
        ));
        if p.detail {
            // aiming-only greebles (§5: weapon detail steps up on ADS)
            e.insert((Visibility::Hidden, AdsDetail));
        }
        e.set_parent(root);
    }
    // §1.2 (Brief VI): the on-weapon ammo bar - 8 emissive ticks on the
    // left receiver face, VIEWMODEL ONLY, driven by `ammo_bar_sync`
    if with_hands
        && !matches!(
            kind,
            GunKind::Fists | GunKind::Bow | GunKind::Spear
        )
    {
        let bx = ammo_bar_x(kind);
        for i in 0..AMMO_BAR_SEGS {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.med_glow.clone()),
                    Transform::from_xyz(bx, 0.015, -0.01 + i as f32 * 0.017)
                        .with_scale(Vec3::new(0.004, 0.009, 0.012)),
                    AmmoBarSeg { idx: i },
                ))
                .set_parent(root);
        }
    }
    // §2.1: the viewmodel's hands are FULLY FINGERED - trigger finger,
    // grip wrap, thumb-over - where the world model wears mittens
    if with_hands {
        for (pos, ry, curl, mirror) in weapon_hand_specs(kind) {
            let hand = spawn_hand_fingered(commands, kit, curl, mirror);
            commands
                .entity(hand)
                .insert(Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(ry)))
                .set_parent(root);
        }
    }
    root
}

/// The always-carried tower shield: rounded plate, boss, sight slit,
/// gold trim - held on the left arm when raised (E).
fn spawn_shield_model(commands: &mut Commands, kit: &ModelKit) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // three gently angled slats fake a curved plate
    for (x, ry) in [(-0.16_f32, 0.28_f32), (0.0, 0.0), (0.16, -0.28)] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.armor_dark.clone()),
                Transform::from_xyz(x, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(ry))
                    .with_scale(Vec3::new(0.20, 0.72, 0.045)),
            ))
            .set_parent(root);
    }
    // boss
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.steel.clone()),
            Transform::from_xyz(0.0, 0.05, 0.04)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                .with_scale(Vec3::new(0.18, 0.05, 0.18)),
        ))
        .set_parent(root);
    // sight slit (glows faintly - the robot looks THROUGH the plate)
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.lens.clone()),
            Transform::from_xyz(0.0, 0.26, 0.028).with_scale(Vec3::new(0.16, 0.02, 0.01)),
        ))
        .set_parent(root);
    // gold trim top + bottom
    for y in [0.37_f32, -0.37] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.gold.clone()),
                Transform::from_xyz(0.0, y, 0.0).with_scale(Vec3::new(0.46, 0.035, 0.05)),
            ))
            .set_parent(root);
    }
    // handle bar + a gripping fist behind the plate
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.steel.clone()),
            Transform::from_xyz(0.0, 0.0, -0.06)
                .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
                .with_scale(Vec3::new(0.03, 0.16, 0.03)),
        ))
        .set_parent(root);
    let fist = spawn_hand(commands, kit, 1.0, true);
    commands
        .entity(fist)
        .insert(Transform::from_xyz(0.0, -0.02, -0.08))
        .set_parent(root);
    root
}

/// Spawn the wearable robot-armor rig (chest plate, power pack, pauldrons).
fn spawn_armor_rig(commands: &mut Commands, kit: &ModelKit) -> Entity {
    // §11 (Brief IV): the Mech Armada read - khaki FACETED plates (each
    // face slightly canted so light breaks across them), a single red
    // sensor slit instead of eyes, boxy shoulder pods, exhaust stubs.
    // Worn over the fighter rig; the sim's arcs/hull/eject are untouched.
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    let plates: [(Handle<StandardMaterial>, Vec3, Quat, Vec3); 33] = [
        // chest: two canted khaki facets meeting on the centerline
        (kit.mech_khaki.clone(), Vec3::new(0.13, 0.34, 0.10),
         Quat::from_rotation_y(-0.22), Vec3::new(0.30, 0.46, 0.16)),
        (kit.mech_khaki.clone(), Vec3::new(-0.13, 0.34, 0.10),
         Quat::from_rotation_y(0.22), Vec3::new(0.30, 0.46, 0.16)),
        // collar wedge above the chest facets
        (kit.mech_khaki_lt.clone(), Vec3::new(0.0, 0.56, 0.06),
         Quat::from_rotation_x(0.30), Vec3::new(0.34, 0.10, 0.16)),
        // abdomen band - darker, recessed
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.10, 0.05),
         Quat::IDENTITY, Vec3::new(0.40, 0.14, 0.22)),
        // back: power pack + twin exhaust stubs
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.36, -0.20),
         Quat::IDENTITY, Vec3::new(0.34, 0.40, 0.16)),
        (kit.mech_metal.clone(), Vec3::new(0.10, 0.52, -0.26),
         Quat::from_rotation_x(-0.20), Vec3::new(0.07, 0.16, 0.07)),
        (kit.mech_metal.clone(), Vec3::new(-0.10, 0.52, -0.26),
         Quat::from_rotation_x(-0.20), Vec3::new(0.07, 0.16, 0.07)),
        // shoulder pods: boxy, khaki top over shadow underside
        (kit.mech_khaki.clone(), Vec3::new(0.36, 0.60, 0.0),
         Quat::from_rotation_z(-0.12), Vec3::new(0.22, 0.16, 0.24)),
        (kit.mech_khaki.clone(), Vec3::new(-0.36, 0.60, 0.0),
         Quat::from_rotation_z(0.12), Vec3::new(0.22, 0.16, 0.24)),
        (kit.mech_shadow.clone(), Vec3::new(0.36, 0.50, 0.0),
         Quat::IDENTITY, Vec3::new(0.18, 0.08, 0.20)),
        (kit.mech_shadow.clone(), Vec3::new(-0.36, 0.50, 0.0),
         Quat::IDENTITY, Vec3::new(0.18, 0.08, 0.20)),
        // §4.2 (Brief VI): NO head - an angular recessed SENSOR VISOR
        // slit across the hull front, emissive red (Brief IV language)
        (kit.mech_khaki.clone(), Vec3::new(0.0, 0.90, -0.01),
         Quat::from_rotation_x(0.08), Vec3::new(0.26, 0.16, 0.24)),
        (kit.mech_red.clone(), Vec3::new(0.0, 0.885, 0.115),
         Quat::IDENTITY, Vec3::new(0.16, 0.028, 0.03)),
        // pelvis skirt plate
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, -0.02, 0.10),
         Quat::from_rotation_x(-0.25), Vec3::new(0.34, 0.12, 0.10)),
        // §4.2: hazard chevrons - pod cover + knee plates ONLY
        (kit.mech_hazard.clone(), Vec3::new(-0.36, 0.685, 0.02),
         Quat::from_rotation_z(0.12), Vec3::new(0.20, 0.02, 0.22)),
        (kit.mech_hazard.clone(), Vec3::new(0.14, -0.32, 0.12),
         Quat::from_rotation_x(-0.2), Vec3::new(0.10, 0.06, 0.02)),
        (kit.mech_hazard.clone(), Vec3::new(-0.14, -0.32, 0.12),
         Quat::from_rotation_x(-0.2), Vec3::new(0.10, 0.06, 0.02)),
        // §4.2: knee plates under the chevrons
        (kit.mech_khaki_dk.clone(), Vec3::new(0.14, -0.36, 0.10),
         Quat::from_rotation_x(-0.2), Vec3::new(0.12, 0.14, 0.06)),
        (kit.mech_khaki_dk.clone(), Vec3::new(-0.14, -0.36, 0.10),
         Quat::from_rotation_x(-0.2), Vec3::new(0.12, 0.14, 0.06)),
        // §4.2/§5.3: LEFT-shoulder 4-tube missile pod - tube caps
        // visible, one status light per tube
        (kit.mech_khaki_dk.clone(), Vec3::new(-0.36, 0.62, 0.04),
         Quat::IDENTITY, Vec3::new(0.18, 0.12, 0.20)),
        (kit.mech_shadow.clone(), Vec3::new(-0.315, 0.645, 0.145),
         Quat::IDENTITY, Vec3::new(0.055, 0.055, 0.02)),
        (kit.mech_shadow.clone(), Vec3::new(-0.405, 0.645, 0.145),
         Quat::IDENTITY, Vec3::new(0.055, 0.055, 0.02)),
        (kit.mech_shadow.clone(), Vec3::new(-0.315, 0.595, 0.145),
         Quat::IDENTITY, Vec3::new(0.055, 0.055, 0.02)),
        (kit.mech_shadow.clone(), Vec3::new(-0.405, 0.595, 0.145),
         Quat::IDENTITY, Vec3::new(0.055, 0.055, 0.02)),
        (kit.mech_red.clone(), Vec3::new(-0.27, 0.62, 0.145),
         Quat::IDENTITY, Vec3::new(0.015, 0.09, 0.01)),
        // §4.2: antenna mast, rear-left
        (kit.mech_metal.clone(), Vec3::new(-0.20, 0.86, -0.24),
         Quat::from_rotation_x(-0.12), Vec3::new(0.015, 0.34, 0.015)),
        // Task 5.4 (MISSION doc): knee and waist mechanisms stay EXPOSED
        // - dark actuator/piston stubs poking past the plates, deliberately
        // uncovered. This is what reads as "real machinery" rather than a
        // smooth robot costume, per the doc's own explicit rule.
        (kit.mech_metal.clone(), Vec3::new(0.155, -0.30, 0.16),
         Quat::from_rotation_x(0.35), Vec3::new(0.035, 0.10, 0.035)),
        (kit.mech_metal.clone(), Vec3::new(-0.155, -0.30, 0.16),
         Quat::from_rotation_x(0.35), Vec3::new(0.035, 0.10, 0.035)),
        // waist actuator block - the busiest area on the real art
        // reference (Task 1 notes): visible linkage where the pelvis
        // skirt doesn't reach
        (kit.mech_metal.clone(), Vec3::new(0.0, 0.02, 0.18),
         Quat::from_rotation_x(0.15), Vec3::new(0.14, 0.06, 0.06)),
        (kit.mech_shadow.clone(), Vec3::new(0.09, -0.01, 0.19),
         Quat::from_rotation_z(0.4), Vec3::new(0.03, 0.09, 0.03)),
        // hazard/wear detail pass: the pod cover was the ONLY hazard
        // stripe on the whole hull, so the right side read as an
        // unfinished mirror. A matching stripe on the (pod-less) right
        // shoulder pod restores the left/right read as one assembled
        // machine, not "pod side gets details, other side doesn't."
        (kit.mech_hazard.clone(), Vec3::new(0.36, 0.685, 0.02),
         Quat::from_rotation_z(-0.12), Vec3::new(0.20, 0.02, 0.22)),
        // power-pack warning band - a thin hazard strip across the top
        // edge, where a real power unit would carry a stenciled warning
        (kit.mech_hazard.clone(), Vec3::new(0.0, 0.50, -0.135),
         Quat::IDENTITY, Vec3::new(0.30, 0.02, 0.02)),
        // scuffed metal patch on the abdomen - bare metal showing through
        // worn khaki paint, the cheapest possible "this hull has seen use"
        // cue without a real decal/stencil system
        (kit.mech_metal.clone(), Vec3::new(0.09, 0.05, 0.16),
         Quat::from_rotation_z(0.3), Vec3::new(0.08, 0.05, 0.01)),
    ];
    for (mat, tr, rot, sc) in plates {
        commands
            .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                translation: tr,
                rotation: rot,
                scale: sc,
            }))
            .set_parent(root);
    }
    root
}

/// Spawn the floating model shown over a pickup pad.
fn spawn_pickup_model(commands: &mut Commands, kit: &ModelKit, kind: PickupKind) -> Entity {
    match kind {
        PickupKind::Health => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 1.0, 0.0), Visibility::default()))
                .id();
            for (mat, tr, sc) in [
                (kit.white.clone(), Vec3::ZERO, Vec3::new(0.30, 0.24, 0.30)),
                (kit.med_glow.clone(), Vec3::new(0.0, 0.0, 0.16), Vec3::new(0.20, 0.06, 0.02)),
                (kit.med_glow.clone(), Vec3::new(0.0, 0.0, 0.16), Vec3::new(0.06, 0.20, 0.02)),
                (kit.med_glow.clone(), Vec3::new(0.0, 0.13, 0.0), Vec3::new(0.20, 0.02, 0.06)),
            ] {
                commands
                    .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                        translation: tr,
                        rotation: Quat::IDENTITY,
                        scale: sc,
                    }))
                    .set_parent(root);
            }
            root
        }
        PickupKind::Ammo => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 1.0, 0.0), Visibility::default()))
                .id();
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.olive.clone()),
                    Transform::from_scale(Vec3::new(0.34, 0.22, 0.26)),
                ))
                .set_parent(root);
            for x in [-0.09_f32, 0.0, 0.09] {
                commands
                    .spawn((
                        Mesh3d(kit.cyl.clone()),
                        MeshMaterial3d(kit.gold.clone()),
                        Transform::from_xyz(x, 0.17, 0.0).with_scale(Vec3::new(0.05, 0.14, 0.05)),
                    ))
                    .set_parent(root);
            }
            root
        }
        PickupKind::RobotArmor => {
            let e = spawn_armor_rig(commands, kit);
            commands.entity(e).insert(
                Transform::from_xyz(0.0, 0.75, 0.0).with_scale(Vec3::splat(0.9)),
            );
            e
        }
        // §6: the three new set pads - a readable silhouette each
        PickupKind::FolkArmor => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 0.9, 0.0), Visibility::default()))
                .id();
            for (mat, tr, sc) in [
                (kit.steel.clone(), Vec3::ZERO, Vec3::new(0.42, 0.40, 0.24)),
                (kit.wood.clone(), Vec3::new(0.0, -0.06, 0.13), Vec3::new(0.44, 0.08, 0.02)),
                (kit.steel.clone(), Vec3::new(0.0, 0.26, 0.0), Vec3::new(0.30, 0.12, 0.22)),
            ] {
                commands
                    .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                        translation: tr,
                        rotation: Quat::IDENTITY,
                        scale: sc,
                    }))
                    .set_parent(root);
            }
            root
        }
        PickupKind::PyroArmor => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 0.9, 0.0), Visibility::default()))
                .id();
            for (mat, tr, sc) in [
                (kit.grey_black.clone(), Vec3::ZERO, Vec3::new(0.42, 0.40, 0.24)),
                (kit.core_glow.clone(), Vec3::new(0.0, 0.0, 0.13), Vec3::new(0.30, 0.04, 0.02)),
                (kit.core_glow.clone(), Vec3::new(0.0, 0.14, 0.13), Vec3::new(0.20, 0.04, 0.02)),
            ] {
                commands
                    .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                        translation: tr,
                        rotation: Quat::IDENTITY,
                        scale: sc,
                    }))
                    .set_parent(root);
            }
            root
        }
        PickupKind::ReconWeave => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 0.9, 0.0), Visibility::default()))
                .id();
            for (mat, tr, sc) in [
                (kit.grey_mid.clone(), Vec3::ZERO, Vec3::new(0.36, 0.42, 0.18)),
                (kit.lens.clone(), Vec3::new(0.0, 0.20, 0.10), Vec3::new(0.16, 0.03, 0.02)),
            ] {
                commands
                    .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                        translation: tr,
                        rotation: Quat::IDENTITY,
                        scale: sc,
                    }))
                    .set_parent(root);
            }
            root
        }
        // §7: the pad shows the gun itself, laid across the plinth
        PickupKind::Minigun => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 0.9, 0.0), Visibility::default()))
                .id();
            for (mat, tr, sc) in [
                // barrel cluster hint: a fat cylinder-read from cubes
                (kit.grey_dark.clone(), Vec3::new(0.0, 0.0, 0.22), Vec3::new(0.11, 0.11, 0.44)),
                (kit.grey_black.clone(), Vec3::new(0.0, 0.0, -0.10), Vec3::new(0.16, 0.18, 0.24)),
                (kit.grey_mid.clone(), Vec3::new(0.0, -0.12, 0.10), Vec3::new(0.05, 0.10, 0.06)),
                (kit.grey_black.clone(), Vec3::new(0.0, 0.0, 0.45), Vec3::new(0.13, 0.13, 0.04)),
            ] {
                commands
                    .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                        translation: tr,
                        rotation: Quat::IDENTITY,
                        scale: sc,
                    }))
                    .set_parent(root);
            }
            root
        }
    }
}

// ------------------------------------------- match-scoped world builders --

/// Build every fighter's rig. §1 art direction: the WHITE SERVICE ROBOT -
/// matte white shell panels over exposed dark gloss joints, an oversized
/// rounded head with two big black oval eyes (no mouth, no visor), glossy
/// knee domes, mitten hands. Same skeleton as ever: hips/knees/ankles,
/// shoulder/elbow/wrist - the animation code is untouched.
///
/// ! §1.2: the sim classifies hits by HEIGHT FRACTION (head > 0.82,
/// arms > 0.66, torso > 0.35 - `apply_hit`). The model FITS those bands:
/// standing hip 0.63 (legs fill 0.00–0.35), chest tops out at 1.19,
/// shoulder yoke rides 1.19–1.476, and the head base sits exactly on the
/// 0.82 line (1.476) with its crown at 1.80. Check any change with F3.
fn spawn_fighter_rigs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    kit: &ModelKit,
    sim: &TdmSim,
    sel: &Selected,
) {
    let metal = |c: Color, m: f32, r: f32| StandardMaterial {
        base_color: c,
        metallic: m,
        perceptual_roughness: r,
        ..default()
    };
    // rounded primitives only - no hard edges on a body
    let mesh_thigh = meshes.add(Capsule3d::new(0.072, 0.15));
    let mesh_shin = meshes.add(Capsule3d::new(0.060, 0.15));
    // §1 (Brief IV): shells LONG enough to reach their joint balls -
    // zero daylight, in every pose, by construction
    let mesh_upper = meshes.add(Capsule3d::new(0.055, 0.14));
    let mesh_fore = meshes.add(Capsule3d::new(0.048, 0.12));
    // §1.4 shared shell/joint materials - created ONCE per rebuild, cloned
    // per fighter only for the accent slot
    let shell = materials.add(metal(Color::srgb_u8(0xED, 0xEE, 0xF0), 0.0, 0.42));
    let shell2 = materials.add(metal(Color::srgb_u8(0xDC, 0xDE, 0xE1), 0.0, 0.45));
    let joint = materials.add(metal(Color::srgb_u8(0x17, 0x19, 0x1D), 0.85, 0.22));
    let knee = materials.add(metal(Color::srgb_u8(0x0E, 0x10, 0x13), 0.20, 0.08));
    let eye_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x0A, 0x0B, 0x0D),
        perceptual_roughness: 0.15,
        // a faint cool glow so the eyes still read at range
        emissive: LinearRgba::new(0.016, 0.021, 0.028, 1.0),
        ..default()
    });
    let accent_of = |team: Team| {
        let (r, g, b) = match team {
            Team::Blue => (0x3F as f32, 0x7B as f32, 0xD6 as f32),
            Team::Red => (0xD6 as f32, 0x50 as f32, 0x3F as f32),
        };
        let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
        StandardMaterial {
            base_color: Color::srgb(r, g, b),
            perceptual_roughness: 0.35,
            emissive: LinearRgba::new(r * 0.4, g * 0.4, b * 0.4, 1.0),
            ..default()
        }
    };
    let accent_blue = materials.add(accent_of(Team::Blue));
    let accent_red = materials.add(accent_of(Team::Red));

    for (i, f) in sim.fighters.iter().enumerate() {
        let is_player = i == sim.player;
        let slot = i % 5;
        let accent = match f.team {
            Team::Blue => accent_blue.clone(),
            Team::Red => accent_red.clone(),
        };
        // cosmetics: the player's tunic pick tints THEIR waist stripe;
        // team identity stays on the emblem ring + shoulder band
        let stripe = if is_player {
            let (_, (r, g, b)) = TUNIC_CHOICES[sel.tunic % TUNIC_CHOICES.len()];
            materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
                perceptual_roughness: 0.35,
                emissive: LinearRgba::new(r * 0.4, g * 0.4, b * 0.4, 1.0),
                ..default()
            })
        } else {
            accent.clone()
        };
        let hat_c = if is_player {
            let (_, (r, g, b)) = HAT_CHOICES[sel.hat % HAT_CHOICES.len()];
            Color::srgb(r, g, b)
        } else {
            hat_color(slot, false)
        };
        let hat = materials.add(metal(hat_c, 0.05, 0.85));
        let root = commands
            .spawn((
                Transform::from_xyz(f.pos[0], f.pos[1], f.pos[2]),
                Visibility::default(),
                FighterVis { index: i },
            ))
            .id();
        // LEGS fill the 0.00–0.63 budget: thigh 0.29, shin 0.28, foot 0.06.
        // Every joint is a visible dark gap; the KNEE is the signature - a
        // glossy dark dome sitting proud of the shin.
        let mut legs = [[Entity::PLACEHOLDER; 3]; 2];
        for (li, lx) in [(-0.11_f32), 0.11].into_iter().enumerate() {
            let thigh = commands
                .spawn((Transform::from_xyz(lx, 0.63, 0.0), Visibility::default()))
                .set_parent(root)
                .id();
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(joint.clone()),
                    Transform::from_scale(Vec3::new(0.13, 0.11, 0.13)),
                ))
                .set_parent(thigh);
            commands
                .spawn((
                    Mesh3d(mesh_thigh.clone()),
                    MeshMaterial3d(shell.clone()),
                    Transform::from_xyz(0.0, -0.145, 0.0),
                ))
                .set_parent(thigh);
            let shin = commands
                .spawn((Transform::from_xyz(0.0, -0.29, 0.0), Visibility::default()))
                .set_parent(thigh)
                .id();
            // the glossy knee dome, clearly larger than the other joints
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(knee.clone()),
                    Transform::from_xyz(0.0, -0.005, 0.045)
                        .with_scale(Vec3::new(0.13, 0.13, 0.13)),
                ))
                .set_parent(shin);
            commands
                .spawn((
                    Mesh3d(mesh_shin.clone()),
                    MeshMaterial3d(shell2.clone()),
                    Transform::from_xyz(0.0, -0.14, 0.0),
                ))
                .set_parent(shin);
            let foot = commands
                .spawn((Transform::from_xyz(0.0, -0.28, 0.0), Visibility::default()))
                .set_parent(shin)
                .id();
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(joint.clone()),
                    Transform::from_scale(Vec3::new(0.09, 0.07, 0.09)),
                ))
                .set_parent(foot);
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(shell.clone()),
                    Transform::from_xyz(0.0, -0.025, 0.05)
                        .with_scale(Vec3::new(0.14, 0.09, 0.22)),
                ))
                .set_parent(foot);
            legs[li] = [thigh, shin, foot];
        }
        let [leg_l, leg_r] = legs;
        // TORSO: pelvis → abdomen → chest shells (0.63 → 1.19 world),
        // shoulder yoke 1.19 → 1.476, all under one animated pivot
        let torso = commands
            .spawn((Transform::from_xyz(0.0, 0.63, 0.0), Visibility::default()))
            .set_parent(root)
            .id();
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(shell.clone()),
                Transform::from_xyz(0.0, 0.09, 0.0).with_scale(Vec3::new(0.34, 0.16, 0.26)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(shell2.clone()),
                Transform::from_xyz(0.0, 0.235, 0.0).with_scale(Vec3::new(0.30, 0.24, 0.24)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(shell.clone()),
                Transform::from_xyz(0.0, 0.455, 0.0).with_scale(Vec3::new(0.40, 0.30, 0.30)),
            ))
            .set_parent(torso);
        // §1.4 accent 1/3: the thin waist stripe (player: tunic pick)
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(stripe),
                Transform::from_xyz(0.0, 0.155, 0.0).with_scale(Vec3::new(0.345, 0.03, 0.27)),
            ))
            .set_parent(torso);
        // §1.4 accent 2/3: the chest emblem - a small ring inset on the
        // upper-left chest, dark center (the player's is gold)
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(accent.clone()),
                Transform::from_xyz(-0.09, 0.52, 0.145)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                    .with_scale(Vec3::new(0.075, 0.012, 0.075)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(if is_player {
                    kit.gold.clone()
                } else {
                    joint.clone()
                }),
                Transform::from_xyz(-0.09, 0.52, 0.152)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                    .with_scale(Vec3::new(0.04, 0.012, 0.04)),
            ))
            .set_parent(torso);
        // shoulder yoke + §1.4 accent 3/3: the band across it - visible
        // from front, back, and both sides. §1 (Brief IV): wide enough
        // to reach past the shoulder pivots - no daylight at the arms.
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(shell.clone()),
                Transform::from_xyz(0.0, 0.625, 0.0)
                    .with_scale(Vec3::new(YOKE_HALF_W * 2.0, 0.14, 0.24)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(accent.clone()),
                Transform::from_xyz(0.0, 0.625, 0.0)
                    .with_scale(Vec3::new(YOKE_HALF_W * 2.0 + 0.01, 0.032, 0.245)),
            ))
            .set_parent(torso);
        // §1.2 (Brief IV) THE NECK: a dark cylinder BRIDGING yoke → head -
        // sunk into the shoulders below, piercing past the head pivot
        // above, so the head-to-body connection survives the full look
        // range. A joint fills its gap; background never shows through.
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(joint.clone()),
                Transform::from_xyz(0.0, (NECK_BOT + NECK_TOP) * 0.5, 0.0)
                    .with_scale(Vec3::new(NECK_R * 2.0, NECK_TOP - NECK_BOT, NECK_R * 2.0)),
            ))
            .set_parent(torso);
        // HEAD: the focal mass - a rounded ellipsoid, wider than tall,
        // matte white, two big black oval eyes set wide and low. Its BASE
        // sits exactly on the 0.82 hit-band line (world 1.476): every
        // pixel of face you can see is a real ×4 headshot.
        let head = commands
            .spawn((Transform::from_xyz(0.0, 0.846, 0.0), Visibility::default()))
            .set_parent(torso)
            .id();
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(shell.clone()),
                Transform::from_xyz(0.0, 0.162, 0.01).with_scale(Vec3::new(0.38, 0.324, 0.35)),
            ))
            .set_parent(head);
        for ex in [-0.075_f32, 0.075] {
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(eye_mat.clone()),
                    Transform::from_xyz(ex, 0.135, 0.155)
                        .with_scale(Vec3::new(0.065, 0.092, 0.024)),
                ))
                .set_parent(head);
        }
        // §1.3 HatSocket: the hat is FROZEN - same meshes, same materials,
        // same local values. The socket sits where the old torso pivot was
        // (world 0.62 standing: 0.63 hip + 0.846 head − 0.856), so every
        // hat piece lands at its exact pre-rebuild world transform.
        let hat_socket = commands
            .spawn((Transform::from_xyz(0.0, -0.856, 0.0), Visibility::default()))
            .set_parent(head)
            .id();
        // hat: brim, crown, band - plus the little antenna
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(hat.clone()),
                Transform::from_xyz(0.0, 1.02, 0.0).with_scale(Vec3::new(0.72, 0.028, 0.72)),
            ))
            .set_parent(hat_socket);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(hat.clone()),
                Transform::from_xyz(0.0, 1.11, 0.0).with_scale(Vec3::new(0.36, 0.18, 0.36)),
            ))
            .set_parent(hat_socket);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.gunmetal.clone()),
                Transform::from_xyz(0.0, 1.045, 0.0).with_scale(Vec3::new(0.365, 0.04, 0.365)),
            ))
            .set_parent(hat_socket);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.steel.clone()),
                Transform::from_xyz(0.13, 1.22, 0.0).with_scale(Vec3::new(0.015, 0.13, 0.015)),
            ))
            .set_parent(hat_socket);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.core_glow.clone()),
                Transform::from_xyz(0.13, 1.30, 0.0).with_scale(Vec3::splat(0.035)),
            ))
            .set_parent(hat_socket);
        // ARMS: shoulder → elbow → wrist off the yoke, every joint a dark
        // ball in a visible gap, white shell segments, mitten hands
        let mut arms = [[Entity::PLACEHOLDER; 3]; 2];
        for (ai, ax) in [(-SHOULDER_X), SHOULDER_X].into_iter().enumerate() {
            let upper = commands
                .spawn((Transform::from_xyz(ax, 0.62, 0.02), Visibility::default()))
                .set_parent(torso)
                .id();
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(joint.clone()),
                    Transform::from_scale(Vec3::new(0.11, 0.10, 0.11)),
                ))
                .set_parent(upper);
            commands
                .spawn((
                    Mesh3d(mesh_upper.clone()),
                    MeshMaterial3d(shell.clone()),
                    Transform::from_xyz(0.0, UPPER_CENTER, 0.0),
                ))
                .set_parent(upper);
            let fore = commands
                .spawn((Transform::from_xyz(0.0, ELBOW_Y, 0.0), Visibility::default()))
                .set_parent(upper)
                .id();
            // §1: the elbow ball is BIG enough to be the bridge
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(joint.clone()),
                    Transform::from_scale(Vec3::splat(ELBOW_R * 2.0)),
                ))
                .set_parent(fore);
            commands
                .spawn((
                    Mesh3d(mesh_fore.clone()),
                    MeshMaterial3d(shell2.clone()),
                    Transform::from_xyz(0.0, FORE_CENTER, 0.0),
                ))
                .set_parent(fore);
            let hand = commands
                .spawn((Transform::from_xyz(0.0, WRIST_Y, 0.0), Visibility::default()))
                .set_parent(fore)
                .id();
            // §1: a dark wrist ball closes the forearm → mitten seam
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(joint.clone()),
                    Transform::from_scale(Vec3::splat(WRIST_R * 2.0)),
                ))
                .set_parent(hand);
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(shell.clone()),
                    Transform::from_xyz(0.0, -0.02, 0.0)
                        .with_scale(Vec3::new(0.13, 0.10, 0.15)),
                ))
                .set_parent(hand);
            arms[ai] = [upper, fore, hand];
        }
        let [arm_l, arm_r] = arms;
        // §1.3: the WEAPON ROOT on the spine - the gun is parented here
        // (muzzle already +Z = body forward) and BOTH hands IK onto its
        // grip sockets. Parenting the gun to a hand is what produces the
        // floating-weapon look; this is the correct dependency direction.
        let weapon_root = commands
            .spawn((Transform::from_xyz(0.10, 0.50, 0.14), Visibility::default()))
            .set_parent(torso)
            .id();
        let mut weapons = [Entity::PLACEHOLDER; N_WEAPONS];
        for (wi, wk) in ALL_WEAPONS.into_iter().enumerate() {
            let model = spawn_weapon_model(commands, kit, wk, is_player, false);
            commands
                .entity(model)
                .insert((Transform::IDENTITY, Visibility::Hidden))
                .set_parent(weapon_root);
            weapons[wi] = model;
        }
        // the always-carried shield, on the left forearm
        let shield = spawn_shield_model(commands, kit);
        commands
            .entity(shield)
            .insert((
                Transform::from_xyz(0.0, -0.12, 0.09)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                Visibility::Hidden,
            ))
            .set_parent(arm_l[1]);
        // the nocked arrow, shown while a bow is drawn
        let bow_arrow = commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.wood.clone()),
                Transform::from_xyz(0.0, -0.12, 0.04)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                    .with_scale(Vec3::new(0.015, 0.015, 0.62)),
                Visibility::Hidden,
            ))
            .set_parent(arm_l[2])
            .id();
        // wearable robot armor
        let armor_rig = spawn_armor_rig(commands, kit);
        commands
            .entity(armor_rig)
            .insert((Transform::IDENTITY, Visibility::Hidden))
            .set_parent(torso);
        commands.entity(root).insert(FighterRig {
            phase: 0.0,
            prev_speed: 0.0,
            accel_lean: 0.0,
            sprint_t: 0.0,
            carry_t: 0.0,
            prev_yaw_vis: f.yaw,
            wr_lag_yaw: 0.0,
            wr_lag_v: 0.0,
            leg_l,
            leg_r,
            torso,
            neck: head,
            arm_l,
            arm_r,
            weapon_root,
            weapons,
            shield,
            bow_arrow,
            armor_rig,
        });
    }
}

fn spawn_health_bars(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    bars: &BarAssets,
    n: usize,
) {
    let bar_back_mesh = meshes.add(Cuboid::new(0.74, 0.11, 0.01));
    let bar_fill_mesh = meshes.add(Cuboid::new(1.0, 0.08, 0.014));
    let bar_armor_mesh = meshes.add(Cuboid::new(1.0, 0.045, 0.016));
    let unlit = |c: Color| StandardMaterial {
        base_color: c,
        unlit: true,
        ..default()
    };
    let back_mat = materials.add(unlit(Color::srgba(0.05, 0.05, 0.07, 0.9)));
    let armor_mat = materials.add(unlit(Color::srgb(0.35, 0.8, 0.95)));
    for i in 0..n {
        let root = commands
            .spawn((Transform::IDENTITY, Visibility::default()))
            .id();
        commands
            .spawn((
                Mesh3d(bar_back_mesh.clone()),
                MeshMaterial3d(back_mat.clone()),
                Transform::IDENTITY,
            ))
            .set_parent(root);
        let fill = commands
            .spawn((
                Mesh3d(bar_fill_mesh.clone()),
                MeshMaterial3d(bars.green.clone()),
                Transform::from_xyz(0.0, 0.0, 0.01),
                BarFill,
            ))
            .set_parent(root)
            .id();
        let afill = commands
            .spawn((
                Mesh3d(bar_armor_mesh.clone()),
                MeshMaterial3d(armor_mat.clone()),
                Transform::from_xyz(0.0, 0.09, 0.01),
                Visibility::Hidden,
                BarFill,
            ))
            .set_parent(root)
            .id();
        commands
            .entity(root)
            .insert(HealthBarVis { index: i, fill, afill });
    }
}

fn spawn_pickup_pads(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    kit: &ModelKit,
    sim: &TdmSim,
) {
    let pad_mesh = meshes.add(Cylinder::new(0.95, 0.08));
    let pad_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.28, 0.35),
        emissive: LinearRgba::new(0.4, 0.9, 1.4, 1.0),
        ..default()
    });
    for (pi, p) in sim.pickups.iter().enumerate() {
        let root = commands
            .spawn((
                Transform::from_xyz(p.pos[0], p.pos[1], p.pos[2]),
                Visibility::default(),
            ))
            .id();
        commands
            .spawn((
                Mesh3d(pad_mesh.clone()),
                MeshMaterial3d(pad_mat.clone()),
                Transform::from_xyz(0.0, 0.04, 0.0),
            ))
            .set_parent(root);
        let item = spawn_pickup_model(commands, kit, p.kind);
        commands.entity(item).set_parent(root);
        commands.entity(root).insert(PickupVis { index: pi, item });
    }
    // checkpoint rings: a glowing circle per forward-spawn point
    let ring_mesh = meshes.add(Cylinder::new(CHECKPOINT_RADIUS, 0.04));
    for (ci, cp) in sim.checkpoints.iter().enumerate() {
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.85, 0.85, 0.9, 0.30),
            emissive: LinearRgba::new(0.7, 0.7, 0.9, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.spawn((
            Mesh3d(ring_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(cp.pos[0], cp.pos[1] + 0.05, cp.pos[2]),
            CheckpointVis { index: ci },
        ));
    }
}

// ------------------------------------------------------------------- setup

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let game = Game {
        sim: TdmSim::new(MatchConfig::default()),
        accum: 0.0,
        rebuild: true, // first rebuild_world pass spawns the cover
        last_t: 0.0,
        pending_jump: false,
        pending_reload: false,
        pending_dodge: false,
        pending_shield: false,
        pending_slot: None,
        pending_cycle_throw: false,
    };
    commands.insert_resource(Selected::default());
    // settings survive restarts now - loaded from config/settings.txt,
    // clamped defaults on any missing/malformed file
    commands.insert_resource(load_settings());
    commands.insert_resource(SpottedEnemies::default());

    // ---- sounds ---------------------------------------------------------
    commands.insert_resource(Sfx {
        shot_glock: asset_server.load("audio/shot_glock.wav"),
        shot_deagle: asset_server.load("audio/shot_deagle.wav"),
        shot_mp5: asset_server.load("audio/shot_mp5.wav"),
        shot_shotgun: asset_server.load("audio/shot_shotgun.wav"),
        shot_ak: asset_server.load("audio/shot_ak.wav"),
        shot_rifle: asset_server.load("audio/shot_rifle.wav"),
        shot_mg: asset_server.load("audio/shot_mg.wav"),
        shot_sniper: asset_server.load("audio/shot_sniper.wav"),
        bow: asset_server.load("audio/bow.wav"),
        spear: asset_server.load("audio/spear.wav"),
        click: asset_server.load("audio/click.wav"),
        shield: asset_server.load("audio/shield.wav"),
        hit: asset_server.load("audio/hit.wav"),
        headshot: asset_server.load("audio/headshot.wav"),
        hurt: asset_server.load("audio/hurt.wav"),
        pickup: asset_server.load("audio/pickup.wav"),
        reload: asset_server.load("audio/reload.wav"),
        jump: asset_server.load("audio/jump.wav"),
        roll: asset_server.load("audio/roll.wav"),
        kill: asset_server.load("audio/kill.wav"),
        win: asset_server.load("audio/win.wav"),
    });
    commands.insert_resource(SfxState::default());

    // ---- lights (ground + walls are per-map: see rebuild_world) --------
    commands.spawn((
        DirectionalLight {
            illuminance: 13_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.5, 0.0)),
        // §2.3: the ONLY light in the game must reach the viewmodel too.
        // Bevy filters lights per view by render layer, and the viewmodel
        // camera is on its own layer - with no RenderLayers here the
        // light defaulted to layer 0 only, so the first-person weapon was
        // lit by AmbientLight alone: flat, shadowless, and visibly a
        // different material from the same gun seen in third person.
        RenderLayers::from_layers(&[0, VIEWMODEL_LAYER]),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.85, 0.82, 0.78),
        brightness: 380.0,
    });

    // ---- shared model kit ----------------------------------------------
    let metal = |c: Color, m: f32, r: f32| StandardMaterial {
        base_color: c,
        metallic: m,
        perceptual_roughness: r,
        ..default()
    };
    // §2.1 four-grey weapon palette: flat-shaded panels - low roughness
    // VARIANCE so the blocks read as hard surfaces, not plastic
    let flat = |hex: u32| StandardMaterial {
        base_color: Color::srgb_u8(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        ),
        metallic: 0.05,
        perceptual_roughness: 0.60,
        ..default()
    };
    let kit = ModelKit {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        cyl: meshes.add(Cylinder::new(0.5, 1.0)),
        ball: meshes.add(Sphere::new(0.5)),
        grey_light: materials.add(flat(0xC8C9CB)),
        grey_mid: materials.add(flat(0x8A8C8F)),
        grey_dark: materials.add(flat(0x3A3C40)),
        grey_black: materials.add(flat(0x1E2024)),
        gunmetal: materials.add(metal(Color::srgb(0.16, 0.17, 0.19), 0.8, 0.45)),
        steel: materials.add(metal(Color::srgb(0.62, 0.64, 0.68), 0.95, 0.30)),
        wood: materials.add(metal(Color::srgb(0.42, 0.28, 0.15), 0.0, 0.85)),
        string: materials.add(metal(Color::srgb(0.85, 0.82, 0.70), 0.0, 0.9)),
        lens: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 1.0),
            emissive: LinearRgba::new(0.4, 1.6, 2.4, 1.0),
            unlit: true,
            ..default()
        }),
        olive: materials.add(metal(Color::srgb(0.32, 0.35, 0.22), 0.2, 0.8)),
        gold: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.80, 0.30),
            metallic: 1.0,
            perceptual_roughness: 0.25,
            emissive: LinearRgba::new(0.6, 0.45, 0.1, 1.0),
            ..default()
        }),
        white: materials.add(metal(Color::srgb(0.92, 0.92, 0.90), 0.1, 0.6)),
        med_glow: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.95, 0.4),
            emissive: LinearRgba::new(0.2, 1.8, 0.4, 1.0),
            unlit: true,
            ..default()
        }),
        armor_dark: materials.add(metal(Color::srgb(0.14, 0.15, 0.18), 0.9, 0.35)),
        core_glow: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.25, 0.15),
            emissive: LinearRgba::new(2.2, 0.35, 0.15, 1.0),
            unlit: true,
            ..default()
        }),
        // the robot's mitten: matte white shell, same as the body (§1.4)
        hand: materials.add(metal(Color::srgb_u8(0xED, 0xEE, 0xF0), 0.0, 0.42)),
        // Task 5.2 (MISSION doc, supersedes Brief VI's gunmetal): a real
        // military walker is olive-drab/khaki, not gray - the art's
        // whole "this is real hardware" read depends on the palette.
        mech_khaki: materials.add(metal(Color::srgb_u8(0x8A, 0x87, 0x70), 0.05, 0.72)),
        mech_khaki_dk: materials.add(metal(Color::srgb_u8(0x5F, 0x5E, 0x52), 0.05, 0.75)),
        mech_khaki_lt: materials.add(metal(Color::srgb_u8(0x9A, 0x93, 0x84), 0.05, 0.65)),
        mech_shadow: materials.add(flat(0x33352F)),
        mech_metal: materials.add(metal(Color::srgb_u8(0x2B, 0x2C, 0x2B), 0.15, 0.45)),
        // §4.2: hazard chevrons - shoulder-pod cover and knee plates
        // ONLY (≤10% of surface; an accent, not a paint job)
        mech_hazard: materials.add(flat(0xD9A916)),
        mech_red: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0xC2, 0x3B, 0x2E),
            emissive: LinearRgba::new(1.6, 0.25, 0.12, 1.0),
            unlit: true,
            ..default()
        }),
    };

    // Fighter rigs, health bars, and pickup pads are match-scoped now
    // (team size and map can change) - they're built by rebuild_world.
    let unlit = |c: Color| StandardMaterial {
        base_color: c,
        unlit: true,
        ..default()
    };
    let green = materials.add(unlit(Color::srgb(0.25, 0.9, 0.35)));
    let orange = materials.add(unlit(Color::srgb(0.95, 0.65, 0.15)));
    let red = materials.add(unlit(Color::srgb(0.92, 0.18, 0.15)));
    commands.insert_resource(BarAssets { green, orange, red });
    commands.insert_resource(kit.clone());

    // ---- shot / impact FX pools ----------------------------------------
    commands.insert_resource(FxAssets {
        tracer_mesh: meshes.add(Cuboid::new(0.02, 0.02, 1.0)),
        tracer_blue: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.9, 1.0),
            emissive: LinearRgba::new(2.0, 2.5, 4.0, 1.0),
            unlit: true,
            ..default()
        }),
        tracer_red: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.7),
            emissive: LinearRgba::new(4.0, 2.2, 1.2, 1.0),
            unlit: true,
            ..default()
        }),
        missile_mesh: meshes.add(Cuboid::new(0.05, 0.05, 1.0)),
        arrow_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.62, 0.44, 0.22),
            ..default()
        }),
        spear_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.31, 0.16),
            metallic: 0.4,
            ..default()
        }),
        decal_mesh: meshes.add(Cuboid::new(0.15, 0.15, 0.02)),
        decal_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.07, 0.06, 0.05),
            unlit: true,
            ..default()
        }),
    });
    commands.insert_resource(TracerPool::default());
    commands.insert_resource(MissilePool::default());
    commands.insert_resource(DecalPool::default());
    commands.insert_resource(DroppedPool::default());
    // ---- §5 throwable FX assets ----------------------------------------
    commands.insert_resource(ThrowPools::default());
    commands.insert_resource(ThrowAssets {
        ball: meshes.add(Sphere::new(0.5)),
        body: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x1E, 0x20, 0x24),
            metallic: 0.4,
            perceptual_roughness: 0.5,
            ..default()
        }),
        smoke: materials.add(StandardMaterial {
            base_color: Color::srgba_u8(0xB8, 0xBC, 0xC0, 235),
            perceptual_roughness: 1.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        fire_mesh: meshes.add(Cylinder::new(1.0, 0.06)),
        fire: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0xE8, 0x87, 0x3F),
            emissive: LinearRgba::new(2.4, 0.9, 0.25, 1.0),
            unlit: true,
            ..default()
        }),
        flashband: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::new(6.0, 6.0, 6.0, 1.0),
            unlit: true,
            ..default()
        }),
    });
    // §8 horde visuals
    commands.insert_resource(ZombiePool::default());
    commands.insert_resource(ZombieAssets {
        moss: materials.add(StandardMaterial {
            base_color: Color::srgb(0.34, 0.42, 0.28),
            perceptual_roughness: 0.9,
            ..default()
        }),
        pale: materials.add(StandardMaterial {
            base_color: Color::srgb(0.66, 0.68, 0.58),
            perceptual_roughness: 0.8,
            ..default()
        }),
        beacon: materials.add(StandardMaterial {
            base_color: Color::srgba(0.3, 0.95, 0.85, 0.55),
            emissive: LinearRgba::new(0.5, 2.4, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    });
    // §10 low-health vignette (under the flash overlay)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.5, 0.02, 0.02, 0.0)),
        GlobalZIndex(20),
        HealthVignette,
    ));
    // §5.3 flash whiteout overlay (UI, quantised alpha steps)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
        GlobalZIndex(40),
        FlashOverlay,
    ));

    // ---- camera ---------------------------------------------------------
    let cam = commands
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: FOV_HIP_DEG.to_radians(),
                ..default()
            }),
            DistanceFog {
                color: Color::srgb(0.58, 0.63, 0.72),
                falloff: FogFalloff::Linear {
                    start: 45.0,
                    end: 130.0,
                },
                ..default()
            },
            Transform::from_xyz(0.0, 3.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
            MainCam,
            // two cameras now exist - the HUD belongs to this one
            IsDefaultUiCamera,
        ))
        .id();

    // ---- §2.3 viewmodel camera: the gun renders on its OWN camera with a
    // FIXED ~55° FOV, so the world FOV zooming on ADS never stretches the
    // viewmodel. It draws over the world (order 1, no clear) and sees only
    // the VIEWMODEL render layer; the world camera never sees that layer.
    let vm_cam = commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov: VM_FOV_DEG.to_radians(),
                ..default()
            }),
            RenderLayers::layer(VIEWMODEL_LAYER),
            Transform::IDENTITY,
        ))
        .set_parent(cam)
        .id();

    // ---- first-person viewmodel: hands + weapon on the camera ----------
    // (the camera looks down its -Z, so the models yaw 180°: muzzle out)
    let vm_root = commands
        .spawn((Transform::IDENTITY, Visibility::Hidden))
        .set_parent(vm_cam)
        .id();
    let vm_arm_mat = materials.add(metal(Color::srgb(0.28, 0.24, 0.20), 0.1, 0.9));
    let mut vm_weapons = [Entity::PLACEHOLDER; N_WEAPONS];
    for (wi, wk) in ALL_WEAPONS.into_iter().enumerate() {
        let model = spawn_weapon_model(&mut commands, &kit, wk, true, true);
        // §2.1 (Brief IV) CS:GO placement: right +0.11, down −0.13,
        // forward 0.32, yawed ~1.5° so the muzzle converges on screen
        // center; long guns exit the frame bottom-right through the stock
        let (tr, extra_rx) = match wk {
            GunKind::Bow => (Vec3::new(-0.10, -0.16, -0.36), 0.0),
            GunKind::Spear => (Vec3::new(0.15, -0.10, -0.28), -0.12),
            GunKind::Glock | GunKind::Deagle => (Vec3::new(0.10, -0.125, -0.30), 0.0),
            _ => (Vec3::new(0.11, -0.13, -0.32), 0.0),
        };
        commands
            .entity(model)
            .insert((
                Transform {
                    translation: tr,
                    rotation: Quat::from_rotation_y(PI + 0.026)
                        * Quat::from_rotation_x(extra_rx),
                    scale: Vec3::splat(0.9),
                },
                Visibility::Hidden,
            ))
            .set_parent(vm_root);
        // robot forearms running from the screen edges to the grips (the
        // weapon models already carry their own fingered hands)
        let fore_z = if wk == GunKind::Bow { -0.02 } else { 0.24 };
        for (fp, frx) in [
            (Vec3::new(0.09, -0.20, -0.16), -0.7_f32),
            (Vec3::new(-0.11, -0.22, fore_z - 0.14), -0.6),
        ] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(vm_arm_mat.clone()),
                    Transform {
                        translation: fp,
                        rotation: Quat::from_rotation_x(frx),
                        scale: Vec3::new(0.075, 0.30, 0.075),
                    },
                ))
                .set_parent(model);
        }
        vm_weapons[wi] = model;
    }
    // the raised shield fills the view when it's up
    let vm_shield = spawn_shield_model(&mut commands, &kit);
    commands
        .entity(vm_shield)
        .insert((
            Transform::from_xyz(-0.10, -0.14, -0.55).with_scale(Vec3::splat(1.1)),
            Visibility::Hidden,
        ))
        .set_parent(vm_root);
    commands.insert_resource(VmRig {
        root: vm_root,
        weapons: vm_weapons,
        shield: vm_shield,
    });

    // ---- §4.2 projectile arc preview: pixel squares spaced by ARC
    // LENGTH, a landing ring + drop-line, and a ±spread cone -------------
    let dot_mesh = meshes.add(Cuboid::new(0.09, 0.09, 0.09)); // pixel square
    let laser_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.2, 0.15),
        emissive: LinearRgba::new(3.0, 0.4, 0.3, 1.0),
        unlit: true,
        ..default()
    });
    let faint_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.25, 0.18, 0.35),
        emissive: LinearRgba::new(1.1, 0.15, 0.10, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let mut dots = Vec::new();
    for _ in 0..24 {
        dots.push(
            commands
                .spawn((
                    Mesh3d(dot_mesh.clone()),
                    MeshMaterial3d(laser_mat.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    let mut cone = Vec::new();
    for _ in 0..16 {
        cone.push(
            commands
                .spawn((
                    Mesh3d(dot_mesh.clone()),
                    MeshMaterial3d(faint_mat.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    let ring = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.45, 0.03))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.2, 0.15, 0.55),
                emissive: LinearRgba::new(2.2, 0.3, 0.2, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::IDENTITY,
            Visibility::Hidden,
        ))
        .id();
    // the vertical drop-line under the marker: reads the distance
    let drop_line = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.03, 1.0, 0.03))),
            MeshMaterial3d(faint_mat.clone()),
            Transform::IDENTITY,
            Visibility::Hidden,
        ))
        .id();
    commands.insert_resource(ArcVis {
        dots,
        cone,
        ring,
        drop_line,
    });
    commands.insert_resource(ArcState::default());

    // ---- §1 (Brief V): grenade pre-aim arc - amber, with a fainter
    // post-bounce run and a landing ring --------------------------------
    let amber_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.75, 0.2),
        emissive: LinearRgba::new(2.6, 1.6, 0.3, 1.0),
        unlit: true,
        ..default()
    });
    let amber_faint = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.75, 0.25, 0.30),
        emissive: LinearRgba::new(0.9, 0.55, 0.12, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let mut gpre = Vec::new();
    for _ in 0..16 {
        gpre.push(
            commands
                .spawn((
                    Mesh3d(dot_mesh.clone()),
                    MeshMaterial3d(amber_mat.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    let mut gpost = Vec::new();
    for _ in 0..8 {
        gpost.push(
            commands
                .spawn((
                    Mesh3d(dot_mesh.clone()),
                    MeshMaterial3d(amber_faint.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    let gring = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.55, 0.03))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.75, 0.2, 0.55),
                emissive: LinearRgba::new(2.0, 1.2, 0.25, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::IDENTITY,
            Visibility::Hidden,
        ))
        .id();
    commands.insert_resource(GrenadeArcVis {
        pre: gpre,
        post: gpost,
        ring: gring,
    });
    // §5.3 (Brief VI): pooled pod-missile visuals (≤ 8 in flight)
    for i in 0..8 {
        commands.spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(kit.mech_red.clone()),
            Transform::from_scale(Vec3::new(0.14, 0.14, 0.34)),
            Visibility::Hidden,
            RocketVis(i),
        ));
    }

    // ---- HUD ------------------------------------------------------------
    commands.spawn((
        Text::new("+"),
        TextFont {
            font_size: 26.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(49.6),
            top: Val::Percent(48.6),
            ..default()
        },
        CrosshairText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.45, 0.35, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(51.6),
            top: Val::Percent(52.2),
            ..default()
        },
        RangeText,
    ));
    // §1.2 (Brief III) contextual prompt line, bottom-center
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 19.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.88, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            bottom: Val::Percent(20.0),
            ..default()
        },
        PromptText,
    ));
    // §9.1 (Brief IV): vertical weapon strip, right screen edge
    for slot in 0..3usize {
        commands.spawn((
            Text::new(""),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.92, 0.93, 0.95, 0.4)),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                top: Val::Percent(40.0 + slot as f32 * 4.6),
                ..default()
            },
            WeaponStripCell(slot),
        ));
    }
    // §7 compass strip
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(0.95, 0.95, 0.98, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(44.0),
            top: Val::Px(34.0),
            ..default()
        },
        CompassText,
    ));
    // §7 stability bracket: two glyphs that ride the live spread
    for (i, ch) in ["[", "]"].into_iter().enumerate() {
        commands.spawn((
            Text::new(ch),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(48.9),
                ..default()
            },
            StabilityBracket(i as u8),
        ));
    }
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(HUD_ANCHORS[2].2[0] * 100.0),
            top: Val::Percent(HUD_ANCHORS[2].2[1] * 100.0),
            ..default()
        },
        HudText,
    ));
    // §3.4: timer + score - TRUE top-center via a full-width centering
    // rail (data-driven top offset from HUD_ANCHORS)
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Percent(HUD_ANCHORS[3].2[1] * 100.0 - 1.5),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 1.0)),
                ScoreTimerText,
            ));
        });
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(-HUD_ANCHORS[4].2[0] * 100.0),
            top: Val::Percent(HUD_ANCHORS[4].2[1] * 100.0),
            ..default()
        },
        FeedText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(40.0),
            top: Val::Percent(58.0),
            ..default()
        },
        HitFeedText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 34.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.85, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(38.0),
            ..default()
        },
        BannerText,
    ));
    // damage-direction strips
    for (idx, node) in [
        (
            0u8,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                ..default()
            },
        ),
        (
            1,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(24.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ),
        (
            2,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                ..default()
            },
        ),
        (
            3,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(24.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ),
    ] {
        commands.spawn((
            node,
            BackgroundColor(Color::srgba(0.85, 0.08, 0.08, 0.0)),
            DmgEdge(idx),
        ));
    }
    // §3 (Brief VI): the four-corner anatomy. BOTTOM-LEFT = vitals
    // (HP largest text on screen, armor beside); BOTTOM-RIGHT = ammo.
    // Anchors come from HUD_ANCHORS - the layout test asserts them.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(HUD_ANCHORS[0].2[0] * 100.0),
                bottom: Val::Percent(-HUD_ANCHORS[0].2[1] * 100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.5)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 34.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.96, 0.98)),
                PanelInfoText,
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(-HUD_ANCHORS[1].2[0] * 100.0),
                bottom: Val::Percent(-HUD_ANCHORS[1].2[1] * 100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.5)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 34.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PanelAmmoText,
            ));
        });
    // scoreboard (TAB)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(24.0),
                top: Val::Percent(16.0),
                width: Val::Percent(52.0),
                padding: UiRect::all(Val::Px(18.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.88)),
            Visibility::Hidden,
            ScoreboardRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ScoreboardText,
            ));
        });

    // ---- AWM scope overlay: full-screen glass, not a zoom -------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            ScopeRoot,
        ))
        .with_children(|p| {
            // black curtains left/right of the lens
            for side in [true, false] {
                let mut n = Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    width: Val::Percent(32.0),
                    height: Val::Percent(100.0),
                    ..default()
                };
                if side {
                    n.left = Val::Px(0.0);
                } else {
                    n.right = Val::Px(0.0);
                }
                p.spawn((n, BackgroundColor(Color::BLACK)));
            }
            // thin curtains top/bottom
            for side in [true, false] {
                let mut n = Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(7.0),
                    ..default()
                };
                if side {
                    n.top = Val::Px(0.0);
                } else {
                    n.bottom = Val::Px(0.0);
                }
                p.spawn((n, BackgroundColor(Color::BLACK)));
            }
            // the lens ring
            p.spawn((
                Node {
                    width: Val::Vh(88.0),
                    height: Val::Vh(88.0),
                    border: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
            ));
            // crosshair lines
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(10.0),
                    width: Val::Px(1.5),
                    height: Val::Percent(80.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            ));
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(20.0),
                    top: Val::Percent(50.0),
                    width: Val::Percent(60.0),
                    height: Val::Px(1.5),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            ));
        });

    // ---- minimap (M / settings to toggle) ------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                bottom: Val::Px(16.0),
                width: Val::Px(MINIMAP_PX),
                height: Val::Px(MINIMAP_PX),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.05, 0.72)),
            MinimapRoot,
        ))
        .with_children(|p| {
            // teammates (max 8) - blue squares
            for i in 0..8 {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(7.0),
                        height: Val::Px(7.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.6, 1.0)),
                    Visibility::Hidden,
                    MinimapDot(i),
                ));
            }
            // §4.3: spotted enemies - red dots, round (not square like
            // teammates/self) so a glance tells friend from foe by
            // shape alone, not just color
            for i in 0..MINIMAP_ENEMY_SLOTS {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(7.0),
                        height: Val::Px(7.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 0.25, 0.2, 1.0)),
                    BorderRadius::all(Val::Px(3.5)),
                    Visibility::Hidden,
                    MinimapEnemyDot(i),
                ));
            }
            // objectives: checkpoints + the hill
            for i in 0..2 {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(11.0),
                        height: Val::Px(11.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BorderRadius::MAX,
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    MinimapCp(i),
                ));
            }
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(9.0),
                    height: Val::Px(9.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.8, 0.4, 1.0, 0.9)),
                BorderRadius::MAX,
                MinimapHill,
            ));
            // YOU: gold, with a facing needle
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(9.0),
                    height: Val::Px(9.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.8, 0.25)),
                BorderRadius::MAX,
                MinimapPlayer,
            ))
            .with_children(|b| {
                b.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(3.2),
                        top: Val::Px(-6.0),
                        width: Val::Px(2.5),
                        height: Val::Px(7.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.95, 0.8, 0.25)),
                ));
            });
        });

    commands.insert_resource(game);
    commands.insert_resource(CamCtl::default());
}

/// The whole battlefield look is map-owned: ground, border walls, cover
/// (styled by what each block is made of), sky/fog tint, and decorations.
/// Rebuilt on map change and on rematch (the crate layout is reseeded).
fn rebuild_world(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    kit: Res<ModelKit>,
    bars: Res<BarAssets>,
    sel: Res<Selected>,
    old: Query<
        Entity,
        Or<(
            With<CoverVis>,
            With<HillVis>,
            With<FighterVis>,
            With<HealthBarVis>,
            With<PickupVis>,
            With<CheckpointVis>,
        )>,
    >,
    mut clear: ResMut<ClearColor>,
    mut fog_q: Query<&mut DistanceFog, With<MainCam>>,
) {
    if !game.rebuild {
        return;
    }
    game.rebuild = false;
    for e in &old {
        commands.entity(e).despawn_recursive();
    }
    // everything match-scoped comes back for the CURRENT sim: fighters
    // (count follows team size), their bars, pads, checkpoint rings
    spawn_fighter_rigs(
        &mut commands,
        &mut meshes,
        &mut materials,
        &kit,
        &game.sim,
        &sel,
    );
    spawn_health_bars(
        &mut commands,
        &mut meshes,
        &mut materials,
        &bars,
        game.sim.fighters.len(),
    );
    spawn_pickup_pads(&mut commands, &mut meshes, &mut materials, &kit, &game.sim);
    let map = game.sim.map;
    // sky / ground / border palettes - the castle maps go green
    let (sky, ground_c, border_c) = match map {
        MapKind::Arena => (
            Color::srgb(0.58, 0.63, 0.72),
            Color::srgb(0.45, 0.40, 0.33),
            Color::srgb(0.55, 0.52, 0.47),
        ),
        MapKind::Bailey => (
            Color::srgb(0.52, 0.66, 0.60),
            Color::srgb(0.30, 0.42, 0.24),
            Color::srgb(0.52, 0.52, 0.50),
        ),
        MapKind::Gardens => (
            Color::srgb(0.55, 0.71, 0.55),
            Color::srgb(0.27, 0.48, 0.24),
            Color::srgb(0.58, 0.56, 0.50),
        ),
        MapKind::Battlefield => (
            Color::srgb(0.60, 0.62, 0.68),
            Color::srgb(0.36, 0.40, 0.28),
            Color::srgb(0.50, 0.50, 0.48),
        ),
    };
    clear.0 = sky;
    if let Ok(mut fog) = fog_q.get_single_mut() {
        fog.color = sky;
    }
    // ground + border walls, sized to THIS map
    let half = game.sim.half;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(half * 2.2, half * 2.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: ground_c,
            perceptual_roughness: 1.0,
            ..default()
        })),
        CoverVis,
    ));
    let border_mat = materials.add(StandardMaterial {
        base_color: border_c,
        perceptual_roughness: 0.85,
        ..default()
    });
    for (pos, size) in [
        (
            Vec3::new(0.0, 2.0, half + 0.25),
            Vec3::new(half * 2.0 + 1.0, 4.0, 0.5),
        ),
        (
            Vec3::new(0.0, 2.0, -half - 0.25),
            Vec3::new(half * 2.0 + 1.0, 4.0, 0.5),
        ),
        (
            Vec3::new(half + 0.25, 2.0, 0.0),
            Vec3::new(0.5, 4.0, half * 2.0 + 1.0),
        ),
        (
            Vec3::new(-half - 0.25, 2.0, 0.0),
            Vec3::new(0.5, 4.0, half * 2.0 + 1.0),
        ),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(border_mat.clone()),
            Transform::from_translation(pos),
            CoverVis,
        ));
    }
    // castle dressing on the border: crenellated battlements + corner drums
    if map != MapKind::Arena {
        let merlon = meshes.add(Cuboid::new(0.9, 0.7, 0.6));
        let mut n = -half + 1.0;
        while n < half {
            for (p, s) in [
                (Vec3::new(n, 4.3, half + 0.25), false),
                (Vec3::new(n, 4.3, -half - 0.25), false),
                (Vec3::new(half + 0.25, 4.3, n), true),
                (Vec3::new(-half - 0.25, 4.3, n), true),
            ] {
                commands.spawn((
                    Mesh3d(merlon.clone()),
                    MeshMaterial3d(border_mat.clone()),
                    Transform::from_translation(p).with_rotation(if s {
                        Quat::from_rotation_y(FRAC_PI_2)
                    } else {
                        Quat::IDENTITY
                    }),
                    CoverVis,
                ));
            }
            n += 2.2;
        }
        let drum = meshes.add(Cylinder::new(2.4, 7.0));
        for sx in [-1.0_f32, 1.0] {
            for sz in [-1.0_f32, 1.0] {
                commands.spawn((
                    Mesh3d(drum.clone()),
                    MeshMaterial3d(border_mat.clone()),
                    Transform::from_xyz(sx * (half + 0.6), 3.5, sz * (half + 0.6)),
                    CoverVis,
                ));
            }
        }
    }
    // cover blocks, styled by what they're made of
    let crate_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.50, 0.40, 0.28),
        perceptual_roughness: 0.9,
        ..default()
    });
    let stone_mat = materials.add(StandardMaterial {
        base_color: match map {
            MapKind::Arena => Color::srgb(0.52, 0.48, 0.42),
            _ => Color::srgb(0.56, 0.56, 0.54),
        },
        perceptual_roughness: 0.95,
        ..default()
    });
    let hedge_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.38, 0.14),
        perceptual_roughness: 1.0,
        ..default()
    });
    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.24, 0.13),
        perceptual_roughness: 1.0,
        ..default()
    });
    let leaf_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.45, 0.16),
        perceptual_roughness: 1.0,
        ..default()
    });
    let leaf_mesh = meshes.add(Sphere::new(1.0));
    for (c, k) in game.sim.cover.iter().zip(game.sim.cover_kind.iter()) {
        let size = Vec3::new(
            c.max[0] - c.min[0],
            c.max[1] - c.min[1],
            c.max[2] - c.min[2],
        );
        let center = Vec3::new(
            (c.min[0] + c.max[0]) * 0.5,
            (c.min[1] + c.max[1]) * 0.5,
            (c.min[2] + c.max[2]) * 0.5,
        );
        let mat = match k {
            CoverKind::Crate => crate_mat.clone(),
            CoverKind::Stone => stone_mat.clone(),
            CoverKind::Hedge => hedge_mat.clone(),
            CoverKind::Tree => trunk_mat.clone(),
        };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(mat),
            Transform::from_translation(center),
            CoverVis,
        ));
        if *k == CoverKind::Tree {
            // crown the trunk with foliage
            for (dy, r) in [(0.4_f32, 1.5_f32), (1.4, 1.0)] {
                commands.spawn((
                    Mesh3d(leaf_mesh.clone()),
                    MeshMaterial3d(leaf_mat.clone()),
                    Transform::from_xyz(center.x, c.max[1] + dy, center.z)
                        .with_scale(Vec3::splat(r)),
                    CoverVis,
                ));
            }
        }
    }
    if game.sim.mode == Mode::Koth {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(HILL_RADIUS, 0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.75, 0.35, 0.95, 0.35),
                emissive: LinearRgba::new(1.2, 0.4, 1.8, 1.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(game.sim.hill[0], game.sim.hill[1] + 0.06, game.sim.hill[2]),
            HillVis,
        ));
    }
}

// ------------------------------------------------------------------- input

#[allow(clippy::too_many_arguments)]
fn input_and_step(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    settings: Res<GameSettings>,
    mut motion: EventReader<MouseMotion>,
    mut cam: ResMut<CamCtl>,
    mut game: ResMut<Game>,
    mut toast: ResMut<Toast>,
    cam_q: Query<&Transform, With<MainCam>>,
) {
    // §3.1: raw input. Every motion event is consumed, the delta is a
    // displacement (never scaled by dt), and there is NO smoothing or
    // acceleration on the angles - any lerp here would be input lag.
    // §3.2: the one multiplier allowed is the zoom sensitivity match,
    // fed the LIVE mid-transition FOV so tracking stays continuous.
    if cam.grabbed {
        // the zoom match is measured against the player's CHOSEN hip FOV,
        // not the old fixed one, or picking a wide FOV would silently
        // change effective ADS sensitivity too
        let zoom_mult = ads_sens_mult(settings.fov_deg().to_radians(), cam.fov_now, ADS_SENS_RATIO);
        let sens = MOUSE_SENS * settings.sens_mult();
        let y_sign = if settings.invert_y { -1.0 } else { 1.0 };
        for ev in motion.read() {
            cam.yaw -= ev.delta.x * sens * zoom_mult;
            cam.pitch = (cam.pitch + ev.delta.y * sens * zoom_mult * y_sign)
                .clamp(-1.53, 1.53);
        }
    } else {
        motion.clear();
    }

    let mut mv = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        mv.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        mv.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        mv.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        mv.x += 1.0;
    }
    let mv = mv.normalize_or_zero();
    let (s, c) = (cam.yaw.sin(), cam.yaw.cos());
    // note the strafe sign: per playtest, A = screen-left, D = screen-right
    let world = Vec2::new(-mv.x * c + mv.y * s, mv.x * s + mv.y * c);

    // §3.3/§5.3 two-stage aim: in first person eye ≈ muzzle and this
    // collapses to the camera ray; in third person it kills the parallax
    // that made shots sail over near cover.
    let aim = if let Ok(cam_tf) = cam_q.get_single() {
        let (dir, blocked) = crosshair_aim_dir(&game.sim, cam_tf);
        cam.blocked = blocked;
        dir
    } else {
        Vec3::Z
    };

    // SPACE jump, Q roll (or tap crouch at a sprint), E shield,
    // O or V first/third person, Z/X lean, 1-3 weapon slots, M minimap.
    // CTRL crouch, C armor ability, T inspect. Edge inputs latch until a
    // sim step runs. (The v6 mapping this comment used to describe -
    // LEFT aim / RIGHT-or-T fire - has been dead since Brief VI; its
    // leftovers were still being printed to the player in three places.)
    // §1 (Brief VI): CS:GO grammar - LEFT fires, always. RIGHT is an ALT
    // function that only exists on scoped weapons (camera zoom, Rule 2)
    // and projectile draws (bow/spear, Brief II grammar). Standard guns
    // have NO aim-down-sights state of any kind. swap_mouse still swaps.
    let (aim_btn, fire_btn) = mouse_map(settings.swap_mouse);
    let p_gun = game.sim.fighters[game.sim.player].gun;
    let scoped_gun = gun(p_gun).scoped;
    let alt_capable = scoped_gun || gun(p_gun).projectile.is_some();
    // §5.2 (Brief VI): scoped-class zoom is a two-stage CYCLE (40° →
    // 10° → out), and EVERY shot auto-unscopes - the bolt is cycled
    // out of the glass
    if scoped_gun && buttons.just_pressed(aim_btn) {
        cam.zoom_stage = (cam.zoom_stage + 1) % 3;
    }
    if !scoped_gun {
        cam.zoom_stage = 0;
    }
    let pf = game.sim.fighters[game.sim.player].fire_cd;
    if scoped_gun && pf > cam.prev_fire_cd + 1e-6 {
        cam.zoom_stage = 0;
    }
    cam.prev_fire_cd = pf;
    let ads = if scoped_gun {
        cam.zoom_stage > 0
    } else {
        buttons.pressed(aim_btn) && alt_capable
    };
    cam.ads = ads;
    // §3.4: ADS progress advances framerate-independently; the sim's ADS
    // benefits (spread ×0.32, ADS walk speed) key off ads_t > 0.9 so
    // frame 1 of the zoom doesn't get full accuracy.
    {
        let dir = if ads { 1.0 } else { -1.0 };
        // §6: Recon Weave aims in 40% faster; §5.2 (Brief VI): the
        // scope transition is 0.05 s per step
        let ads_time = if scoped_gun {
            0.05
        } else if game.sim.fighters[game.sim.player].armor_set == ArmorSet::Recon {
            ADS_TIME_S / 1.4
        } else {
            ADS_TIME_S
        };
        cam.ads_t = (cam.ads_t + dir * time.delta_secs() / ads_time).clamp(0.0, 1.0);
    }
    let ads_settled = ads && cam.ads_t > 0.9;
    if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::KeyO) {
        cam.first_person = !cam.first_person;
        // the toggle CONFIRMS itself - an unseen mode switch reads as
        // a dead key
        toast.text = if cam.first_person {
            "FIRST PERSON  (V: back to third)".to_string()
        } else {
            "THIRD PERSON  (V: first person)".to_string()
        };
        toast.t = 1.8;
    }
    if keys.just_pressed(KeyCode::Space) {
        game.pending_jump = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.pending_reload = true;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        game.pending_shield = true;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        game.pending_cycle_throw = true; // §5: cycle the throwable
    }
    for (key, s) in [
        (KeyCode::Digit1, 0u8),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
    ] {
        if keys.just_pressed(key) {
            game.pending_slot = Some(s);
        }
    }
    // lean: hold Z (left) / X (right) - Phantom-Forces peeking
    let lean = (keys.pressed(KeyCode::KeyX) as i32 - keys.pressed(KeyCode::KeyZ) as i32) as f32;
    let sprinting = keys.pressed(KeyCode::ShiftLeft) && !ads;
    if keys.just_pressed(KeyCode::KeyQ)
        || (sprinting && keys.just_pressed(KeyCode::ControlLeft))
    {
        game.pending_dodge = true;
    }
    let mut cmd = PlayerCmd {
        move_x: world.x,
        move_z: world.y,
        sprint: sprinting,
        yaw: cam.yaw,
        aim: [aim.x, aim.y, aim.z],
        // §2.4 (Brief IV): T is INSPECT now - fire is the mouse alone
        shoot: buttons.pressed(fire_btn),
        reload: game.pending_reload,
        ads: ads_settled,
        // §6: crouch is CTRL only - C now holds the armor ability
        crouch: keys.pressed(KeyCode::ControlLeft),
        jump: game.pending_jump,
        dodge: game.pending_dodge,
        slot: game.pending_slot,
        shield: game.pending_shield,
        lean,
        // §5: hold H (or Mouse4) to cook, release to throw
        throw_hold: keys.pressed(KeyCode::KeyH) || buttons.pressed(MouseButton::Back),
        // §1 (Brief V): B cancels the aimed throw, grenade kept
        throw_cancel: keys.just_pressed(KeyCode::KeyB),
        // §4.6 (Brief VI): U dismounts the mech
        exit_mech: keys.just_pressed(KeyCode::KeyU),
        // §5.3 (Brief VI): Y holds missile targeting; release launches
        pod_aim: keys.pressed(KeyCode::KeyY),
        cycle_throw: game.pending_cycle_throw,
        // §5/§6 (Brief III): F is the KNIFE now; the armor ability
        // (brace / flame / repulsor) moved to held C
        ability: keys.pressed(KeyCode::KeyC),
        knife_hold: keys.pressed(KeyCode::KeyF),
    };

    let prev_fire_cd = game.sim.fighters[game.sim.player].fire_cd;
    game.accum += time.delta_secs().min(0.25);
    let mut steps = 0;
    while game.accum >= DT && steps < 8 {
        game.sim.step(cmd);
        // EDGE commands must fire exactly once per press. The whole `cmd`
        // is replayed for every fixed sub-step, so anything that ADVANCES
        // or TOGGLES state has to be cleared here; only the idempotent
        // ones (jump/reload/dodge/slot - each a "set to this" ) survive a
        // repeat. The shield toggle would flip right back; the throwable
        // cycle would advance 2+ slots in a single G press whenever the
        // frame ran more than one sub-step (i.e. at any framerate below
        // the 120Hz sim, so 60 FPS always).
        cmd.shield = false;
        cmd.cycle_throw = false;
        game.accum -= DT;
        steps += 1;
    }
    if steps > 0 {
        game.pending_jump = false;
        game.pending_reload = false;
        game.pending_dodge = false;
        game.pending_shield = false;
        game.pending_slot = None;
        game.pending_cycle_throw = false;
    }

    // recoil: the camera kicks up when your gun goes off - sustained fire
    // walks the muzzle (the gun is deliberately not stable). A shot is
    // detected by fire_cd jumping up (ammo deltas miss the shot fired on
    // the same tick a reload completes). Leaning braces the shoulder:
    // the CAMERA kick honors the same ×0.8 the sim's bloom gets.
    let p = &game.sim.fighters[game.sim.player];
    if p.alive() && p.fire_cd > prev_fire_cd {
        let spec = gun(p.gun);
        let brace = if p.lean.abs() > 0.1 {
            LEAN_RECOIL_MULT
        } else {
            1.0
        };
        cam.pitch =
            (cam.pitch - (spec.kick * 6.0 + p.bloom * 1.5) * brace).clamp(-0.7, 0.8);
        cam.recoil = (cam.recoil + 0.6).min(1.0);
    }

    // rematch detection: the sim reseeds itself → rebuild cover visuals
    if game.sim.t < game.last_t {
        game.rebuild = true;
    }
    game.last_t = game.sim.t;
}

// ----------------------------------------------------------------- visuals

// ---- §1 (Brief VII): the living-motion layer -----------------------------
// No character is ever a statue. Everything below is RENDER-SIDE and
// additive-only over the existing gait/aim pose - never sim state, never
// the sim's seeded RNG (replay-preservation, same rule as Brief IV's idle
// layer this section extends).

/// A per-fighter deterministic "seed" from its roster index - variety
/// without touching the sim's RNG stream.
fn id_hash(id: u32) -> u32 {
    id.wrapping_mul(2654435761)
}

/// Map an id-hash into a period within `[lo, hi]` seconds.
fn id_period(id: u32, lo: f32, hi: f32) -> f32 {
    lo + (hi - lo) * (id_hash(id) as f32 / u32::MAX as f32)
}

/// §1.1: breathing rate in Hz - 12 cycles/min calm (0.2 Hz) ramping to
/// 30/min (0.5 Hz) as `heat` (0..1, driven by recent sprinting) rises.
fn breath_hz(heat: f32) -> f32 {
    0.2 + 0.3 * heat.clamp(0.0, 1.0)
}

/// The breathing additive offset - always >= 0 (a chest never sinks past
/// neutral on the idle layer), amplitude ~0.5cm peak per the brief.
fn breath_offset(tnow: f32, ph: f32, heat: f32) -> f32 {
    0.0025 * (1.0 + (tnow * std::f32::consts::TAU * breath_hz(heat) + ph).sin())
}

/// §1.1: weight-shift - a slow, per-fighter-period sway (6-12s period,
/// deterministic from id_hash), silenced by gait amplitude so it only
/// reads while the fighter is genuinely standing still.
fn weight_shift(tnow: f32, ph: f32, period_s: f32, gait_amp: f32) -> f32 {
    0.03 * (tnow * std::f32::consts::TAU / period_s + ph * 1.3).sin() * (1.0 - gait_amp.min(1.0))
}

/// §1.1: grip fidget - a brief re-grip BLIP once per `period_s` (8-15s),
/// a short window eased in and out, silent the rest of the period.
fn grip_fidget(tnow: f32, ph: f32, period_s: f32) -> f32 {
    let window = 0.35;
    let phase = (tnow + ph * 3.1).rem_euclid(period_s);
    if phase < window {
        (phase / window * PI).sin() * 0.05
    } else {
        0.0
    }
}

/// §1.1: head-glance - every ~4s, a short ~1.1s window where the head
/// eases toward `target_yaw` (clamped +/-25 degrees) instead of resting
/// neutral; silent the rest of the period, so it reads as a GLANCE, not
/// a lock-on stare.
fn head_glance(tnow: f32, ph: f32, target_yaw: f32) -> f32 {
    const PERIOD: f32 = 4.0;
    const WINDOW: f32 = 1.1;
    let phase = (tnow + ph * 1.7).rem_euclid(PERIOD);
    if phase < WINDOW {
        let e = (phase / WINDOW * PI).sin();
        target_yaw.clamp(-0.436, 0.436) * e
    } else {
        0.0
    }
}

/// Per-fighter persistent state the pure functions above can't carry on
/// their own - decaying reaction timers and edge-detected counters. All
/// cosmetic/render-only; never touches `sim.rs`.
#[derive(Clone)]
struct LifeState {
    /// 0..1, ramps toward 1 while sprinting (~1.5s), decays over 8s once
    /// it stops - drives the breathing-rate ramp/decay from §1.1.
    breath_heat: f32,
    prev_deaths_seen: u32,
    prev_player_kills: u32,
    /// §1.3: a nearby projectile passed close - an involuntary flinch.
    suppress_t: f32,
    /// §1.3: a nearby explosion - a stronger shield-eyes flinch.
    boom_flinch_t: f32,
    /// §1.3: an ally died nearby - head snaps toward them.
    ally_snap_t: f32,
    ally_snap_yaw: f32,
    /// §1.3: a kill was just confirmed - a subtle exhale + re-grip.
    exhale_t: f32,
    /// §1.2: seconds since this fighter last did anything combat-shaped;
    /// Relaxed posture eases in once this clears 10s.
    since_combat: f32,
    /// Task 3.3 sprint-start: the head's lagged copy of the acceleration
    /// lean - the chain ripple's tip. See `chain_lag_chase`.
    lean_lag: f32,
    /// §5.2 (Brief VII v2): the LEGS' own facing, which LAGS the aim.
    ///
    /// The sim keeps `f.yaw` locked to the aim every tick, so body and
    /// aim were always identical and there was no separation to show -
    /// which is why `torso_aim_offset` sat built, tested and called from
    /// nowhere. This is the missing half, and it is COSMETIC (C3): the
    /// legs visually catch up to an aim the sim already considers
    /// authoritative, and nothing here is ever written back.
    ///
    /// NaN-safe sentinel: f32::NAN means "not initialised", so a fresh
    /// or respawned fighter snaps to its facing instead of spinning up
    /// from 0.
    leg_yaw: f32,
    /// Task 3.3: seconds since this fighter's spear throw/thrust ENDED -
    /// the follow-through clock for `spear_followthrough_yaw`.
    /// NEGATIVE means no release has happened yet, which is the state a
    /// fresh or just-respawned fighter must start in: the follow-through
    /// now begins at the release yaw (0.35 rad), so a 0.0 default would
    /// make every fighter who merely PICKS UP a spear spawn mid-unwind
    /// with a 20-degree torso twist.
    spear_release_t: f32,
}

impl Default for LifeState {
    fn default() -> Self {
        LifeState {
            breath_heat: 0.0,
            prev_deaths_seen: 0,
            prev_player_kills: 0,
            suppress_t: 0.0,
            boom_flinch_t: 0.0,
            ally_snap_t: 0.0,
            ally_snap_yaw: 0.0,
            exhale_t: 0.0,
            since_combat: 0.0,
            lean_lag: 0.0,
            leg_yaw: f32::NAN, // uninitialised: snap on the first frame
            spear_release_t: -1.0, // no throw/thrust has happened yet
        }
    }
}

/// §5.2: how fast the legs turn to catch up with the aim, rad/s. Slower
/// than a mouse flick on purpose - that gap IS the turn-in-place.
const LEG_TURN_RATE: f32 = 6.5;

/// §5.2: advance the legs' facing toward `aim_yaw` and return
/// `(leg_yaw, torso_offset)`. The torso offset is what puts the shoulders
/// back on the aim, clamped by `torso_aim_offset` - past the clamp the
/// legs simply have to catch up, which is the whole mechanic.
fn step_leg_yaw(prev: f32, aim_yaw: f32, dt: f32) -> (f32, f32) {
    if !prev.is_finite() {
        return (aim_yaw, 0.0); // first frame / respawn: snap, never spin
    }
    let delta = wrap_pi(aim_yaw - prev);
    // The legs ALWAYS square up toward the aim - they just do it slowly.
    // (Only moving them once the gap exceeds the clamp leaves a standing
    // 60deg twist forever, which is both anatomically absurd and what
    // this function's first version actually did.)
    let mut step = (LEG_TURN_RATE * dt).min(delta.abs());
    // ...but the torso can only cover +/-60deg, so if the gap is wider
    // than that the legs must move at least enough to close the excess
    // this frame - otherwise the shoulders could not reach the aim.
    let clamp = TORSO_AIM_LIMIT_DEG.to_radians();
    let must_close = (delta.abs() - clamp).max(0.0);
    step = step.max(must_close).min(delta.abs());
    let leg = prev + delta.signum() * step;
    // the torso covers whatever gap is left, clamped
    let off = torso_aim_offset(wrap_pi(aim_yaw - leg).to_degrees()).to_radians();
    (leg, off)
}

/// Wrap to (-PI, PI]. Turning 350 degrees left is really 10 right.
fn wrap_pi(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut x = a % tau;
    if x > std::f32::consts::PI {
        x -= tau;
    } else if x < -std::f32::consts::PI {
        x += tau;
    }
    x
}

fn sync_fighters(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    mut life: Local<Vec<LifeState>>,
    mut roots: Query<(&FighterVis, &mut FighterRig, &mut Transform, &mut Visibility)>,
    mut parts: Query<(&mut Transform, &mut Visibility), Without<FighterVis>>,
) {
    let dt = time.delta_secs();
    if life.len() < game.sim.fighters.len() {
        life.resize(game.sim.fighters.len(), LifeState::default());
    }
    for (vis, mut rig, mut tf, mut root_vis) in &mut roots {
        // on the deploy frame the OLD rigs still exist while the sim may
        // already be smaller (8v8 → 5v5): index safely, never panic
        let Some(f) = game.sim.fighters.get(vis.index) else {
            *root_vis = Visibility::Hidden;
            continue;
        };
        let is_player = vis.index == game.sim.player;
        // first person - or looking through the AWM's glass, which parks
        // the camera at the eye in EITHER mode: your own body leaves the
        // frame (hands take over)
        // §5: the model swap rides the blend MIDPOINT, not the toggle -
        // no frame of the inside of your own head
        let self_view = cam_ctl.person_t < 0.5
            || (cam_ctl.ads && gun(f.gun).scoped && !f.shield_up);
        *root_vis = if is_player && self_view && f.alive() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if !f.alive() {
            // dead: lie flat until respawn - the chibi body is ~0.3 wide,
            // so only a small sink keeps the corpse visibly ON the ground
            tf.translation = Vec3::new(f.pos[0], f.pos[1] - 0.10, f.pos[2]);
            tf.rotation = Quat::from_rotation_y(f.yaw)
                * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            // Task 3.3: clear the follow-through clock so the fighter does
            // not respawn mid-unwind from the throw they died during.
            life[vis.index].spear_release_t = -1.0;
            // §5.2: and un-initialise the legs, so a fighter respawning
            // to a new facing SNAPS to it instead of visibly spinning
            // around from wherever the corpse was pointing.
            life[vis.index].leg_yaw = f32::NAN;
            continue;
        }
        let rolling = f.roll_t > 0.0;
        let in_mech = f.armor_set == ArmorSet::RobotSuit && f.hull > 0.0;
        // §5.2: the legs LAG the aim; the torso covers the difference, up
        // to +/-60deg, and past that the legs have to catch up - which is
        // the turn-in-place. Computed here because the root rotation
        // below consumes it. A mech turns as ONE piece (its own capped
        // turn rate is already the commitment), so it never splits.
        let (leg_yaw, torso_aim) = if in_mech {
            (f.yaw, 0.0)
        } else {
            let prev = life[vis.index].leg_yaw;
            let (l, o) = step_leg_yaw(prev, f.yaw, dt);
            life[vis.index].leg_yaw = l;
            (l, o)
        };
        if rolling && in_mech {
            // §2 (Brief V): the mech does NOT tumble - the side-step is a
            // braced lean into the travel direction, tall the whole way
            let total = MECH_STEP_S + ROLL_EASE_S;
            let e = ((total - f.roll_t) / 0.08).clamp(0.0, 1.0)
                * (f.roll_t / 0.10).clamp(0.0, 1.0); // in fast, out easing
            let step_yaw = f.roll_dir[0].atan2(f.roll_dir[1]);
            tf.translation = Vec3::new(f.pos[0], f.pos[1], f.pos[2]);
            tf.rotation = Quat::from_rotation_y(f.yaw)
                * Quat::from_axis_angle(
                    // lean about the axis PERPENDICULAR to travel
                    Vec3::new((step_yaw - f.yaw).cos(), 0.0, -(step_yaw - f.yaw).sin()),
                    0.16 * e,
                );
        } else if rolling {
            // the somersault: a full forward tumble about the ball's center,
            // facing the roll direction. §2 (Brief V): progress runs over
            // the WHOLE load → burst → ease timeline, so the coil begins
            // slow, the tumble carries, and the uncurl lands with the ease.
            let total = ROLL_LOAD_S + ROLL_S + ROLL_EASE_S;
            let prog = (1.0 - f.roll_t / total).clamp(0.0, 1.0);
            let roll_yaw = f.roll_dir[0].atan2(f.roll_dir[1]);
            // pivot at the BALL's center (0.6 up) with the body curled
            // tight around it - the head must orbit, not plow the floor
            tf.translation = Vec3::new(f.pos[0], f.pos[1] + 0.6, f.pos[2]);
            tf.rotation = Quat::from_rotation_y(roll_yaw)
                * Quat::from_rotation_x(ease_out(prog) * std::f32::consts::TAU);
        } else if f.flip_t > 0.0 {
            // §4: the aerial flip - a full rotation about the body's
            // CENTER (the head must orbit, not sweep the floor)
            let prog = (1.0 - f.flip_t / FLIP_S).clamp(0.0, 1.0);
            let spin = prog * std::f32::consts::TAU;
            let rot = match f.flip_kind {
                0 => Quat::from_rotation_x(spin),
                1 => Quat::from_rotation_x(-spin),
                2 => Quat::from_rotation_z(spin),
                _ => Quat::from_rotation_z(-spin),
            };
            let full = Quat::from_rotation_y(f.yaw) * rot;
            let center = Vec3::new(f.pos[0], f.pos[1] + 0.9, f.pos[2]);
            tf.translation = center - full * (Vec3::Y * 0.9);
            tf.rotation = full;
        } else {
            tf.translation = Vec3::new(f.pos[0], f.pos[1], f.pos[2]);
            // lean tilts the whole body sideways off the hips (positive
            // lean = peek screen-right = roll clockwise)
            let lean_roll = Quat::from_rotation_z(f.lean * 0.24);
            // Task 5.1 (MISSION doc): the mech stands hull-forward, hips
            // high and set back, leaning INTO its own mass - a level
            // upright hull loses the silhouette entirely. A soldier's
            // rotation is untouched (mech_pitch is 0 off-mech).
            let mech_pitch = if f.armor_set == ArmorSet::RobotSuit && f.hull > 0.0 {
                0.085
            } else {
                0.0
            };
            tf.rotation = tf.rotation.slerp(
                Quat::from_rotation_y(leg_yaw) * lean_roll * Quat::from_rotation_x(mech_pitch),
                0.35,
            );
        }
        // §11: the MECH is the same rig at walker scale - unmistakable
        tf.scale = Vec3::splat(
            if f.armor_set == ArmorSet::RobotSuit && f.hull > 0.0 {
                MECH_SCALE
            } else {
                1.0
            },
        );
        // spawn-protection shimmer: bob slightly
        if f.protect_t > 0.0 {
            tf.translation.y += (game.sim.t * 14.0).sin() * 0.02;
        }
        let speed = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
        // §1.4: phase is driven by DISTANCE, never by time - feet cannot
        // slide at any speed because stride length is the integral
        if speed > 0.1 && !rolling && f.grounded {
            rig.phase += speed * dt * PI / STRIDE_M;
        }
        // §1.4 accel lean: sells starts and stops (smoothed, capped so
        // the head stays in band - the cap lives in gait_pose)
        let accel = (speed - rig.prev_speed) / dt.max(1e-4);
        rig.prev_speed = speed;
        let lean_target = (accel * 0.012).clamp(-0.07, 0.07);
        rig.accel_lean += (lean_target - rig.accel_lean) * (8.0 * dt).min(1.0);
        // move direction in body space: forward + lateral fractions drive
        // the swing AXIS, so strafing reads as a crossover step instead of
        // the forward cycle rotated sideways
        let (fx, fz) = (f.yaw.sin(), f.yaw.cos());
        let (fwd_c, lat_c) = if speed > 0.3 {
            (
                (f.vel[0] * fx + f.vel[1] * fz) / speed,
                (f.vel[0] * fz - f.vel[1] * fx) / speed,
            )
        } else {
            (1.0, 0.0)
        };
        let swing_axis = Vec3::new(fwd_c, 0.0, -lat_c).normalize_or(Vec3::X);
        let amp = (speed / SPRINT_SPEED).clamp(0.0, 1.0)
            * 0.6
            * if f.crouch { 0.6 } else { 1.0 };
        let swing = rig.phase.sin() * amp;
        let airborne = !f.grounded;
        // just-fired jerk, driven by the sim's own cooldown
        let spec = gun(f.gun);
        let jerk = if f.armed() && spec.fire_period > 0.0 {
            ((f.fire_cd / spec.fire_period).clamp(0.0, 1.0)).powi(3)
        } else {
            0.0
        };
        // §2 (Brief V): the landing SETTLES - a weight-absorb dip in the
        // first 0.20 s after a roll/side-step ends, easing back up. The
        // window is read straight off the cooldown clock: no new state.
        // Computed BEFORE the pose so it can go through `gait_pose`,
        // which is what keeps the §0.2 band test sampling the same hip
        // the renderer writes.
        let settle = if !rolling && f.roll_t <= 0.0 {
            let cd_base = if in_mech { MECH_STEP_CD_S } else { ROLL_CD_S };
            ((f.roll_cd - (cd_base - ROLL_SETTLE_S)) / ROLL_SETTLE_S).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // §1.4 pose core - the SAME pure function the §0.2 band test
        // samples, so the render cannot drift out of the hit bands
        let (hip_y, torso_pitch_base) = if rolling {
            (0.25, 1.0) // curled tight around the tumble pivot
        } else if airborne {
            (0.63, 0.10)
        } else {
            gait_pose(f.crouch, rig.phase, amp, rig.accel_lean, settle)
        };
        // legs: thigh (hip), shin (knee), foot (ankle) per side.
        // Sign convention: +X rotation swings the limb BACKWARD, so knees
        // fold positive and a raised thigh is negative.
        for (leg, off) in [(rig.leg_l, 0.0_f32), (rig.leg_r, PI)] {
            let (thigh, shin, foot);
            if rolling {
                // tucked ball
                thigh = -1.55;
                shin = 2.30;
                foot = -0.55;
            } else if f.crouch && !airborne {
                // half-squat under the §0.2-tuned crouch lean
                thigh = -0.85;
                shin = 1.55;
                foot = -0.42;
            } else if airborne {
                // jump tuck
                thigh = -0.85;
                shin = 1.50;
                foot = -0.40;
            } else {
                // the gait: elliptical foot path (60% stance / 40% swing
                // read), knee bends hardest on recovery, and the ankle
                // rolls heel-strike → toe-off - the *human* read
                let sw = (rig.phase + off).sin() * amp;
                let lift = (rig.phase + off + 0.9).sin().max(0.0) * amp;
                thigh = sw * 0.9;
                shin = 0.10 + lift * 1.25;
                foot = -(thigh + shin) * 0.55
                    + (rig.phase + off - 1.2).sin() * 0.20 * amp
                    + (rig.phase + off + 0.6).sin() * 0.22 * amp; // ankle roll
            }
            if let Ok((mut t, _)) = parts.get_mut(leg[0]) {
                t.translation.y = hip_y;
                t.rotation = if rolling || f.crouch || airborne {
                    Quat::from_rotation_x(thigh)
                } else {
                    // swing in the plane of MOVEMENT (crossover strafe)
                    Quat::from_axis_angle(swing_axis, thigh)
                };
            }
            if let Ok((mut t, _)) = parts.get_mut(leg[1]) {
                t.rotation = Quat::from_rotation_x(shin);
            }
            if let Ok((mut t, _)) = parts.get_mut(leg[2]) {
                t.rotation = Quat::from_rotation_x(foot);
            }
        }
        // §3.4: recoil is a BODY event with staggered peaks - the wrist
        // (weapon root, jerk³) spikes first, the torso rocks back ~1.2°
        // on a slower curve and settles last
        let torso_pitch = torso_pitch_base - jerk.powf(0.4) * 0.021;
        // §3.2 the Achilles throw: windup rotates the torso 42° AWAY,
        // the plant blocks, the hips fire open through release, and the
        // follow-through carries past - sequencing, not detail. §2
        // (MISSION doc rig audit): this IS the hip-shoulder separation -
        // root (legs) carries `f.yaw` alone, torso is root's CHILD with
        // this as its own additive local yaw, so torso_world - root_world
        // = this value exactly. Extracted so it's independently testable.
        // Task 3.3 real consumer: the follow-through clock (client-side
        // only, per §1.3's `LifeState` pattern) so the curve is driven by
        // the actual kinetic chain rather than the gun-recoil `jerk`
        // proxy it used before. Held at 0 for as long as the action is
        // live, then counts up from the frame it ends - so the
        // follow-through always starts at the release yaw. A negative
        // value (the default, and what death restores) means "nothing
        // thrown yet", which keeps the curve silent.
        {
            let spear_active =
                f.gun == GunKind::Spear && (f.spear_wind_t > 0.0 || f.knife_phase > 0.0);
            let ls0 = &mut life[vis.index];
            if spear_active {
                ls0.spear_release_t = 0.0;
            } else if ls0.spear_release_t >= 0.0 {
                ls0.spear_release_t += dt;
            }
        }
        let spear_release_t = life[vis.index].spear_release_t;
        let spear_yaw = torso_coil_yaw(f.gun, f.spear_wind_t, f.knife_phase, in_mech, spear_release_t);
        // §1 (Brief VII), extending §4 (Brief IV): the living-motion
        // layer - breathing, weight shift, micro-sway, grip fidget, and
        // reactions (suppression/explosion flinch, ally-death snap, kill
        // exhale). Phase offsets come from a RENDER-side hash of the
        // fighter index (twenty soldiers desync for free) - NEVER the
        // sim RNG, or replays break. All additive-only over the gait
        // pose, and clamped, so the head band stays law by construction.
        let ph = vis.index as f32 * 2.399;
        let tnow = game.sim.t;
        let ls = &mut life[vis.index];
        // combat = fired, took damage, or (for bots) is presently armed
        // and moving with intent; Relaxed posture eases in past 10s clear
        let in_combat = f.fire_cd > 0.0 || tnow - f.last_dmg_at < 2.0;
        if in_combat {
            ls.since_combat = 0.0;
        } else {
            ls.since_combat += dt;
        }
        let relaxed_e = ((ls.since_combat - 10.0) / 2.0).clamp(0.0, 1.0);
        // breathing rate ramps toward 30/min while sprinting, decays
        // back to 12/min over 8s once it stops
        if speed > 4.0 {
            ls.breath_heat = (ls.breath_heat + dt / 1.5).min(1.0);
        } else {
            ls.breath_heat = (ls.breath_heat - dt / 8.0).max(0.0);
        }
        // §1.3 reactions: a nearby explosion or a projectile passing
        // close both read as an involuntary flinch; an ally dying nearby
        // snaps the head toward them; a confirmed kill gets a small
        // exhale + re-grip. All edge-detected against last frame's count
        // so each event fires exactly once.
        let boom_near = game
            .sim
            .booms
            .iter()
            .any(|(b, ttl)| {
                *ttl > 1.7
                    && (Vec3::from_array(b.at) - Vec3::new(f.pos[0], f.pos[1], f.pos[2]))
                        .length()
                        < 8.0
            });
        if boom_near {
            ls.boom_flinch_t = 0.4;
        } else {
            ls.boom_flinch_t = (ls.boom_flinch_t - dt).max(0.0);
        }
        let missile_near = game.sim.missiles.iter().any(|m| {
            (Vec3::from_array(m.pos) - Vec3::new(f.pos[0], f.pos[1], f.pos[2])).length() < 2.0
        });
        if missile_near {
            ls.suppress_t = 0.2;
        } else {
            ls.suppress_t = (ls.suppress_t - dt).max(0.0);
        }
        let deaths_now: u32 = game
            .sim
            .fighters
            .iter()
            .enumerate()
            .filter(|(j, g)| *j != vis.index && g.team == f.team)
            .map(|(_, g)| g.deaths)
            .sum();
        if deaths_now > ls.prev_deaths_seen {
            if let Some((_, dead)) = game
                .sim
                .fighters
                .iter()
                .enumerate()
                .filter(|(j, g)| *j != vis.index && g.team == f.team)
                .map(|(_, g)| g)
                .filter(|g| !g.alive())
                .map(|g| {
                    let d = ((g.pos[0] - f.pos[0]).powi(2) + (g.pos[2] - f.pos[2]).powi(2)).sqrt();
                    (d, g)
                })
                .filter(|(d, _)| *d < 6.0)
                .min_by(|a, b| a.0.total_cmp(&b.0))
            {
                ls.ally_snap_t = 0.5;
                let dx = dead.pos[0] - f.pos[0];
                let dz = dead.pos[2] - f.pos[2];
                ls.ally_snap_yaw = dx.atan2(dz) - f.yaw;
            }
        }
        ls.prev_deaths_seen = deaths_now;
        ls.ally_snap_t = (ls.ally_snap_t - dt).max(0.0);
        if is_player && f.kills > ls.prev_player_kills {
            ls.exhale_t = 0.6;
        }
        ls.prev_player_kills = f.kills;
        ls.exhale_t = (ls.exhale_t - dt).max(0.0);
        let breath = breath_offset(tnow, ph, ls.breath_heat)
            + if ls.exhale_t > 0.0 { 0.004 * (ls.exhale_t / 0.6) } else { 0.0 };
        let wshift = weight_shift(tnow, ph, id_period(vis.index as u32, 6.0, 12.0), amp)
            - relaxed_e * 0.02;
        let sway_r = 0.007 * (tnow * 0.94 + ph).sin();
        let flinch = {
            let age = tnow - f.last_dmg_at;
            let dmg = if (0.0..0.3).contains(&age) {
                0.06 * (1.0 - age / 0.3)
            } else {
                0.0
            };
            // §5.5: BURNING is a continuous panic, not a single flinch -
            // a fast irregular shudder for as long as the fighter is
            // alight. `burn_t` is set by fire pools and the flame
            // projector and was, until this, written by the sim and read
            // by nothing at all despite its doc claiming a client FX.
            let burn = if f.burn_t > 0.0 {
                0.035 * (tnow * 27.0 + ph * 5.0).sin().abs() * f.burn_t.min(1.0)
            } else {
                0.0
            };
            dmg + burn + ls.boom_flinch_t * 0.5 + ls.suppress_t * 0.3
        };
        if let Ok((mut t, _)) = parts.get_mut(rig.torso) {
            // §1.4 pelvis layers: lateral sway toward the stance foot,
            // pelvis yaw with the spine counter-rotating most of it back
            // (net ±1.5° - which is also all the arm swing an armed
            // carry gets: the upper body moves through the spine, not
            // the shoulders)
            t.translation.y =
                hip_y - if f.crouch && !rolling { 0.12 } else { 0.0 } + breath;
            // amplitudes tuned UP so the gait reads from the 2.6 m boom
            t.translation.x = 0.048 * rig.phase.sin() * amp + wshift;
            // §5.2: `torso_aim` puts the shoulders back on the aim that
            // the legs have not caught up to yet - the turn-in-place read
            t.rotation = Quat::from_rotation_y(
                0.045 * rig.phase.sin() * amp + spear_yaw + torso_aim,
            )
                * Quat::from_rotation_x(
                    (torso_pitch + flinch + 0.07 * settle + relaxed_e * 0.05).min(0.185),
                )
                * Quat::from_rotation_z(sway_r);
        }
        // §1.1 the neck aims: head pitch follows the view (×0.75, capped
        // so head geometry stays inside its band), minus part of the
        // torso pitch so the head reads as LOOKING, not nodding along
        let aim_pitch_view = if is_player { cam_ctl.pitch } else { 0.05 };
        if let Ok((mut t, _)) = parts.get_mut(rig.neck) {
            // Task 3.3 sprint-start: the head is the chain's LAST
            // segment - it arrives at a new acceleration lean one
            // tip-onset behind the pelvis, so a hard start reads
            // hips-first with the head trailing, and a hard stop whips
            // it forward. Transient only: at steady lean the difference
            // is zero. Band-safe: a rotation about the neck pivot does
            // not move the head's lowest geometry, which is what the
            // §0.2 band measures.
            ls.lean_lag = chain_lag_chase(ls.lean_lag, rig.accel_lean, dt);
            let chain_lag_rx = ((rig.accel_lean - ls.lean_lag) * 1.6).clamp(-0.12, 0.12);
            let head_rx = (aim_pitch_view * 0.75).clamp(-0.55, 0.55)
                - torso_pitch.min(0.30)
                + chain_lag_rx;
            // §2.2 cheek weld: on ADS the HEAD comes to the stock - ~6°
            // tilt and 3 cm toward the shoulder. The weapon does not
            // float up to a stationary head. (Previous-frame blend.)
            let weld = ease_out(rig.carry_t)
                * if f.armed() && !f.shield_up && f.gun != GunKind::Bow {
                    1.0
                } else {
                    0.0
                };
            // §3.4: eyes and head FOLLOW the magazine through a reload
            let reload_look = if f.reload_t > 0.0 { 0.22 } else { 0.0 };
            // §1.1 (Brief VII): idle head-look - glances toward the
            // nearest MOVING entity every ~4s, never while anything is
            // actually happening (aiming, firing, reloading, moving).
            let scan = if speed < 0.5 && f.fire_cd <= 0.0 && f.reload_t <= 0.0 {
                if ls.ally_snap_t > 0.0 {
                    // §1.3: an ally just died nearby - the head snaps and
                    // holds, overriding the idle glance
                    ls.ally_snap_yaw.clamp(-0.7, 0.7) * (ls.ally_snap_t / 0.5)
                } else if tnow - f.last_dmg_at > 4.0 {
                    let mut nearest = None;
                    let mut best_d = 20.0_f32;
                    for (j, g) in game.sim.fighters.iter().enumerate() {
                        if j == vis.index || !g.alive() {
                            continue;
                        }
                        let gs = (g.vel[0] * g.vel[0] + g.vel[1] * g.vel[1]).sqrt();
                        if gs < 0.3 {
                            continue; // only MOVING entities draw the eye
                        }
                        let d = ((g.pos[0] - f.pos[0]).powi(2) + (g.pos[2] - f.pos[2]).powi(2))
                            .sqrt();
                        if d < best_d {
                            best_d = d;
                            nearest = Some(g);
                        }
                    }
                    let target_yaw = nearest
                        .map(|g| {
                            let dx = g.pos[0] - f.pos[0];
                            let dz = g.pos[2] - f.pos[2];
                            dx.atan2(dz) - f.yaw
                        })
                        .unwrap_or(0.35 * (tnow * 0.55 + ph).sin());
                    head_glance(tnow, ph, target_yaw)
                } else {
                    0.0
                }
            } else {
                0.0
            };
            t.translation = Vec3::new(0.03 * weld, 0.846, 0.0);
            t.rotation = Quat::from_rotation_y(scan)
                * Quat::from_rotation_z(-0.105 * weld)
                * Quat::from_rotation_x(head_rx + reload_look);
        }
        // ---- arms + the held weapon --------------------------------------
        // Jointed chains now: shoulder (free) → elbow (hinge, no
        // inversion) → wrist. The pose solver keeps the WEAPON's pitch on
        // the aim line while the elbow carries a visible bend: shoulder
        // gets (aim + bend), elbow gets (−bend), so the chain sum tracks
        // the crosshair - the arm-IK contract of §1.
        let aim_pitch = if is_player { cam_ctl.pitch * 0.9 } else { 0.05 };
        let slot = weapon_slot(f.gun);
        for (wi, we) in rig.weapons.iter().enumerate() {
            if let Ok((_, mut v)) = parts.get_mut(*we) {
                // the bow shares the left hand with the shield - a raised
                // shield stows it
                let show = slot == Some(wi)
                    && !(f.shield_up && ALL_WEAPONS[wi] == GunKind::Bow);
                *v = if show {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
        // ---- §1.3/§2.3: the weapon root leads; the hands follow ---------
        // Weapon-root pitch tracks the aim line (compensating the torso),
        // the sprint low-ready drops it, and the spine counter-rotation
        // above feeds through it - that's the whole "tactical carry".
        let sprinting =
            speed > SPRINT_SPEED * 0.85 && f.armed() && !(is_player && cam_ctl.ads);
        {
            let dir = if sprinting { 1.0 } else { -1.0 };
            let rate = if sprinting { 0.22 } else { 0.14 }; // fast OUT
            rig.sprint_t = (rig.sprint_t + dir * dt / rate).clamp(0.0, 1.0);
        }
        let wr_pitch = aim_pitch - torso_pitch;
        let spear_cocked =
            f.gun == GunKind::Spear && if is_player { cam_ctl.ads } else { true };
        let bow_draw = if f.gun == GunKind::Bow {
            if is_player {
                if cam_ctl.ads {
                    1.0
                } else {
                    0.25
                }
            } else {
                0.6
            }
        } else {
            0.0
        };
        // §2.2 carry states: a soldier is recognisable by how the weapon
        // is held when NOT shooting. Aimed blends in over 220 ms and out
        // over 140 ms; un-aimed carry is low-ready when contact is live
        // (recent fire or recent damage), patrol otherwise.
        let aiming = if is_player {
            cam_ctl.ads
        } else {
            f.los_time > 0.0
        };
        {
            let dir = if aiming { 1.0 } else { -1.0 };
            let rate = if aiming { 0.22 } else { 0.14 };
            rig.carry_t = (rig.carry_t + dir * dt / rate).clamp(0.0, 1.0);
        }
        let aim_e = ease_out(rig.carry_t);
        let contact = f.fire_cd > 0.0 || game.sim.t - f.last_dmg_at < 6.0;
        let carry_pitch = if contact { 0.56 } else { 0.31 }; // low ready / patrol
        // §2.3 weapon-mass settle: the gun trails the spine on turns -
        // heavy weapons lag and overshoot, light ones snap
        let yaw_delta = wrap_angle(f.yaw - rig.prev_yaw_vis);
        rig.prev_yaw_vis = f.yaw;
        let (lag_s, zeta) = weapon_lag(f.gun);
        rig.wr_lag_yaw = (rig.wr_lag_yaw - yaw_delta * lag_s * 6.0).clamp(-0.22, 0.22);
        let w_spring = 1.0 / lag_s.max(0.01);
        rig.wr_lag_v +=
            (-2.0 * zeta * w_spring * rig.wr_lag_v - w_spring * w_spring * rig.wr_lag_yaw) * dt;
        rig.wr_lag_yaw += rig.wr_lag_v * dt;
        // §2.2 muzzle avoidance: probe 0.9 m ahead of the muzzle; raise
        // the barrel proportionally to the intrusion (to ~55°) - solves
        // wall clipping AND reads as trained CQB handling
        let muzzle_up = {
            let (fx, fz) = (f.yaw.sin(), f.yaw.cos());
            let o = [f.pos[0] + fx * 0.25, f.pos[1] + 1.28, f.pos[2] + fz * 0.25];
            match game.sim.raycast_cover(o, [fx, 0.0, fz], 0.9) {
                Some((t, _)) => -(1.0 - t / 0.9) * 0.96,
                None => 0.0,
            }
        };
        let reload_cant = if f.reload_t > 0.0 { 0.44 } else { 0.0 };
        let (wr_pos, wr_rot) = if spear_cocked || f.spear_wind_t > 0.0 {
            // §3.2: drawn back level with the shoulder through the wind,
            // whipping forward at release
            let wind_back = if f.spear_wind_t > 0.0 {
                (1.0 - f.spear_wind_t / SPEAR_WINDUP_S).min(0.68) * 0.8
            } else {
                0.0
            };
            (
                Vec3::new(0.16, 0.72, 0.02 - 0.12 * wind_back),
                Quat::from_rotation_x(wr_pitch - 1.35 - 0.5 * wind_back + jerk * 1.5),
            )
        } else if f.gun == GunKind::Bow {
            // the bow stands in front of the LEFT side
            (
                Vec3::new(-0.04, 0.48, 0.16),
                Quat::from_rotation_x(wr_pitch * 0.9),
            )
        } else {
            let s = ease_out(rig.sprint_t);
            // aim ↔ carry blend, then sprint, muzzle-avoidance, and the
            // §3.4 reload cant (25° toward the body) on top.
            // §5 (Brief IV): the root rides far enough out that the
            // longest stock clears the chest ellipse in every stance -
            // ADS pulls it in and UP to the shoulder pocket instead of
            // back through the torso.
            let pitch = (aim_pitch * aim_e + carry_pitch * (1.0 - aim_e)) - torso_pitch;
            let z = WR_Z_HIP + (WR_Z_ADS - WR_Z_HIP) * aim_e;
            (
                Vec3::new(WR_X, 0.50 + 0.06 * aim_e - 0.06 * s, z),
                Quat::from_rotation_y(0.35 * s + rig.wr_lag_yaw)
                    * Quat::from_rotation_x(pitch + jerk * 0.18 + 0.61 * s + muzzle_up)
                    * Quat::from_rotation_z(reload_cant),
            )
        };
        if let Ok((mut t, _)) = parts.get_mut(rig.weapon_root) {
            // §1.1: the grip fidget - a brief re-settle of the hands on
            // the weapon once every 8-15 s, per-fighter phase from the
            // id hash (never the sim RNG). It was written, tested, and
            // named in the living-motion comment but never actually
            // reached a Transform; this is its consumer. Suppressed
            // while the fighter is doing something committed, so it can
            // only ever read as idle life.
            let idle_hands = !rolling
                && f.spear_wind_t <= 0.0
                && f.knife_phase <= 0.0
                && f.reload_t <= 0.0
                && jerk < 0.05;
            let fidget = if idle_hands {
                grip_fidget(tnow, ph, id_period(vis.index as u32 + 41, 8.0, 15.0))
            } else {
                0.0
            };
            t.translation = wr_pos + Vec3::new(0.0, -0.4 * fidget, 0.25 * fidget);
            t.rotation = wr_rot * Quat::from_rotation_x(-0.5 * fidget);
        }
        let mut arrow_vis = Visibility::Hidden;
        // hand IK targets in torso space: the weapon's OWN sockets
        let sockets = weapon_hand_specs(f.gun);
        let grip_t = sockets.first().map(|(p, ..)| wr_pos + wr_rot * *p);
        let fore_t = sockets.get(1).map(|(p, ..)| wr_pos + wr_rot * *p);
        let sh_l = Vec3::new(-0.26, 0.62, 0.02);
        let sh_r = Vec3::new(0.26, 0.62, 0.02);
        let pole_l = Vec3::new(-0.574, -0.80, 0.15); // down-and-out 35°
        let pole_r = Vec3::new(0.574, -0.80, 0.15);
        // (shoulder quat, elbow flex, wrist pitch) per arm
        let mut left = (Quat::from_rotation_x(swing * 0.5), 0.15, 0.0_f32);
        let mut right = (Quat::from_rotation_x(-swing * 0.5), 0.15, 0.0_f32);
        if rolling {
            // arms wrap the ball
            left = (Quat::from_rotation_x(-0.9), 1.2, 0.0);
            right = (Quat::from_rotation_x(-0.9), 1.2, 0.0);
        } else if f.shield_up {
            // shield forward on the left arm; gun hand drops to the hip
            left = (Quat::from_rotation_x(-1.35), 0.35, 0.0);
            right = (Quat::from_rotation_x(-0.5), 0.3, 0.0);
        } else {
            match f.gun {
                GunKind::Fists => {}
                GunKind::Bow => {
                    // left hand IK to the bow grip; right hand IK to the
                    // STRING, pulled back with the draw
                    if let Some(t) = grip_t {
                        let (q, e) = solve_arm_ik(sh_l, t, pole_l);
                        left = (q, e, 0.0);
                    }
                    // §3.3: a REAL anchor - the string hand draws to the
                    // corner of the mouth at full draw, and flies back
                    // past the ear on release (the follow-through sells it)
                    let release_fly = if jerk > 0.55 { (jerk - 0.55) * 0.5 } else { 0.0 };
                    let nock = wr_pos
                        + wr_rot
                            * Vec3::new(
                                0.03,
                                0.14 * bow_draw,
                                -0.09 - 0.18 * bow_draw - release_fly,
                            );
                    let (q, e) = solve_arm_ik(sh_r, nock, pole_r);
                    right = (q, e, 0.0);
                    if f.ammo > 0 && f.reload_t <= 0.0 {
                        arrow_vis = Visibility::Inherited;
                    }
                }
                _ => {
                    // gun/spear: right hand IK to the grip socket; left
                    // hand IK to the foregrip, or a chest-guard idle when
                    // the weapon has none (pistols)
                    if let Some(t) = grip_t {
                        let (q, e) = solve_arm_ik(sh_r, t, pole_r);
                        right = (q, e, 0.0);
                    }
                    // §3.2: through the windup the OFF ARM points at the
                    // target - the classic javelin sight line, and the
                    // single most readable tell of the committed throw
                    let lt = if f.gun == GunKind::Spear && f.spear_wind_t > 0.0 {
                        Vec3::new(-0.06, 0.58, 0.55)
                    } else {
                        fore_t.unwrap_or(Vec3::new(-0.12, 0.38, 0.16))
                    };
                    let (q, e) = solve_arm_ik(sh_l, lt, pole_l);
                    left = (q, e, 0.0);
                }
            }
        }
        // §6 (Brief IV): a live melee swing owns the RIGHT arm - the hand
        // rises beside the ear through the wind, then sweeps across to
        // the opposite hip. Readable at range; enemies see it coming.
        // §2 (Brief V): with the SPEAR in hand this is the THRUST - the
        // hand draws back along the flank, DRIVES past full reach (the
        // follow-through overshoot), and settles back - arms finishing
        // after the hips above.
        if f.knife_phase > 0.0 && !rolling && !f.shield_up {
            let thrust_v = f.gun == GunKind::Spear;
            let tmul_v = if thrust_v && in_mech {
                MECH_THRUST_TIME_MULT
            } else {
                1.0
            };
            let w = if thrust_v {
                THRUST_WIND_S * tmul_v
            } else {
                match (f.melee_axe, f.knife_committed) {
                    (false, false) => KNIFE_QUICK_WIND_S,
                    (false, true) => KNIFE_LUNGE_WIND_S,
                    (true, false) => AXE_QUICK_WIND_S,
                    (true, true) => AXE_LUNGE_WIND_S,
                }
            };
            let ph = f.knife_phase;
            let target = if thrust_v {
                if ph < w {
                    // load: draw back along the flank as the hips coil
                    let e = ease_out((ph / w).clamp(0.0, 1.0));
                    Vec3::new(0.20, 0.46, 0.24).lerp(Vec3::new(0.30, 0.42, -0.12), e)
                } else {
                    let span =
                        (THRUST_ACTIVE_S + THRUST_RECOVER_HIT_S) * tmul_v;
                    let r = ((ph - w) / span).clamp(0.0, 1.0);
                    if r < 0.25 {
                        // the drive: past full reach - the overshoot
                        Vec3::new(0.30, 0.42, -0.12)
                            .lerp(Vec3::new(0.10, 0.50, 0.98), ease_out(r / 0.25))
                    } else {
                        // the tip settles back from the overshoot
                        Vec3::new(0.10, 0.50, 0.98).lerp(
                            Vec3::new(0.16, 0.47, 0.72),
                            ease_out((r - 0.25) / 0.75),
                        )
                    }
                }
            } else if ph < w {
                // wind: from a mid guard up beside the ear
                let e = ease_out((ph / w).clamp(0.0, 1.0));
                Vec3::new(0.16, 0.44, 0.28).lerp(Vec3::new(0.36, 0.80, -0.06), e)
            } else {
                // strike snaps across, then eases back to the guard
                let active = if f.melee_axe {
                    AXE_QUICK_ACTIVE_S + AXE_QUICK_RECOVER_S
                } else {
                    KNIFE_QUICK_ACTIVE_S + KNIFE_QUICK_RECOVER_S
                };
                let r = ((ph - w) / active).clamp(0.0, 1.0);
                let hit = Vec3::new(-0.26, 0.26, 0.44);
                hit.lerp(Vec3::new(0.16, 0.44, 0.28), ease_out(r))
            };
            let (q, e) = solve_arm_ik(sh_r, target, pole_r);
            right = (q, e, 0.0);
        }
        // §1 (Brief V): an aimed throw coils the OFF hand up beside the
        // head - visible at range, like the spear's javelin sight line
        if f.cook_t > 0.0
            && !rolling
            && !(f.armor_set == ArmorSet::RobotSuit && f.hull > 0.0)
        {
            let wind_e = (f.cook_t / THROW_CHARGE_MAX_S).clamp(0.0, 1.0);
            let t = Vec3::new(-0.12, 0.52, 0.20)
                .lerp(Vec3::new(-0.20, 0.86, -0.10), wind_e);
            let (q, e) = solve_arm_ik(sh_l, t, pole_l);
            left = (q, e, 0.0);
        }
        for (arm, (sh, elbow, hand)) in [(rig.arm_l, left), (rig.arm_r, right)] {
            if let Ok((mut t, _)) = parts.get_mut(arm[0]) {
                t.rotation = sh;
            }
            if let Ok((mut t, _)) = parts.get_mut(arm[1]) {
                // hinge constraint: the elbow only folds forward
                t.rotation = Quat::from_rotation_x(-elbow.max(0.0));
            }
            if let Ok((mut t, _)) = parts.get_mut(arm[2]) {
                t.rotation = Quat::from_rotation_x(hand);
            }
        }
        // the shield plate itself
        if let Ok((_, mut v)) = parts.get_mut(rig.shield) {
            *v = if f.shield_up {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if let Ok((_, mut v)) = parts.get_mut(rig.bow_arrow) {
            *v = arrow_vis;
        }
        // §6: the powered shell shows while the Robot Suit is worn
        if let Ok((_, mut v)) = parts.get_mut(rig.armor_rig) {
            *v = if f.armor_set == ArmorSet::RobotSuit {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn sync_tracers(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    mut pool: ResMut<TracerPool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<TracerMarker>>,
) {
    while pool.0.len() < game.sim.tracers.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.tracer_mesh.clone()),
                MeshMaterial3d(assets.tracer_blue.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                TracerMarker,
            ))
            .id();
        pool.0.push(e);
    }
    for (idx, e) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis)) = q.get_mut(*e) else {
            continue;
        };
        match game.sim.tracers.get(idx) {
            Some(tr) => {
                // §3.3: the HIT ray runs eye→aim-point; the VISIBLE streak
                // starts a touch forward and below - where the muzzle sits.
                // Purely cosmetic, fully decoupled from the hit test.
                let a0 = Vec3::from_array(tr.from);
                let b = Vec3::from_array(tr.to);
                let seg = b - a0;
                let sl = seg.length().max(0.05);
                let a = a0 + (seg / sl) * (0.45_f32).min(sl * 0.4) - Vec3::Y * 0.12;
                let mid = (a + b) * 0.5;
                let len = (b - a).length().max(0.05);
                let dir = (b - a) / len;
                *tf = Transform::from_translation(mid)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(1.0, 1.0, len));
                *vis = Visibility::Visible;
                let mat = match tr.team {
                    Team::Blue => assets.tracer_blue.clone(),
                    Team::Red => assets.tracer_red.clone(),
                };
                commands.entity(*e).insert(MeshMaterial3d(mat));
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn sync_missiles(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    mut pool: ResMut<MissilePool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<MissileMarker>>,
) {
    while pool.0.len() < game.sim.missiles.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.missile_mesh.clone()),
                MeshMaterial3d(assets.arrow_mat.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                MissileMarker,
            ))
            .id();
        pool.0.push(e);
    }
    for (idx, e) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis)) = q.get_mut(*e) else {
            continue;
        };
        match game.sim.missiles.get(idx) {
            Some(m) => {
                let dir = Vec3::from_array(m.vel).normalize_or(Vec3::Z);
                let (len, thick) = if m.is_spear { (1.9, 1.6) } else { (0.8, 1.0) };
                *tf = Transform::from_translation(Vec3::from_array(m.pos))
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(thick, thick, len));
                *vis = Visibility::Visible;
                let mat = if m.is_spear {
                    assets.spear_mat.clone()
                } else {
                    assets.arrow_mat.clone()
                };
                commands.entity(*e).insert(MeshMaterial3d(mat));
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// §3: render recoverable ammo piles - a lying shaft per pile, slightly
/// scaled up as arrows merge into it. (The through-wall pixel outline
/// belongs to the §7 HUD pass.)
fn sync_dropped(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    mut pool: ResMut<DroppedPool>,
    mut q: Query<
        (&mut Transform, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>),
        With<DroppedMarker>,
    >,
) {
    while pool.0.len() < game.sim.dropped.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.missile_mesh.clone()),
                MeshMaterial3d(assets.arrow_mat.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                DroppedMarker,
            ))
            .id();
        pool.0.push(e);
    }
    for (idx, e) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis, mut mat)) = q.get_mut(*e) else {
            continue;
        };
        match game.sim.dropped.get(idx) {
            Some(d) => {
                let yaw = d.rest_tick as f32 * 0.61;
                let bulk = 1.0 + 0.18 * ((d.count as f32) - 1.0).min(4.0);
                *tf = Transform::from_xyz(d.pos[0], d.pos[1] + 0.06, d.pos[2])
                    .with_rotation(Quat::from_rotation_y(yaw))
                    .with_scale(Vec3::new(bulk, 1.0, if d.kind == AmmoKind::Spear { 1.8 } else { 0.8 }));
                *mat = MeshMaterial3d(match d.kind {
                    AmmoKind::Arrow => assets.arrow_mat.clone(),
                    AmmoKind::Spear => assets.spear_mat.clone(),
                });
                *vis = Visibility::Visible;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

/// §5: render throwables - tumbling grenades (spin is VISUAL only), smoke
/// spheres blooming in, flickering fire pools, and detonation flashes.
/// The §5.3 whiteout uses QUANTISED alpha steps (pixel-dither read), and
/// only ever reads sim state - nothing here feeds back.
#[allow(clippy::too_many_arguments)]
fn sync_throwables(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<ThrowAssets>,
    mut pools: ResMut<ThrowPools>,
    mut q: Query<
        (&mut Transform, &mut Visibility),
        Or<(
            With<GrenadeMarker>,
            With<SmokeMarker>,
            With<FireMarker>,
            With<BoomMarker>,
        )>,
    >,
    mut flash: Query<&mut BackgroundColor, With<FlashOverlay>>,
) {
    let simr = &game.sim;
    // grow pools to demand
    while pools.grenades.len() < simr.grenades_air.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.ball.clone()),
                MeshMaterial3d(assets.body.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                GrenadeMarker,
            ))
            .id();
        pools.grenades.push(e);
    }
    while pools.smokes.len() < simr.smokes.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.ball.clone()),
                MeshMaterial3d(assets.smoke.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                SmokeMarker,
            ))
            .id();
        pools.smokes.push(e);
    }
    while pools.fires.len() < simr.fires.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.fire_mesh.clone()),
                MeshMaterial3d(assets.fire.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                FireMarker,
            ))
            .id();
        pools.fires.push(e);
    }
    while pools.booms.len() < simr.booms.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.ball.clone()),
                MeshMaterial3d(assets.flashband.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                BoomMarker,
            ))
            .id();
        pools.booms.push(e);
    }
    for (idx, e) in pools.grenades.iter().enumerate() {
        if let Ok((mut t, mut v)) = q.get_mut(*e) {
            match simr.grenades_air.get(idx) {
                Some(g) => {
                    // 9 rad/s tumble - cosmetic, never fed back to the sim
                    let spin = simr.t * 9.0 + g.id as f32;
                    *t = Transform::from_xyz(g.pos[0], g.pos[1].max(0.08), g.pos[2])
                        .with_rotation(Quat::from_rotation_x(spin) * Quat::from_rotation_z(spin * 0.7))
                        .with_scale(Vec3::splat(0.16));
                    *v = Visibility::Visible;
                }
                None => *v = Visibility::Hidden,
            }
        }
    }
    for (idx, e) in pools.smokes.iter().enumerate() {
        if let Ok((mut t, mut v)) = q.get_mut(*e) {
            match simr.smokes.get(idx) {
                Some(s) => {
                    let age = SMOKE_TTL_S - s.ttl;
                    let bloom = (age * 1.6).clamp(0.1, 1.0);
                    let fade = (s.ttl / 2.5).clamp(0.0, 1.0);
                    let r = throw_spec(ThrowKind::Smoke).radius_m * 2.0 * bloom * fade.max(0.25);
                    *t = Transform::from_translation(Vec3::from_array(s.pos))
                        .with_scale(Vec3::splat(r));
                    *v = Visibility::Visible;
                }
                None => *v = Visibility::Hidden,
            }
        }
    }
    for (idx, e) in pools.fires.iter().enumerate() {
        if let Ok((mut t, mut v)) = q.get_mut(*e) {
            match simr.fires.get(idx) {
                Some(fp) => {
                    let flick = 1.0 + (simr.t * 17.0 + idx as f32).sin() * 0.08;
                    let r = throw_spec(ThrowKind::Molotov).radius_m * flick;
                    *t = Transform::from_xyz(fp.pos[0], fp.pos[1] + 0.04, fp.pos[2])
                        .with_scale(Vec3::new(r, 1.0, r));
                    *v = Visibility::Visible;
                }
                None => *v = Visibility::Hidden,
            }
        }
    }
    for (idx, e) in pools.booms.iter().enumerate() {
        if let Ok((mut t, mut v)) = q.get_mut(*e) {
            match simr.booms.get(idx) {
                Some((b, ttl)) => {
                    let age = (2.0 - ttl).max(0.0);
                    if age > 0.35 || b.kind == ThrowKind::Smoke {
                        *v = Visibility::Hidden;
                        continue;
                    }
                    *t = Transform::from_translation(Vec3::from_array(b.at))
                        .with_scale(Vec3::splat(0.4 + age * 9.0));
                    *v = Visibility::Visible;
                }
                None => *v = Visibility::Hidden,
            }
        }
    }
    // §5.3 whiteout: quantised to 8 steps - dither, never a smooth fade
    if let Ok(mut bg) = flash.get_single_mut() {
        let p = &simr.fighters[simr.player];
        let a = (p.blind_t / FLASH_BLIND_S).clamp(0.0, 1.0);
        let stepped = (a * 8.0).round() / 8.0;
        *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, stepped * 0.96));
    }
}

/// §8: render the horde - a moss body + pale head per zombie, scaled by
/// kind (the Brute reads at a glance), plus the extraction beacon.
fn sync_zombies(
    mut commands: Commands,
    game: Res<Game>,
    kit: Res<ModelKit>,
    assets: Res<ZombieAssets>,
    mut pool: ResMut<ZombiePool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<ZombieMarker>>,
) {
    let simr = &game.sim;
    while pool.bodies.len() < simr.zombies.len() {
        let b = commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(assets.moss.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                ZombieMarker,
            ))
            .id();
        let h = commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(assets.pale.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                ZombieMarker,
            ))
            .id();
        pool.bodies.push(b);
        pool.heads.push(h);
    }
    if pool.beacon.is_none() {
        pool.beacon = Some(
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(assets.beacon.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                    ZombieMarker,
                ))
                .id(),
        );
    }
    for idx in 0..pool.bodies.len() {
        let (be, he) = (pool.bodies[idx], pool.heads[idx]);
        match simr.zombies.get(idx) {
            Some(z) => {
                let zs = zspec(z.kind);
                let yaw = (z.target[0] - z.pos[0]).atan2(z.target[1] - z.pos[2]);
                let lurch = (simr.t * 3.1 + z.id as f32).sin() * 0.06;
                if let Ok((mut t, mut v)) = q.get_mut(be) {
                    *t = Transform::from_xyz(z.pos[0], z.pos[1] + zs.height * 0.45, z.pos[2])
                        .with_rotation(
                            Quat::from_rotation_y(yaw) * Quat::from_rotation_z(lurch),
                        )
                        .with_scale(Vec3::new(
                            zs.girth * 2.0,
                            zs.height * 0.72,
                            zs.girth * 1.6,
                        ));
                    *v = Visibility::Visible;
                }
                if let Ok((mut t, mut v)) = q.get_mut(he) {
                    *t = Transform::from_xyz(
                        z.pos[0],
                        z.pos[1] + zs.height * 0.90,
                        z.pos[2],
                    )
                    .with_scale(Vec3::splat(zs.girth * 1.1));
                    *v = Visibility::Visible;
                }
            }
            None => {
                for e in [be, he] {
                    if let Ok((_, mut v)) = q.get_mut(e) {
                        *v = Visibility::Hidden;
                    }
                }
            }
        }
    }
    if let Some(be) = pool.beacon {
        if let Ok((mut t, mut v)) = q.get_mut(be) {
            match simr.extract_point() {
                Some(p2) => {
                    *t = Transform::from_xyz(p2[0], 20.0, p2[2])
                        .with_scale(Vec3::new(EXTRACT_RADIUS * 2.0, 40.0, EXTRACT_RADIUS * 2.0));
                    *v = Visibility::Visible;
                }
                None => *v = Visibility::Hidden,
            }
        }
    }
}

fn sync_decals(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    mut pool: ResMut<DecalPool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<DecalMarker>>,
) {
    const MAX_DECALS: usize = 400;
    let n = game.sim.impacts.len().min(MAX_DECALS);
    while pool.0.len() < n {
        let e = commands
            .spawn((
                Mesh3d(assets.decal_mesh.clone()),
                MeshMaterial3d(assets.decal_mat.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                DecalMarker,
            ))
            .id();
        pool.0.push(e);
    }
    let start = game.sim.impacts.len() - n;
    for (idx, e) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis)) = q.get_mut(*e) else {
            continue;
        };
        match game.sim.impacts.get(start + idx) {
            Some((im, _)) => {
                let nrm = Vec3::from_array(im.normal).normalize_or(Vec3::Y);
                *tf = Transform::from_translation(Vec3::from_array(im.at) + nrm * 0.02)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, nrm));
                *vis = Visibility::Visible;
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn sync_pickups(
    game: Res<Game>,
    roots: Query<&PickupVis>,
    mut items: Query<(&mut Transform, &mut Visibility)>,
) {
    for pv in &roots {
        let Some(p) = game.sim.pickups.get(pv.index) else {
            continue;
        };
        let Ok((mut tf, mut vis)) = items.get_mut(pv.item) else {
            continue;
        };
        if p.respawn_t > 0.0 {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Inherited;
            let t = game.sim.t + pv.index as f32 * 0.7;
            let base_y = if matches!(p.kind, PickupKind::RobotArmor) {
                0.75
            } else {
                1.0
            };
            tf.translation.y = base_y + (t * 2.0).sin() * 0.12;
            tf.rotation = Quat::from_rotation_y(t * 1.5);
        }
    }
}

fn sync_health_bars(
    game: Res<Game>,
    cam: Res<CamCtl>,
    bars: Res<BarAssets>,
    mut roots: Query<(&HealthBarVis, &mut Transform, &mut Visibility), Without<BarFill>>,
    mut fills: Query<
        (&mut Transform, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>),
        (With<BarFill>, Without<HealthBarVis>),
    >,
) {
    for (hb, mut tf, mut vis) in &mut roots {
        // same deploy-frame safety as the rigs: stale bars must not panic
        let Some(f) = game.sim.fighters.get(hb.index) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let self_view =
            cam.person_t < 0.5 || (cam.ads && gun(f.gun).scoped && !f.shield_up);
        if !f.alive() || (hb.index == game.sim.player && self_view) {
            // dead men carry no bar; in first person (or scoped glass)
            // neither do YOU - the HUD panel already shows your numbers
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;
        // above the hat brim
        tf.translation = Vec3::new(f.pos[0], f.pos[1] + f.height() + 0.72, f.pos[2]);
        // billboard: quad faces the camera
        tf.rotation = Quat::from_rotation_y(cam.yaw + PI);
        let frac = (f.health / MAX_HEALTH).clamp(0.0, 1.0);
        if let Ok((mut ft, _, mut mat)) = fills.get_mut(hb.fill) {
            ft.scale = Vec3::new(0.7 * frac.max(0.01), 1.0, 1.0);
            ft.translation.x = -0.35 * (1.0 - frac);
            let handle = if frac > 0.55 {
                bars.green.clone()
            } else if frac > 0.28 {
                bars.orange.clone()
            } else {
                bars.red.clone()
            };
            *mat = MeshMaterial3d(handle);
        }
        if let Ok((mut at, mut av, _)) = fills.get_mut(hb.afill) {
            let afrac = (f.armor / ROBOT_ARMOR_HP).clamp(0.0, 1.0);
            *av = if afrac > 0.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            at.scale = Vec3::new(0.7 * afrac.max(0.01), 1.0, 1.0);
            at.translation.x = -0.35 * (1.0 - afrac);
        }
    }
}

// ---- §B.1 (mech plan): entry/exit presentation ----------------------
// `MechEnterStage` is a fully-specified, TESTED 8-stage sim timer that
// had ZERO client references - the sim knew exactly which stage the
// player was in and nothing ever asked. This is that wiring.
//
// No new sim state and no new constants: everything below reads
// `mech_enter_stage_for` and drives presentation from it.

/// Client-side view of where the player is in the boarding sequence.
#[derive(Default)]
struct MechStageState {
    last_stage: Option<sim::MechEnterStage>,
    /// True once boarding has REACHED `HudBoot`, cleared when a fresh
    /// boarding begins. §B.3's visor camera keys off this.
    visor_ready: bool,
    /// True while an EXIT is running, so the power-down can be sequenced
    /// even though `mech_enter_stage_for` deliberately returns `None`
    /// for the exiting case.
    was_exiting: bool,
}

/// Whether the visor camera may be used, given the stage transition.
///
/// **This exists because the obvious version is a trap.**
/// `mech_enter_stage_for` returns `None` BOTH when boarding has finished
/// AND when the fighter is not a mech at all - so a naive
/// `matches!(stage, None | Some(HudBoot))` cannot tell "fully entered"
/// from "never boarded", and would snap the camera into a visor that
/// does not exist. Tracking the HudBoot EDGE is what distinguishes
/// them. Pure, so the distinction is directly testable.
fn visor_ready_after(
    prev: Option<sim::MechEnterStage>,
    new: Option<sim::MechEnterStage>,
    current: bool,
) -> bool {
    match (prev, new) {
        // a fresh boarding starts: the camera is OUTSIDE, watching
        (_, Some(sim::MechEnterStage::CockpitOpen)) => false,
        // the last stage: the cut into the visor happens here
        (_, Some(sim::MechEnterStage::HudBoot)) => true,
        // any other stage mid-boarding: hold whatever we had
        (_, Some(_)) => current,
        // not transitioning. `None` is ambiguous by itself, which is
        // exactly the trap - so we keep the flag we already earned and
        // let the caller clear it when the mech is actually gone.
        (_, None) => current,
    }
}

/// Drives the one-shot beat for each boarding stage.
///
/// Fires ONLY on a stage CHANGE, never every frame while inside a
/// stage - the plan's "one at a time" rule, which is what keeps the
/// sequence reading as a machine waking up rather than everything
/// happening at once.
///
/// Audio and per-stage meshes are stubbed with `debug!` markers where
/// the assets do not exist yet. That is deliberate per the plan: prove
/// the SEQUENCING is right first, so art and audio drop into a
/// timeline already known to be correct.
fn mech_stage_presentation(
    game: Res<Game>,
    mut st: Local<MechStageState>,
    mut commands: Commands,
    sfx: Option<Res<Sfx>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let stage = sim::mech_enter_stage_for(p);
    let in_mech = p.in_mech();

    // Leaving the chassis entirely clears the earned visor state, so a
    // pilot on foot can never be left looking through a visor.
    if !in_mech {
        st.visor_ready = false;
    }

    // EXIT sequencing. `mech_enter_stage_for` returns None while
    // exiting (it has no stage list of its own), so drive the reverse
    // walk directly off the timer. Presentation only.
    let exiting = in_mech && p.mech_exiting && p.mech_transition_t > 0.0;
    if exiting != st.was_exiting {
        st.was_exiting = exiting;
        if exiting {
            debug!("mech: power-down begins - reverse stage walk over MECH_EXIT_S");
            st.visor_ready = false; // the visor goes dark first
        }
    }

    if stage == st.last_stage {
        return; // still inside the same stage: nothing to fire
    }
    st.visor_ready = visor_ready_after(st.last_stage, stage, st.visor_ready);
    st.last_stage = stage;

    let Some(s) = stage else {
        return; // not boarding: no beat to fire
    };

    // One dry mechanical click per stage is better than silence while
    // the bespoke servo/hydraulic set does not exist - `click` is the
    // one existing sound with the right character. Volume rises across
    // the sequence so the machine audibly builds toward readiness.
    if let Some(sfx) = sfx.as_ref() {
        let idx = sim::MECH_ENTER_STAGES
            .iter()
            .position(|x| *x == s)
            .unwrap_or(0) as f32;
        let vol = 0.18 + idx * 0.035;
        play(&mut commands, &sfx.click, vol);
    }

    match s {
        sim::MechEnterStage::CockpitOpen => debug!("mech stage 1/8: cockpit opens"),
        sim::MechEnterStage::ClimbIn => debug!("mech stage 2/8: pilot climbs in"),
        sim::MechEnterStage::Harness => debug!("mech stage 3/8: harness closes"),
        sim::MechEnterStage::PowerUp => debug!("mech stage 4/8: power-up, seam lights"),
        sim::MechEnterStage::ServoSync => debug!("mech stage 5/8: servo sync"),
        sim::MechEnterStage::GyroCalibration => debug!("mech stage 6/8: gyro calibration"),
        sim::MechEnterStage::WeaponDiagnostics => {
            debug!("mech stage 7/8: weapon diagnostics - both hull mounts cycle")
        }
        sim::MechEnterStage::HudBoot => {
            debug!("mech stage 8/8: HUD boot - camera may cut to the visor")
        }
    }
}

fn camera_system(
    time: Res<Time>,
    game: Res<Game>,
    mut cam_ctl: ResMut<CamCtl>,
    cam_tuning: Res<CameraTuning>,
    settings: Res<GameSettings>,
    mut q: Query<(&mut Transform, &mut Projection), With<MainCam>>,
) {
    cam_ctl.recoil = (cam_ctl.recoil - time.delta_secs() * 5.0).max(0.0);
    let Ok((mut tf, mut proj)) = q.get_single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    let p = &game.sim.fighters[game.sim.player];
    // §5.1: the person toggle BLENDS - boom length eases 0 → target over
    // PERSON_BLEND_S, framerate-independently, never a snap
    {
        let dir = if cam_ctl.first_person { -1.0 } else { 1.0 };
        cam_ctl.person_t = (cam_ctl.person_t + dir * dt / PERSON_BLEND_S).clamp(0.0, 1.0);
    }
    // §2 channel 2 (Brief VI): the camera shows punch × 2.0 × 0.45 -
    // 45% of the true deflection. The crosshair is screen-anchored and
    // never moves; impacts drifting above it is the skill expression.
    let vp = p.punch;
    let view_pitch = cam_ctl.pitch
        - (vp[0] * RECOIL_SCALE * VIEW_RECOIL_TRACKING).to_radians();
    let view_yaw = cam_ctl.yaw
        + (vp[1] * RECOIL_SCALE * VIEW_RECOIL_TRACKING).to_radians();
    let (sy, cy) = (view_yaw.sin(), view_yaw.cos());
    let (sp, cp) = (view_pitch.sin(), view_pitch.cos());
    let fwd = Vec3::new(sy * cp, -sp, cy * cp);
    let right = Vec3::new(cy, 0.0, -sy);
    // an ADS'd AWM ALWAYS looks through the glass: even in third person
    // the scoped view snaps to the eye (a scope over a shoulder-cam
    // makes no sense)
    let scoped_in = cam_ctl.ads && p.alive() && gun(p.gun).scoped && !p.shield_up;
    // `right` above is the lean-LEFT direction under this game's yaw
    // convention; the true screen-right is its negation
    let screen_right = -right;

    // first-person eye - exact, no positional smoothing (aim never swims)
    let eye_h = (p.height() - 0.16).max(0.55);
    let eye = Vec3::new(p.pos[0], p.pos[1] + eye_h, p.pos[2])
        + screen_right * (p.lean * LEAN_SHIFT);

    // third person: over-the-RIGHT-shoulder boom off the head pivot;
    // ADS pulls in tight. The boom pitch is clamped so a near-vertical
    // look never degenerates the frame.
    let crouch_drop = if p.roll_t > 0.0 {
        0.75
    } else if p.crouch {
        0.62
    } else {
        0.0
    };
    // Task 4 (MISSION doc): the anchor height must scale with the
    // fighter's ACTUAL height - a hardcoded 1.6m put the camera INSIDE
    // a 3m mech's own body once the scale changed. 1.6 was tuned for a
    // 1.78m soldier; keep that same proportion for every height.
    let anchor_h = 1.6 * (p.height() / BODY_HEIGHT);
    let anchor = Vec3::new(p.pos[0], p.pos[1] + anchor_h - crouch_drop, p.pos[2])
        + screen_right * (p.lean * LEAN_SHIFT * 0.8);
    let ads_e = ease_out(cam_ctl.ads_t);
    // §5.1 (Brief VII v2): the hip boom itself isn't fixed - it eases
    // OUT to 2.5m under sprint (0.12s lag, a simple first-order chase;
    // the collision-recovery spring below is a separate, heavier k=90
    // critical spring - this lag is a lighter, faster settle by design)
    // and ADS still pulls IN from whichever hip base is currently active.
    let sp = (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt();
    let sprint_target = if sp > SPRINT_SPEED * 0.9 { cam_tuning.tp_boom_sprint } else { cam_tuning.tp_boom };
    cam_ctl.sprint_boom +=
        (sprint_target - cam_ctl.sprint_boom) * (dt / cam_tuning.tp_sprint_lag_s).min(1.0);
    // Task 4: the boom distance ALSO needs to grow with a taller subject
    // - otherwise the corrected anchor height still sits close enough to
    // clip the mech's own wide hull at the old 2.2m hip distance.
    let height_boom_mult = (p.height() / BODY_HEIGHT).max(1.0);
    let boom_target = (cam_ctl.sprint_boom
        + (cam_tuning.tp_boom_aim - cam_ctl.sprint_boom) * ads_e)
        * height_boom_mult;
    let lift = cam_tuning.tp_up + (0.22 - cam_tuning.tp_up) * ads_e;
    let right_amt = cam_tuning.tp_right + (cam_tuning.tp_right_aim - cam_tuning.tp_right) * ads_e;
    let bp = cam_ctl.pitch.clamp(-1.2, 1.2);
    let bfwd = Vec3::new(sy * bp.cos(), -bp.sin(), cy * bp.cos());
    // §10: auto-mirror the shoulder when hugging a wall on the camera
    // side - peeking left out of cover must not bury the camera in it
    let want = if game
        .sim
        .raycast_cover(anchor.to_array(), screen_right.to_array(), 1.2)
        .is_some()
    {
        -1.0
    } else {
        1.0
    };
    cam_ctl.shoulder += (want - cam_ctl.shoulder) * (dt / 0.15).min(1.0);
    let desired = anchor - bfwd * boom_target
        + screen_right * (right_amt * cam_ctl.shoulder)
        + Vec3::Y * lift;
    // §5.2 boom collision: cast anchor → desired with a pad; pull the
    // camera in INSTANTLY on contact, push it back out on the k=90
    // SPRING_K_CAMERA_BOOM critical spring - instant push-out pops every
    // time you clear a corner. `boom_step` owns that asymmetry AND the
    // distinction the spring must respect: it only filters recovery from
    // an occlusion, never ordinary free-space boom changes (the sprint
    // ease, the ADS blend, and mouse-look all move `len` on their own
    // documented timings and must not be re-filtered through a heavier
    // spring on top).
    let off = desired - anchor;
    let len = off.length().max(1e-4);
    let dirn = off / len;
    let mut allowed = len;
    let mut hit = false;
    if let Some((t, _)) = game
        .sim
        .raycast_cover(anchor.to_array(), dirn.to_array(), len)
    {
        allowed = (t - CAM_PAD).max(0.25);
        hit = true;
    }
    let (nb, nv, occ) = boom_step(
        cam_ctl.boom,
        cam_ctl.boom_vel,
        cam_ctl.boom_occluded,
        allowed,
        len,
        hit,
        dt,
    );
    cam_ctl.boom = nb;
    cam_ctl.boom_vel = nv;
    cam_ctl.boom_occluded = occ;
    let tp_pos = anchor + dirn * cam_ctl.boom.min(len);

    // blend eye ↔ boom on the eased person fraction; the dead spectate
    // from the boom, the scoped AWM is always the eye
    let pe = if !p.alive() {
        1.0
    } else if scoped_in {
        0.0
    } else {
        ease_out(cam_ctl.person_t)
    };
    // landing weight-absorb: the camera DIPS on the grounded edge, in
    // proportion to the impact, and springs back over ~90 ms - the body
    // absorbing the landing instead of the world stopping dead.
    // Task 3 rule 5 (MISSION doc): never fully damp in one frame - an
    // 8% rebound lifts the camera back UP briefly rather than a pure
    // one-way decay to neutral.
    if p.grounded && !cam_ctl.prev_grounded && p.alive() {
        let impact = (-cam_ctl.prev_vy - 3.0).max(0.0);
        cam_ctl.land_dip = (cam_ctl.land_dip + impact * 0.016).min(0.15);
        // Task 3 rule 5: scaled so the delayed rebound actually EXCEEDS
        // the dip's residue at its own peak - otherwise it can only ever
        // shrink the dip and the camera never crosses neutral.
        cam_ctl.land_rebound = landing_rebound_vy(cam_ctl.prev_vy) * 0.05;
        cam_ctl.land_t = 0.0;
    }
    cam_ctl.land_t += dt;
    // both curves are now sampled from one clock by `landing_offset` -
    // decaying them independently per-frame is what made the rebound
    // inert (it started smaller and faded faster than the dip).
    let land_offset = landing_offset(cam_ctl.land_dip, cam_ctl.land_rebound, cam_ctl.land_t);
    cam_ctl.prev_vy = p.vy;
    cam_ctl.prev_grounded = p.grounded;
    // the mech has a CADENCE: a slow, heavy sway while it walks - the
    // pilot feels the tonnage (render-only, first person mostly)
    let mech_bob = if p.armor_set == ArmorSet::RobotSuit && p.hull > 0.0 && p.grounded {
        let sp = (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt();
        (game.sim.t * std::f32::consts::TAU * 0.9).sin()
            * 0.022
            * (sp / MOVE_SPEED).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // §A.6: the brace stance drop. The hull physically settles lower
    // when braced - the camera cue that sells the ZMP widening the sim
    // already models as a speed and recoil trade. Cosmetic only; the
    // sim's own `mech_brace` is the authority.
    let mech_brace_drop = if p.armor_set == ArmorSet::RobotSuit && p.hull > 0.0 && p.mech_brace {
        p.height() * MECH_BRACE_STANCE_DROP
    } else {
        0.0
    };
    // §B.2: idle life. A multi-ton machine that goes perfectly inert the
    // instant it stops moving reads as a prop, and `mech_bob` alone is
    // walk-only - it returns to exactly zero at a dead stop. These two
    // terms are what keep the hull alive while standing still. Both are
    // pure functions of sim time (see `mech_servo_tremor`/`mech_hull_breath`),
    // so they are directly testable without Bevy.
    let (tremor_x, tremor_z) = if p.in_mech() {
        mech_servo_tremor(game.sim.t)
    } else {
        (0.0, 0.0)
    };
    let hull_breath = if p.in_mech() {
        mech_hull_breath(game.sim.t)
    } else {
        0.0
    };
    // nearby detonations SHAKE the frame - amplitude by proximity, only
    // while the boom is fresh (its ttl starts at 2.0), decaying with it
    let mut shake = 0.0_f32;
    for (b, ttl) in &game.sim.booms {
        if *ttl < 1.55 || b.kind == ThrowKind::Smoke {
            continue;
        }
        let d = Vec3::from_array(b.at).distance(Vec3::new(p.pos[0], p.pos[1], p.pos[2]));
        shake += (1.0 - d / 22.0).max(0.0) * 0.09 * ((*ttl - 1.55) / 0.45);
    }
    // §4.4 (Brief VI): a walking mech THUMPS - soldiers within 6 m of
    // its footfalls feel a subtle 0.2-intensity shake
    for (fi, f2) in game.sim.fighters.iter().enumerate() {
        if fi == game.sim.player
            || !(f2.armor_set == ArmorSet::RobotSuit && f2.hull > 0.0)
            || !f2.alive()
        {
            continue;
        }
        let sp2 = (f2.vel[0] * f2.vel[0] + f2.vel[1] * f2.vel[1]).sqrt();
        if sp2 < 0.5 {
            continue;
        }
        let dx = f2.pos[0] - p.pos[0];
        let dz = f2.pos[2] - p.pos[2];
        let d = (dx * dx + dz * dz).sqrt();
        if d < 6.0 {
            // periodic thump matched to the walk cadence
            let pulse = (game.sim.t * std::f32::consts::TAU * 1.7).sin().max(0.0).powi(6);
            shake += 0.2 * 0.09 * (1.0 - d / 6.0) * pulse;
        }
    }
    let shake = shake.min(0.14);
    let sh = if shake > 0.0 {
        let n = game.sim.t * 113.0;
        Vec3::new(n.sin(), (n * 1.31).cos(), (n * 0.77).sin()) * shake
    } else {
        Vec3::ZERO
    };
    tf.translation = eye.lerp(tp_pos, pe) + sh;
    // §A.6 brace drop settles the hull DOWN; §B.2 hull breath lifts it a
    // fraction on the intake - subtracted and added respectively, in the
    // one line that already owns vertical camera offset.
    tf.translation.y -=
        (land_offset + mech_bob.abs() * 0.5 + mech_brace_drop - hull_breath) * (1.0 - pe * 0.6);
    let look = tf.translation + fwd;
    tf.look_at(look, Vec3::Y);
    // the head tilts with the lean - first person only
    tf.rotate_local_z(p.lean * 0.10 * (1.0 - pe));
    // ...and rolls a breath with the mech's stride
    tf.rotate_local_z(mech_bob * 0.35 * (1.0 - pe));
    // §B.2: servo micro-tremor - an idling machine is never perfectly
    // still. Deliberately on its OWN frequencies, not mech_bob's, so it
    // reads as tremor rather than as the stride at low amplitude.
    tf.rotate_local_x(tremor_x * (1.0 - pe));
    tf.rotate_local_z(tremor_z * (1.0 - pe));

    // §3.4: FOV rides ads_t (ease-out, framerate-independent) - never the
    // `+= (target-fov)*k` exponential that stalls and never arrives
    if let Projection::Perspective(persp) = &mut *proj {
        // §5.2 (Brief VI): scoped-class two-stage zoom - 40° then 10°
        let zoom = if p.armed() && !p.shield_up {
            if gun(p.gun).scoped && cam_ctl.zoom_stage == 2 {
                10.0
            } else {
                gun(p.gun).zoom_deg
            }
        } else {
            settings.fov_deg()
        };
        // the player's chosen hip FOV, not the fixed 62 (Settings)
        let hip = settings.fov_deg().to_radians();
        let fov = hip + (zoom.to_radians() - hip) * ads_e;
        persp.fov = fov;
        cam_ctl.fov_now = fov; // §3.2 reads the live value next frame
    }
}

/// §2.3: `RenderLayers` does not propagate to children in Bevy 0.15 -
/// walk the viewmodel hierarchy once (after the spawn flush) and stamp
/// every descendant onto the viewmodel layer so only the fixed-FOV
/// viewmodel camera renders it.
fn tag_viewmodel_layer(
    mut done: Local<bool>,
    mut commands: Commands,
    vm: Res<VmRig>,
    children: Query<&Children>,
) {
    if *done {
        return;
    }
    let mut stack = vec![vm.root];
    let mut count = 0usize;
    while let Some(e) = stack.pop() {
        commands
            .entity(e)
            .insert(RenderLayers::layer(VIEWMODEL_LAYER));
        count += 1;
        if let Ok(ch) = children.get(e) {
            stack.extend(ch.iter().copied());
        }
    }
    if count > 10 {
        *done = true; // hierarchy flushed and stamped - nothing left to do
    }
}

/// §2.2 first-person carry motion ("gunpowder motion"): figure-8 bob,
/// critically-damped mouse sway, breathing idle, pitch lag, landing dip,
/// sprint low-ready - ALL on the viewmodel transform only. Brief I §3.1
/// is absolute: nothing here touches the yaw/pitch used for shooting.
/// §3 (Brief IV): signature reloads - per-class choreography as pose
/// curves over reload progress r ∈ 0..1. The SIM's reload clock is the
/// master (the animation fits the duration, never the reverse), so
/// interrupts and resumes can never desync ammo from motion. Returns
/// (translation offset, euler xyz) for the viewmodel root.
fn reload_pose(kind: GunKind, r: f32) -> (Vec3, Vec3) {
    let pulse = |a: f32, b: f32| -> f32 {
        if r < a || r > b {
            0.0
        } else {
            ((r - a) / (b - a) * PI).sin()
        }
    };
    match kind {
        GunKind::Glock | GunKind::Deagle => {
            // mag drops free with a flick, new mag rocks in, thumb rides
            // the slide release
            let drop = pulse(0.05, 0.35);
            let seat = pulse(0.45, 0.75);
            let slide = pulse(0.82, 0.97);
            (
                Vec3::new(0.02 * drop, -0.09 * drop - 0.05 * seat, 0.02 * seat),
                Vec3::new(
                    0.30 * drop + 0.18 * seat - 0.10 * slide,
                    0.0,
                    -0.45 * drop - 0.20 * seat,
                ),
            )
        }
        GunKind::Ak47 | GunKind::M4 => {
            // rotational mag strip, rock-in with a wrist snap, charging
            // pull while the muzzle dips
            let strip = pulse(0.05, 0.35);
            let rock = pulse(0.40, 0.70);
            let charge = pulse(0.78, 0.96);
            (
                Vec3::new(0.0, -0.10 * strip - 0.06 * rock, 0.03 * charge),
                Vec3::new(
                    0.35 * strip + 0.22 * rock + 0.14 * charge,
                    -0.12 * rock,
                    -0.50 * strip - 0.25 * rock + 0.20 * charge,
                ),
            )
        }
        GunKind::Shotgun => {
            // shell-by-shell underside feed - one dip per shell; the SIM
            // already allows firing mid-reload, so the rhythm just stops
            let shell = ((r * 5.0).fract() * PI).sin();
            (
                Vec3::new(0.0, -0.06 - 0.03 * shell, 0.0),
                Vec3::new(0.35 + 0.10 * shell, 0.15, -0.30),
            )
        }
        GunKind::M249 => {
            // box swap, feed cover up, belt laid flat, cover SLAMS
            let boxs = pulse(0.0, 0.30);
            let cover = pulse(0.30, 0.55);
            let belt = pulse(0.55, 0.85);
            let slam = pulse(0.88, 1.0);
            (
                Vec3::new(0.02, -0.14 * boxs - 0.08 * belt, 0.0),
                Vec3::new(
                    0.30 * boxs + 0.55 * cover + 0.35 * belt - 0.15 * slam,
                    0.10,
                    -0.55 * (boxs + belt).min(1.0),
                ),
            )
        }
        GunKind::Mp5 => {
            // the classic: mag out, mag in, SLAP the charging handle
            let out = pulse(0.05, 0.35);
            let inn = pulse(0.42, 0.68);
            let slap = pulse(0.80, 0.95);
            (
                Vec3::new(0.03 * slap, -0.09 * out - 0.05 * inn, 0.0),
                Vec3::new(
                    0.28 * out + 0.18 * inn,
                    -0.30 * slap,
                    -0.40 * out - 0.35 * slap,
                ),
            )
        }
        GunKind::Awm => {
            // cants 30° left, rounds fed singly from the top
            let cant = (r * PI).sin().min(1.0);
            let feed = ((r * 5.0).fract() * PI).sin() * pulse(0.15, 0.9);
            (
                Vec3::new(-0.02 * cant, -0.08 * cant - 0.02 * feed, 0.02),
                Vec3::new(0.25 * cant + 0.08 * feed, 0.10 * cant, 0.52 * cant),
            )
        }
        _ => {
            // bow nock / spear heft / bare hands: a simple settle
            let s = (r * PI).sin();
            (
                Vec3::new(0.0, -0.06 * s, 0.02 * s),
                Vec3::new(0.25 * s, 0.0, -0.15 * s),
            )
        }
    }
}

/// All of the viewmodel's per-frame state in one bundle - Bevy systems
/// cap at 16 parameters, and the carry motion earns its keep.
#[derive(Default)]
struct VmState {
    theta: f32,
    sway: Vec2,
    sway_v: Vec2,
    pitch_lag: f32,
    prev_pitch: f32,
    prev_vy: f32,
    land_t: f32,
    sprint_t: f32,
    inspect: bool,
    inspect_t: f32,
    /// §3.2: the spear windup fraction, EASED. `spear_wind_t` snaps from
    /// its last tick straight to 0 on release, and this value drives
    /// ~30 cm of translation and ~31 deg of rotation - reading it raw
    /// teleported the viewmodel in a single frame.
    spear_wind_ease: f32,
}

#[allow(clippy::too_many_arguments)]
fn fp_viewmodel(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    vm: Res<VmRig>,
    mut motion: EventReader<MouseMotion>,
    mut st: Local<VmState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Transform, &mut Visibility), (Without<MainCam>, Without<TriggerFinger>)>,
    mut trig: Query<(&mut Transform, &TriggerFinger), Without<MainCam>>,
) {
    let dt = time.delta_secs().max(1e-5);
    let p = &game.sim.fighters[game.sim.player];
    // this system keeps its OWN MouseMotion cursor - reading here does
    // not consume the events the aim path reads
    let mut mdelta = Vec2::ZERO;
    for ev in motion.read() {
        mdelta += ev.delta;
    }
    let spec = gun(p.gun);
    // §2.4 (Brief VII v2): trigger finger travels to the trigger over
    // 0.06s on fire and returns over 0.10s - LEADS the shot (fire_cd is
    // set the same tick the shot resolves).
    let t_since = if p.armed() && p.fire_cd > 0.0 {
        spec.fire_period - p.fire_cd
    } else {
        1.0
    };
    let press = trigger_finger_press(t_since);
    // §2.2 trigger discipline: the index finger rests ON THE RECEIVER in
    // every state except Aimed - it moves to the trigger during the ADS
    // blend. Nearly free, and it reads as "trained".
    let on_trigger = cam_ctl.ads_t;
    for (mut t, tf_) in &mut trig {
        let rest = -0.12 + (tf_.rest + 0.12) * on_trigger;
        t.rotation = Quat::from_rotation_x(rest - press * 0.38);
    }
    let show = cam_ctl.person_t < 0.5 && p.alive() && p.roll_t <= 0.0;
    if let Ok((_, mut v)) = q.get_mut(vm.root) {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !show {
        return;
    }
    let slot = weapon_slot(p.gun);
    let scoped = cam_ctl.ads && spec.scoped;
    for (wi, we) in vm.weapons.iter().enumerate() {
        if let Ok((_, mut v)) = q.get_mut(*we) {
            // the raised shield replaces the gun view; a scoped AWM view
            // is all glass (the overlay), no viewmodel in the way
            *v = if slot == Some(wi) && !p.shield_up && !scoped {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    if let Ok((_, mut v)) = q.get_mut(vm.shield) {
        *v = if p.shield_up {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let speed = (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt();
    // §2.2 suppression during ADS: ×(1 − 0.85-ads_t) - a trace of life
    // stays at full zoom, but a scoped gun does not swim
    let supp = 1.0 - cam_ctl.ads_t * 0.85;
    // §2.3 (Brief IV): MINIMAL bob - half the old amplitudes. CS:GO's
    // gun barely bobs; steadiness is what reads as professional.
    // §1.3 (Brief VI): the bob CLOCK only advances while moving - theta
    // frozen at standstill; the offsets flow through `carry_offset`, the
    // same pure fn the no-bounce tests measure.
    st.theta += speed * dt * 4.4;
    let s = (speed / SPRINT_SPEED).clamp(0.0, 1.0) * supp;
    // sway: the gun lags the camera - critically damped spring (ω = 14 →
    // ~0.1 s lag), CLOSED FORM so 60 fps and 240 fps agree. §1.3 caps
    // the amplitude at 0.3°.
    let tgt = Vec2::new(
        (-mdelta.x * 0.02).clamp(-VM_SWAY_CAP_DEG, VM_SWAY_CAP_DEG),
        (-mdelta.y * 0.02).clamp(-VM_SWAY_CAP_DEG, VM_SWAY_CAP_DEG),
    );
    // §2.5 (Brief VII v2): the shared damped_spring primitive - same
    // w=14 (k=196) this always ran at, now named and reused instead of
    // hand-rolled here only.
    let (new_sway, new_sway_v) = damped_spring(st.sway, st.sway_v, tgt, 196.0, dt);
    st.sway = new_sway;
    st.sway_v = new_sway_v;
    let sway_rad = st.sway * (PI / 180.0) * supp;
    // breathing idle: 0.28 Hz, ±0.4°
    let breathe = (time.elapsed_secs() * std::f32::consts::TAU * 0.28).sin() * 0.007 * supp;
    // pitch lag: the muzzle trails fast camera pitch by up to ~4°
    let pv = (cam_ctl.pitch - st.prev_pitch) / dt;
    st.prev_pitch = cam_ctl.pitch;
    let lag_tgt = (-pv * 0.05).clamp(-0.07, 0.07) * supp;
    st.pitch_lag += (lag_tgt - st.pitch_lag) * (1.0 - (-10.0 * dt).exp());
    // landing dip: 0.04 m over 180 ms on a real impact
    if p.grounded && st.prev_vy < -3.0 {
        st.land_t = 0.18;
    }
    st.prev_vy = if p.grounded { 0.0 } else { p.vy };
    st.land_t = (st.land_t - dt).max(0.0);
    let dip = if st.land_t > 0.0 {
        0.04 * (PI * (1.0 - st.land_t / 0.18)).sin()
    } else {
        0.0
    };
    // sprint low-ready: in over 220 ms, OUT over 140 ms - the out-blend
    // gates the player's ability to shoot, so it must be the fast one
    let sprinting = speed > SPRINT_SPEED * 0.85 && !cam_ctl.ads && p.armed();
    {
        let dir = if sprinting { 1.0 } else { -1.0 };
        let rate = if sprinting { 0.22 } else { 0.14 };
        st.sprint_t = (st.sprint_t + dir * dt / rate).clamp(0.0, 1.0);
    }
    let sp = ease_out(st.sprint_t);
    let reloading = p.reload_t > 0.0;
    // §3: the spear windup reads in FIRST person too - the arm hauls
    // back and up through the wind, then the release whips through
    // §3.2: the windup fraction drives ~30 cm of viewmodel translation
    // and ~31 deg of rotation. Read raw it snaps 0.98 -> 0 the frame the
    // spear leaves the hand (a 1-frame teleport); winding UP is tracked
    // exactly, and only the RELEASE gets a tail.
    let raw_wind = if p.gun == GunKind::Spear && p.spear_wind_t > 0.0 {
        1.0 - p.spear_wind_t / SPEAR_WINDUP_S
    } else {
        0.0
    };
    const SPEAR_WIND_RELEASE_S: f32 = 0.13;
    if raw_wind >= st.spear_wind_ease {
        st.spear_wind_ease = raw_wind;
    } else {
        st.spear_wind_ease +=
            (raw_wind - st.spear_wind_ease) * (dt / SPEAR_WIND_RELEASE_S).min(1.0);
    }
    let wind = st.spear_wind_ease;
    // §2.3: ROTATIONAL recoil that SNAPS back - kick is ~70% pitch-up /
    // 30% roll, recovered inside 140 ms regardless of the gun's cadence;
    // translation kick capped at 1.5 cm. The gun never wanders.
    // §1.3 (Brief VI): return window tightened to 120 ms
    let kick_vm = if p.armed() && p.fire_cd > 0.0 {
        ((VM_KICK_RETURN_S - (spec.fire_period - p.fire_cd)) / VM_KICK_RETURN_S)
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    // §2.4: the inspect pose (T) - image-3 angled presentation with a
    // slow idle drift; any combat input cancels instantly
    if keys.just_pressed(KeyCode::KeyT) {
        st.inspect = !st.inspect;
    }
    if cam_ctl.ads || p.fire_cd > 0.0 || p.reload_t > 0.0 || p.switch_t > 0.0 || sprinting {
        st.inspect = false;
    }
    {
        let dir = if st.inspect { 1.0 } else { -1.0 };
        st.inspect_t = (st.inspect_t + dir * dt / 0.35).clamp(0.0, 1.0);
    }
    let ie = ease_out(st.inspect_t);
    let drift = (time.elapsed_secs() * 0.7).sin() * 0.05 * ie;
    // §0 (Brief VII) / Rule 1 (Brief VI): standard guns get ZERO aim-shift
    // as a LOCAL guarantee, not an emergent side effect of `cam.ads` being
    // gated elsewhere (input_and_step) - if that gate ever changes, this
    // one still holds. Only scoped weapons (AWM - moot, hidden once
    // scoped) and projectile weapons (bow/spear draw/raise, pending their
    // real Sections 2/3 poses) ever see this ramp at all.
    let ads_capable = spec.scoped || spec.projectile.is_some();
    let ads_e = if ads_capable { ease_out(cam_ctl.ads_t) } else { 0.0 };
    let ads_shift = Vec3::new(-0.11, 0.052, 0.10) * ads_e;
    // §3 (Brief IV): the signature reload replaces the flat dip - the
    // hands and weapon do the acting, on the sim's own reload clock
    let (rl_t, rl_e) = if reloading && p.armed() {
        reload_pose(p.gun, 1.0 - (p.reload_t / spec.reload_s).clamp(0.0, 1.0))
    } else {
        (Vec3::ZERO, Vec3::ZERO)
    };
    // §6 (Brief IV): the melee swing reads in first person - wind pulls
    // the hands up-right, the strike whips across the screen. The axe
    // version is slower and twice as heavy.
    let (mel_t, mel_e) = if p.knife_phase > 0.0 && p.gun == GunKind::Spear {
        // §2 (Brief V): the first-person THRUST - the spear draws back
        // through the load, drives past full extension (overshoot), and
        // settles. The whiff holds the recovery longer via the sim clock.
        let w = THRUST_WIND_S;
        let ph = p.knife_phase;
        if ph < w {
            let e = ease_out((ph / w).clamp(0.0, 1.0));
            (
                Vec3::new(0.02, -0.012, 0.11 * e),
                Vec3::new(-0.06 * e, 0.10 * e, 0.05 * e),
            )
        } else {
            let r = ((ph - w) / (THRUST_ACTIVE_S + THRUST_RECOVER_HIT_S))
                .clamp(0.0, 1.0);
            let jab = if r < 0.3 {
                ease_out(r / 0.3) * 1.08 // past full reach - the overshoot
            } else {
                1.08 - 0.34 * ease_out((r - 0.3) / 0.7) // and the settle
            };
            (
                Vec3::new(-0.012, 0.006, -0.26 * jab),
                Vec3::new(0.055 * jab, -0.045 * jab, -0.02 * jab),
            )
        }
    } else if p.knife_phase > 0.0 {
        let axe = p.melee_axe;
        let w = if axe { AXE_QUICK_WIND_S } else { KNIFE_QUICK_WIND_S };
        let total = w
            + if axe {
                AXE_QUICK_ACTIVE_S + AXE_QUICK_RECOVER_S
            } else {
                KNIFE_QUICK_ACTIVE_S + KNIFE_QUICK_RECOVER_S
            };
        let ph = p.knife_phase;
        let amp = if axe { 1.0 } else { 0.5 };
        if ph < w {
            let e = ease_out((ph / w).clamp(0.0, 1.0)) * amp;
            (
                Vec3::new(0.10 * e, 0.09 * e, 0.05 * e),
                Vec3::new(0.20 * e, -0.45 * e, 0.35 * e),
            )
        } else {
            let r = ((ph - w) / (total - w)).clamp(0.0, 1.0);
            // snap through, then settle home over the recovery
            let e = (1.0 - ease_out(r)) * amp;
            (
                Vec3::new(-0.14 * e, -0.06 * e, -0.10 * e),
                Vec3::new(-0.30 * e, 0.55 * e, -0.50 * e),
            )
        }
    } else {
        (Vec3::ZERO, Vec3::ZERO)
    };
    // §1 (Brief V): the aimed throw reads in first person too - the
    // whole frame coils back and up, deepening as the power charges
    let gr = if p.cook_t > 0.0
        && !(p.armor_set == ArmorSet::RobotSuit && p.hull > 0.0)
    {
        ease_out((p.cook_t / THROW_CHARGE_MAX_S).clamp(0.0, 1.0))
    } else {
        0.0
    };
    if let Ok((mut tf, mut vmvis)) = q.get_mut(vm.root) {
        // §1.1 Rule 2 (Brief VI): zoomed scoped-class weapon → the
        // viewmodel is not rendered at all; unscope restores next frame
        *vmvis = if vm_hidden_while_scoped(spec.scoped, cam_ctl.ads) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        tf.translation = ads_shift
            + Vec3::new(-0.06, -0.02, 0.06) * ie
            + rl_t
            + mel_t
            + Vec3::new(0.03, 0.035, -0.05) * gr
            + carry_offset(s, st.theta, p.grounded, kick_vm, sp, dip, wind);
        tf.rotation = Quat::from_rotation_y(
            sway_rad.x + sp * 0.35 - wind * 0.25 + 0.85 * ie + drift + rl_e.y + mel_e.y,
        ) * Quat::from_rotation_x(
            kick_vm * 0.16
                + breathe
                + sway_rad.y
                + st.pitch_lag
                + rl_e.x
                + mel_e.x
                - 0.12 * gr
                + sp * 0.61
                - wind * 0.55
                + 0.22 * ie,
        ) * Quat::from_rotation_z(kick_vm * 0.07 + rl_e.z + mel_e.z + 0.08 * gr);
    }
}

/// §4.2: the aiming preview for bow/spear. It calls the sim's OWN
/// `predict_arc` - never a reimplementation - so the dots trace exactly
/// the flight the projectile will take. Dots are spaced by ARC LENGTH
/// (even spacing, no bunching at the apex) and size down along the arc;
/// a ±spread cone of fainter arcs widens as the §4 stability degrades,
/// so the player can watch their own accuracy in real time.
#[allow(clippy::too_many_arguments)]
fn arc_preview(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    arc: Res<ArcVis>,
    mut arc_state: ResMut<ArcState>,
    mut prev_yaw: Local<Option<f32>>,
    cam_q: Query<&Transform, With<MainCam>>,
    mut q: Query<(&mut Transform, &mut Visibility), Without<MainCam>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let spec = gun(p.gun);
    let show = cam_ctl.ads && p.alive() && spec.projectile.is_some() && p.roll_t <= 0.0;
    // client-side yaw rate for the cone width (mirrors the sim's model)
    let dt = time.delta_secs().max(1e-4);
    let yaw_rate = prev_yaw
        .map(|py| (wrap_angle(cam_ctl.yaw - py) / dt).abs())
        .unwrap_or(0.0);
    *prev_yaw = Some(cam_ctl.yaw);
    if !show {
        arc_state.range = None;
        for e in arc
            .dots
            .iter()
            .chain(arc.cone.iter())
            .chain([&arc.ring, &arc.drop_line])
        {
            if let Ok((_, mut v)) = q.get_mut(*e) {
                *v = Visibility::Hidden;
            }
        }
        return;
    }
    let Ok(cam_tf) = cam_q.get_single() else {
        return;
    };
    // the same launch the sim will use: same two-stage aim, same charge
    let eye = Vec3::from_array(game.sim.muzzle_origin(game.sim.player));
    let (d, _) = crosshair_aim_dir(&game.sim, cam_tf);
    let is_spear = p.gun == GunKind::Spear;
    let settled = cam_ctl.ads_t > 0.9;
    let (v0_full, _) = spec.projectile.unwrap();
    // §4.1: the BOW does not launch at its GunSpec speed. The player's
    // release goes through `step_bow_draw` -> `spawn_arrow`, which uses
    // `BOW_V0_FULL * bow_power_fraction(draw)` - 19.25 m/s at a fresh
    // draw up to 55 at full. Reading the spec's legacy 52.0 drew a
    // preview that was wildly long early in the draw AND completely
    // static across it, hiding the one thing the draw mechanic exists to
    // teach. `gun(Bow).projectile` is now only the bots' path.
    let v0 = if p.gun == GunKind::Bow {
        BOW_V0_FULL * bow_power_fraction(p.bow_draw_t).unwrap_or(BOW_POWER_MIN)
    } else if is_spear && !settled {
        SPEAR_V0_MIN
    } else {
        v0_full
    };
    // The sim's OWN cone, not a parallel copy of it. The copy that used
    // to live here had already drifted: it applied `spread_move` as a
    // hard on/off at 0.5 m/s instead of the sim's 34%->95% ramp, and read
    // `spec.spread` where the sim uses `base_spread` (heat-aware). A
    // drawn bow is aimed, matching `step_bow_draw`'s own ADS-true call.
    let moving = (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt();
    let spread = game
        .sim
        .aim_spread_of(game.sim.player, settled || p.gun == GunKind::Bow);

    let place_arc = |q: &mut Query<(&mut Transform, &mut Visibility), Without<MainCam>>,
                     ents: &[Entity],
                     dir: Vec3,
                     scale0: f32|
     -> ([f32; 3], [f32; 3], f32) {
        let (pts, impact, normal) = game.sim.predict_arc(
            [eye.x, eye.y, eye.z],
            [dir.x, dir.y, dir.z],
            v0,
            is_spear,
            8.0,
        );
        // arc-length resample: even spacing along the flight
        let mut cum = vec![0.0_f32];
        for w in pts.windows(2) {
            let d2 = Vec3::from_array(w[1]) - Vec3::from_array(w[0]);
            cum.push(cum.last().unwrap() + d2.length());
        }
        let total = *cum.last().unwrap_or(&0.0);
        for (i, e) in ents.iter().enumerate() {
            let Ok((mut t, mut v)) = q.get_mut(*e) else {
                continue;
            };
            if pts.len() < 2 || total < 0.4 {
                *v = Visibility::Hidden;
                continue;
            }
            let want = total * (i as f32 + 0.5) / ents.len() as f32;
            let k = cum.partition_point(|&c| c < want).min(pts.len() - 1);
            let frac = i as f32 / ents.len() as f32;
            t.translation = Vec3::from_array(pts[k]);
            t.scale = Vec3::splat(scale0 * (1.0 - 0.55 * frac)); // size down
            *v = Visibility::Visible;
        }
        let range = (Vec3::from_array(impact) - eye).length();
        (impact, normal, range)
    };

    // ±spread cone first (under the main arc): pitch the direction
    let up_dir = perturb_v(d, spread);
    let dn_dir = perturb_v(d, -spread);
    place_arc(&mut q, &arc.cone[..8], up_dir, 0.8);
    place_arc(&mut q, &arc.cone[8..], dn_dir, 0.8);
    let (impact, normal, range) = place_arc(&mut q, &arc.dots, d, 1.0);
    arc_state.range = Some(range);

    // landing ring: oriented to the surface; on a valid target it
    // THICKENS (shape change - §0.4 forbids a colour change)
    let near_enemy = game.sim.fighters.iter().enumerate().any(|(j, g)| {
        j != game.sim.player
            && g.team != p.team
            && g.alive()
            && {
                let dx = g.pos[0] - impact[0];
                let dz = g.pos[2] - impact[2];
                dx * dx + dz * dz < 1.3 * 1.3 && (impact[1] - g.pos[1]).abs() < 2.2
            }
    });
    if let Ok((mut t, mut v)) = q.get_mut(arc.ring) {
        let n = Vec3::from_array(normal).normalize_or(Vec3::Y);
        t.translation = Vec3::from_array(impact) + n * 0.05;
        t.rotation = Quat::from_rotation_arc(Vec3::Y, n);
        t.scale = if near_enemy {
            Vec3::new(1.0, 3.0, 1.0) // filled-in read: taller, denser ring
        } else {
            Vec3::ONE
        };
        *v = Visibility::Visible;
    }
    // drop-line from the marker straight down to the ground
    if let Ok((mut t, mut v)) = q.get_mut(arc.drop_line) {
        let h = impact[1].max(0.0);
        if h > 0.4 {
            t.translation = Vec3::new(impact[0], h * 0.5, impact[2]);
            t.scale = Vec3::new(1.0, h, 1.0);
            *v = Visibility::Visible;
        } else {
            *v = Visibility::Hidden;
        }
    }
}

/// Pitch a direction up/down by `angle` radians in its vertical plane -
/// used for the preview's ±spread cone.
fn perturb_v(d: Vec3, angle: f32) -> Vec3 {
    let flat = Vec3::new(d.x, 0.0, d.z).normalize_or(Vec3::Z);
    let axis = flat.cross(Vec3::Y).normalize_or(Vec3::X);
    Quat::from_axis_angle(axis, -angle) * d
}

/// The AWM's full-screen glass (§4): curtains + lens ring + fine cross,
/// shown only while scoped in. The FOV drop rides the normal ADS path.
fn scope_overlay(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    mut prev_show: Local<bool>,
    mut settle_t: Local<f32>,
    mut q: Query<(&mut Visibility, &mut Node), With<ScopeRoot>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let show =
        cam_ctl.ads && p.alive() && gun(p.gun).scoped && p.roll_t <= 0.0 && !p.shield_up;
    // §9 (Brief III): a 0.06 s scope-in settle - the reticle drifts in
    // and STOPS, giving the eye something to lock onto. Cosmetic only;
    // the shot ray never moves.
    if show && !*prev_show {
        *settle_t = 0.06;
    }
    *prev_show = show;
    *settle_t = (*settle_t - time.delta_secs()).max(0.0);
    let k = *settle_t / 0.06;
    let drift = 7.0 * k * k;
    for (mut v, mut node) in &mut q {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.left = Val::Px(drift);
        node.top = Val::Px(drift * 0.6);
    }
}

/// §5 detail LOD: the extra greebles on every weapon model appear only
/// while aiming (they inherit their weapon's visibility otherwise).
fn ads_detail(cam_ctl: Res<CamCtl>, mut q: Query<&mut Visibility, With<AdsDetail>>) {
    for mut v in &mut q {
        *v = if cam_ctl.ads {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// World-space checkpoint rings tint to their owner.
fn checkpoint_rings(
    game: Res<Game>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q: Query<(&CheckpointVis, &MeshMaterial3d<StandardMaterial>)>,
) {
    for (cv, mat) in &q {
        let Some(cp) = game.sim.checkpoints.get(cv.index) else {
            continue;
        };
        if let Some(m) = materials.get_mut(&mat.0) {
            let (r, g, b) = match cp.owner {
                Some(Team::Blue) => (0.25, 0.5, 1.0),
                Some(Team::Red) => (1.0, 0.3, 0.25),
                None => (0.85, 0.85, 0.9),
            };
            m.base_color = Color::srgba(r, g, b, 0.30);
            m.emissive = LinearRgba::new(r * 1.2, g * 1.2, b * 1.2, 1.0);
        }
    }
}

/// The minimap (§12): teammates, objectives, and your facing - M or the
/// settings page toggles it.
#[allow(clippy::type_complexity)]
fn minimap_system(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    state: Res<State<GameState>>,
    mut settings: ResMut<GameSettings>,
    mut spotted: ResMut<SpottedEnemies>,
    time: Res<Time>,
    mut qs: ParamSet<(
        Query<&mut Visibility, With<MinimapRoot>>,
        Query<(&MinimapDot, &mut Node, &mut Visibility)>,
        Query<(&MinimapCp, &mut Node, &mut BorderColor, &mut Visibility)>,
        Query<(&mut Node, &mut Visibility), With<MinimapHill>>,
        Query<(&mut Node, &mut Transform), With<MinimapPlayer>>,
        Query<(&MinimapEnemyDot, &mut Node, &mut BackgroundColor, &mut Visibility)>,
    )>,
) {
    // the M hotkey only means "minimap" during actual play - not while
    // typing nothing in particular on a menu screen
    if keys.just_pressed(KeyCode::KeyM) && *state.get() == GameState::Playing {
        settings.minimap = !settings.minimap;
    }
    let in_match = matches!(state.get(), GameState::Playing | GameState::Paused);
    let show = settings.minimap && in_match;
    for mut v in &mut qs.p0() {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !show {
        return;
    }
    let simr = &game.sim;
    let half = simr.half;
    let px = MINIMAP_PX - 10.0;
    // §9: the horizontal axis is MIRRORED, because in this game's yaw
    // convention screen-right is -X when facing +Z (camera_system derives
    // `screen_right = -right`, and damage_indicator honours the same
    // rule). Mapping +X to map-right drew every teammate, enemy and
    // objective on the wrong side: glance down, see an ally on your left,
    // turn left, and they are actually on your right.
    let to_map = |x: f32, z: f32| {
        (
            (half - x) / (half * 2.0) * px,
            (1.0 - (z + half) / (half * 2.0)) * px,
        )
    };
    // teammates (the player has their own gold marker)
    let p_team = simr.fighters[simr.player].team;
    let mates: Vec<&Fighter> = simr
        .fighters
        .iter()
        .enumerate()
        .filter(|(i, f)| *i != simr.player && f.team == p_team && f.alive())
        .map(|(_, f)| f)
        .collect();
    for (dot, mut node, mut v) in &mut qs.p1() {
        match mates.get(dot.0) {
            Some(f) => {
                let (u, w) = to_map(f.pos[0], f.pos[2]);
                node.left = Val::Px(u);
                node.top = Val::Px(w);
                *v = Visibility::Inherited;
            }
            None => *v = Visibility::Hidden,
        }
    }
    // §4.3: spotted enemies - a real LOS query (the same `los_clear`
    // every other visibility-gated system uses), ghost-fading to last
    // known position once sight is lost. Purely a client-visible
    // effect - never read by sim.rs, so real delta-time decay is fine
    // here even though the sim itself is fixed-tick.
    {
        let dt = time.delta_secs();
        let me = &simr.fighters[simr.player];
        let eye = [me.pos[0], me.pos[1] + EYE_REL.min(me.height() - 0.12), me.pos[2]];
        for slot in spotted.slots.iter_mut() {
            if slot.fighter.is_some() {
                slot.fade = (slot.fade - dt / MINIMAP_GHOST_FADE_S).max(0.0);
                if slot.fade <= 0.0 {
                    slot.fighter = None;
                }
            }
        }
        for (i, f) in simr.fighters.iter().enumerate() {
            if i == simr.player || f.team == p_team || !f.alive() {
                continue;
            }
            let chest = [f.pos[0], f.pos[1] + f.height() * 0.55, f.pos[2]];
            if !simr.los_clear(eye, chest) {
                continue; // not currently visible - existing slots just decay above
            }
            if let Some(slot) = spotted.slots.iter_mut().find(|s| s.fighter == Some(i)) {
                slot.pos = Vec2::new(f.pos[0], f.pos[2]);
                slot.fade = 1.0;
            } else if let Some(slot) = spotted.slots.iter_mut().find(|s| s.fighter.is_none()) {
                slot.fighter = Some(i);
                slot.pos = Vec2::new(f.pos[0], f.pos[2]);
                slot.fade = 1.0;
            }
        }
    }
    for (dot, mut node, mut bg, mut v) in &mut qs.p5() {
        let slot = spotted.slots[dot.0];
        if slot.fighter.is_some() && slot.fade > 0.0 {
            let (u, w) = to_map(slot.pos.x, slot.pos.y); // .y holds world Z
            node.left = Val::Px(u);
            node.top = Val::Px(w);
            *bg = BackgroundColor(Color::srgba(1.0, 0.25, 0.2, slot.fade));
            *v = Visibility::Inherited;
        } else {
            *v = Visibility::Hidden;
        }
    }
    // objectives: forward-spawn rings, colored by owner
    for (cp, mut node, mut border, mut v) in &mut qs.p2() {
        match simr.checkpoints.get(cp.0) {
            Some(c) => {
                let (u, w) = to_map(c.pos[0], c.pos[2]);
                node.left = Val::Px(u - 2.0);
                node.top = Val::Px(w - 2.0);
                *border = BorderColor(match c.owner {
                    Some(Team::Blue) => Color::srgb(0.3, 0.6, 1.0),
                    Some(Team::Red) => Color::srgb(1.0, 0.35, 0.3),
                    None => Color::WHITE,
                });
                *v = Visibility::Inherited;
            }
            None => *v = Visibility::Hidden,
        }
    }
    // the hill (KOTH only)
    for (mut node, mut v) in &mut qs.p3() {
        if simr.mode == Mode::Koth {
            let (u, w) = to_map(simr.hill[0], simr.hill[2]);
            node.left = Val::Px(u);
            node.top = Val::Px(w);
            *v = Visibility::Inherited;
        } else {
            *v = Visibility::Hidden;
        }
    }
    // YOU + facing needle
    let me = &simr.fighters[simr.player];
    for (mut node, mut tfm) in &mut qs.p4() {
        let (u, w) = to_map(me.pos[0], me.pos[2]);
        node.left = Val::Px(u);
        node.top = Val::Px(w);
        tfm.rotation = Quat::from_rotation_z(me.yaw);
    }
}

// ---- §1.2 debug: hit-zone bands ------------------------------------------

/// F3 toggles translucent rings at the sim's hit-zone boundaries
/// (legs <0.35< torso <0.66< arms <0.82< head - `apply_hit`) on the local
/// player and the nearest living enemy. Every rig geometry change gets
/// eyeballed against these lines: the sim is not to be changed, the model
/// fits the bands.
#[derive(Resource, Default)]
struct DebugZones {
    zones: bool,
    /// §1.3 (Brief IV) F4 gap view: joint pivots rendered as markers so
    /// any daylight between segments is immediately visible.
    joints: bool,
}

fn zone_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut dz: ResMut<DebugZones>,
    game: Res<Game>,
    rigs: Query<&FighterRig>,
    gt: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    if keys.just_pressed(KeyCode::F3) {
        dz.zones = !dz.zones;
    }
    if keys.just_pressed(KeyCode::F4) {
        dz.joints = !dz.joints;
    }
    if dz.joints {
        for rig in &rigs {
            for e in rig
                .leg_l
                .iter()
                .chain(rig.leg_r.iter())
                .chain(rig.arm_l.iter())
                .chain(rig.arm_r.iter())
                .chain([rig.torso, rig.neck, rig.weapon_root].iter())
            {
                if let Ok(g) = gt.get(*e) {
                    gizmos.sphere(
                        Isometry3d::from_translation(g.translation()),
                        0.03,
                        Color::srgb(1.0, 0.2, 0.2),
                    );
                }
            }
        }
    }
    if !dz.zones {
        return;
    }
    let simr = &game.sim;
    let me = simr.player;
    let mpos = simr.fighters[me].pos;
    let mteam = simr.fighters[me].team;
    let mut targets = vec![me];
    targets.extend(
        simr.fighters
            .iter()
            .enumerate()
            .filter(|(i, f)| *i != me && f.team != mteam && f.alive())
            .min_by(|(_, a), (_, b)| {
                let da = (a.pos[0] - mpos[0]).powi(2) + (a.pos[2] - mpos[2]).powi(2);
                let db = (b.pos[0] - mpos[0]).powi(2) + (b.pos[2] - mpos[2]).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i),
    );
    for &i in &targets {
        let f = &simr.fighters[i];
        if !f.alive() {
            continue;
        }
        let h = f.height();
        for (frac, col) in [
            (0.35, Color::srgb(0.30, 0.90, 0.40)), // legs → torso
            (0.66, Color::srgb(0.95, 0.95, 0.95)), // torso → arms
            (0.82, Color::srgb(1.00, 0.60, 0.15)), // arms → head
            (1.00, Color::srgb(1.00, 0.25, 0.20)), // crown
        ] {
            let y = f.pos[1] + h * frac;
            gizmos.circle(
                Isometry3d::new(
                    Vec3::new(f.pos[0], y, f.pos[2]),
                    Quat::from_rotation_x(FRAC_PI_2),
                ),
                BODY_RADIUS + 0.06,
                col,
            );
        }
    }
}

// ------------------------------------------------------------------- sound

/// §8.2 (Brief III): the distance model - sound arrives at 343 m/s. At
/// 100 m the muzzle flash leads the report by ~0.29 s, and that gap is
/// most of why gunfire in a big space feels PHYSICAL rather than like a
/// sound effect. Every non-player shot queues with its travel delay and
/// a distance-blended volume.
#[derive(Resource, Default)]
struct DistantShots {
    prev_cd: Vec<f32>,
    queue: Vec<(f32, GunKind, f32)>,
}

fn distant_gunfire(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    sfx: Res<Sfx>,
    mut st: ResMut<DistantShots>,
) {
    let now = time.elapsed_secs();
    let simr = &game.sim;
    let me = simr.fighters[simr.player].pos;
    st.prev_cd.resize(simr.fighters.len(), 0.0);
    for i in 0..simr.fighters.len() {
        let f = &simr.fighters[i];
        let prev = st.prev_cd[i];
        st.prev_cd[i] = f.fire_cd;
        if i == simr.player || f.fire_cd <= prev {
            continue; // no new shot from this fighter this frame
        }
        let dx = f.pos[0] - me[0];
        let dy = f.pos[1] - me[1];
        let dz = f.pos[2] - me[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        let delay = dist / 343.0; // the speed of sound
        let vol = (1.0 - dist / 140.0).clamp(0.08, 0.9);
        let gun_now = f.gun;
        st.queue.push((now + delay, gun_now, vol));
    }
    st.queue.retain(|&(at, gun_k, vol)| {
        if at <= now {
            play(&mut commands, shot_sound(&sfx, gun_k), vol);
            false
        } else {
            true
        }
    });
}

fn play(commands: &mut Commands, h: &Handle<AudioSource>, vol: f32) {
    commands.spawn((
        AudioPlayer::new(h.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::new(vol)),
    ));
}

fn sfx_system(
    mut commands: Commands,
    game: Res<Game>,
    sfx: Res<Sfx>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    settings: Res<GameSettings>,
    mut st: ResMut<SfxState>,
) {
    let simr = &game.sim;
    // rematch / restart: sim clock went backwards - reset the tracker
    if simr.t < st.last_t {
        *st = SfxState::default();
    }
    // An event is fresh if it was created since the last sfx run. Its age
    // and `elapsed` advance in exact DT lockstep, so the epsilon must be
    // SUBTRACTED - adding it replays every event a second time.
    let elapsed = (simr.t - st.last_t).max(0.0);
    let fresh = |age: f32| age < elapsed - 1e-4;
    let p = &simr.fighters[simr.player];
    let p_team = p.team;

    // your own shot - fire_cd jumps up on the tick you fire (an ammo delta
    // misses the shot fired the same tick a reload completes)
    if p.alive() && p.gun == st.prev_gun && p.fire_cd > st.prev_fire_cd {
        play(&mut commands, shot_sound(&sfx, p.gun), 0.8);
    }
    // dry fire: trigger down on an empty magazine → the click (whatever
    // the reserve says - an empty chamber clicks until you reload)
    st.click_cd = (st.click_cd - elapsed).max(0.0);
    // the shared mapping, and NOT T: T has been inspect since Brief VI,
    // so holding it on an empty mag played a dry-fire click for a trigger
    // pull that never happened
    let fire_held = buttons.pressed(mouse_map(settings.swap_mouse).1);
    if fire_held
        && p.alive()
        && p.armed()
        && p.ammo == 0
        && p.reload_t <= 0.0
        && !p.shield_up
        && st.click_cd <= 0.0
    {
        play(&mut commands, &sfx.click, 0.5);
        st.click_cd = 0.35;
    }
    // shield raised / lowered: the plate clanks onto the arm (gated
    // across the respawn reset so rebirth doesn't phantom-clank)
    if st.prev_alive && p.alive() && p.shield_up != st.prev_shield {
        play(&mut commands, &sfx.shield, 0.35);
    }
    // everyone else's gunfire (hitscan tracers), quieter with distance
    let mut bot_shots = 0;
    for tr in &simr.tracers {
        if !fresh(0.06 - tr.ttl) {
            continue;
        }
        let d2 = (tr.from[0] - p.pos[0]).powi(2)
            + (tr.from[1] - (p.pos[1] + EYE_REL)).powi(2)
            + (tr.from[2] - p.pos[2]).powi(2);
        if tr.team == p_team && d2 < 1.0 {
            continue; // that's your own muzzle (already played above)
        }
        if bot_shots < 2 {
            let vol = (1.0 - (d2.sqrt() / 60.0)).clamp(0.05, 0.3);
            play(&mut commands, &sfx.shot_rifle, vol);
            bot_shots += 1;
        }
    }
    // new arrows / spears in the air
    for m in &simr.missiles {
        if m.id >= st.max_missile {
            if m.shooter != simr.player {
                play(
                    &mut commands,
                    if m.is_spear { &sfx.spear } else { &sfx.bow },
                    0.25,
                );
            }
            st.max_missile = m.id + 1;
        }
    }
    // hits: confirms for you, pain for you - a struck SHIELD clanks
    // instead on both ends
    for (ev, ttl) in &simr.hits {
        if !fresh(2.2 - ttl) {
            continue;
        }
        if ev.shooter == simr.player {
            play(
                &mut commands,
                if ev.shielded {
                    &sfx.shield
                } else if ev.zone == HitZone::Head {
                    &sfx.headshot
                } else {
                    &sfx.hit
                },
                0.5,
            );
        } else if ev.victim == simr.player {
            play(
                &mut commands,
                if ev.shielded { &sfx.shield } else { &sfx.hurt },
                0.6,
            );
        }
    }
    // kills
    for (ev, ttl) in &simr.kill_feed {
        if !fresh(5.0 - ttl) {
            continue;
        }
        if ev.killer == simr.player {
            play(&mut commands, &sfx.kill, 0.55);
        }
    }
    // reload start - magazine weapons only: the bow/spear auto-nock is
    // part of the shot, not a reload foley
    if p.reload_t > 0.0
        && st.prev_reload <= 0.0
        && p.alive()
        && gun(p.gun).projectile.is_none()
    {
        play(&mut commands, &sfx.reload, 0.5);
    }
    // jump: vertical velocity kicked upward
    if p.vy > 3.0 && st.prev_vy <= 3.0 {
        play(&mut commands, &sfx.jump, 0.35);
    }
    // dodge roll / breakfall: the tumble whoosh
    if p.roll_t > 0.0 && st.prev_roll <= 0.0 {
        play(&mut commands, &sfx.roll, 0.5);
    }
    // slot swap = draw foley; ammo top-up / armor / health = pickup chime
    // (all gated to a live player so respawn resets stay silent)
    if st.prev_alive && p.alive() && p.gun != st.prev_gun {
        play(&mut commands, &sfx.reload, 0.35);
    } else if st.prev_alive && p.alive() && p.gun == st.prev_gun && p.reserve > st.prev_reserve {
        play(&mut commands, &sfx.pickup, 0.6);
    }
    if p.armor > st.prev_armor + 1.0 {
        play(&mut commands, &sfx.pickup, 0.6);
    }
    if st.prev_alive && p.alive() && p.health > st.prev_health + 2.0 {
        play(&mut commands, &sfx.pickup, 0.6);
    }
    // round end: fanfare only if YOUR team took it; a low sting for a loss
    let over = simr.round_over_t.is_some();
    if over && !st.prev_over {
        if simr.winner == Some(p_team) {
            play(&mut commands, &sfx.win, 0.7);
        } else {
            play(&mut commands, &sfx.kill, 0.45);
        }
    }

    st.last_t = simr.t;
    st.prev_gun = p.gun;
    st.prev_fire_cd = p.fire_cd;
    st.prev_reserve = p.reserve;
    st.prev_reload = p.reload_t;
    st.prev_vy = p.vy;
    st.prev_roll = p.roll_t;
    st.prev_armor = p.armor;
    st.prev_health = p.health;
    st.prev_alive = p.alive();
    st.prev_over = over;
    st.prev_shield = p.shield_up;
}

// --------------------------------------------------------------------- HUD

fn fmt_clock(t: f32) -> String {
    let t = t.max(0.0) as u32;
    format!("{}:{:02}", t / 60, t % 60)
}

fn hud_system(
    game: Res<Game>,
    settings: Res<GameSettings>,
    cam: Res<CamCtl>,
    arc_state: Res<ArcState>,
    mut texts: ParamSet<(
        Query<&mut Text, With<HudText>>,
        Query<&mut Text, With<ScoreTimerText>>,
        Query<&mut Text, With<FeedText>>,
        Query<&mut Text, With<HitFeedText>>,
        Query<&mut Text, With<BannerText>>,
        Query<&mut Text, With<PanelInfoText>>,
        Query<&mut Text, With<PanelAmmoText>>,
        Query<&mut Text, With<RangeText>>,
    )>,
    // TextColor ONLY - a &mut Text here aliases the ParamSet's eight
    // Text queries and trips B0001 at schedule init (startup crash).
    // The glyph swap lives in `crosshair_kill_pop`, its own system.
    mut cross: Query<&mut TextColor, With<CrosshairText>>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];

    if let Ok(mut t) = texts.p0().get_single_mut() {
        let mut s = format!(
            "K/D {}/{}   hits {}{}\n",
            p.kills,
            p.deaths,
            p.hits_dealt,
            if p.roll_t > 0.0 {
                "   [ROLLING]"
            } else if p.shield_up {
                "   [SHIELD UP]"
            } else if p.crouch {
                "   [crouched]"
            } else {
                ""
            }
        );
        // §7: no permanent controls strip - the first-run card and the
        // Controls screen teach binds; the HUD stays clean
        let _ = &settings;
        **t = s;
    }

    if let Ok(mut t) = texts.p1().get_single_mut() {
        let clock = fmt_clock(simr.match_t);
        let head = if simr.overtime {
            format!("OVERTIME {clock} - sudden death")
        } else {
            clock
        };
        let score = match simr.mode {
            Mode::Tdm => format!(
                "BLUE {:>2} - {:<2} RED   (first to {})",
                simr.score[0] as u32, simr.score[1] as u32, TDM_TARGET
            ),
            Mode::Koth => format!(
                "HILL   BLUE {:>3.0}s - {:<3.0}s RED   (hold {:.0}s)",
                simr.score[0], simr.score[1], KOTH_TARGET_S
            ),
            Mode::Extraction => {
                // §8: the run readout - horde count, pressure, objective
                let obj = match simr.extract_point() {
                    None => "extraction reveals at 4:00".to_string(),
                    Some(_) if simr.extract_hold > 0.0 => format!(
                        "HOLD THE RING  {:.0}/{:.0}s",
                        simr.extract_hold, EXTRACT_HOLD_S
                    ),
                    Some(p2) => format!("EXTRACT at ({:.0}, {:.0})", p2[0], p2[2]),
                };
                format!(
                    "HORDE {:>2}   pressure {:>3.0}%   {obj}",
                    simr.zombies.len(),
                    simr.pressure * 100.0
                )
            }
        };
        **t = format!("{head}\n{score}");
    }

    if let Ok(mut t) = texts.p2().get_single_mut() {
        let mut s = String::new();
        // §3.5 (Brief VI): newest at the BOTTOM, max 5 rows, modifier
        // glyphs, and rows involving the local player marked with a bar.
        // §0 (Brief VII): ASCII only - U+25B6/U+258C had no font glyph.
        let rows: Vec<_> = simr.kill_feed.iter().rev().take(5).collect();
        for (ev, _) in rows.into_iter().rev() {
            let me = ev.killer == simr.player || ev.victim == simr.player;
            // §4.5: "Killer [+Assist] [glyph] Victim" - the assist tag
            // only appears when there was one, never an empty bracket
            let assist_tag = match ev.assist {
                Some(a) => format!(" +{}", simr.fighters[a].name),
                None => String::new(),
            };
            s += &format!(
                "{}{}{} {}> {}\n",
                if me { "| " } else { "" },
                simr.fighters[ev.killer].name,
                assist_tag,
                feed_glyphs(ev.headshot),
                simr.fighters[ev.victim].name,
            );
        }
        **t = s;
    }

    if let Ok(mut t) = texts.p3().get_single_mut() {
        let mut s = String::new();
        for (ev, _) in simr.hits.iter().rev().take(4) {
            if ev.shooter == simr.player {
                s += &format!(
                    "You hit {} - {} ({:.0}){}\n",
                    simr.fighters[ev.victim].name,
                    ev.zone.name(),
                    ev.damage,
                    if ev.fatal { "  KILL" } else { "" }
                );
            } else if ev.victim == simr.player {
                s += &format!(
                    "{} hit you - {}\n",
                    simr.fighters[ev.shooter].name,
                    ev.zone.name()
                );
            }
        }
        **t = s;
    }

    if let Ok(mut t) = texts.p4().get_single_mut() {
        **t = match simr.winner {
            Some(Team::Blue) => "BLUE WINS - rematch shortly".to_string(),
            Some(Team::Red) => "RED WINS - rematch shortly".to_string(),
            // §5.3 (Brief VI): the victim's warning, live from lock
            // START - counterplay begins before the missile exists
            // §0 (Brief VII): the bundled default font has no glyph for
            // U+26A0 - it rendered as a tofu box in every capture. ASCII
            // reads correctly in any font.
            None if p.lock_warn_t > 0.0 => "! MISSILE LOCK !".to_string(),
            None => String::new(),
        };
    }

    // status panel: loadout slots, weapon, ammo kind, HP/armor numbers
    if let Ok(mut t) = texts.p5().get_single_mut() {
        **t = if !p.alive() {
            format!("DOWN - respawn in {:.1}s", p.respawn_t.max(0.0))
        } else {
            // §9.2 (Brief IV): the regen box - countdown while the 12 s
            // clock runs, a pulsing + while healing, hidden at full
            let regen = if p.health >= MAX_HEALTH - 0.01 {
                String::new()
            } else {
                let since = simr.t - p.last_dmg_at;
                if since < REGEN_DELAY_S {
                    format!("   [{:.0}]", (REGEN_DELAY_S - since).ceil())
                } else if (simr.t * 3.0) as i32 % 2 == 0 {
                    "   [+]".to_string()
                } else {
                    "   [ ]".to_string()
                }
            };
            // §3.1 (Brief VI): vitals cluster, bottom-left - HP is the
            // largest number on screen; armor status beside it.
            // §0 (Brief VII): U+271A had no glyph in the bundled font -
            // rendered as a tofu box; ASCII '+' is guaranteed to exist.
            format!(
                "+ {:.0}{regen}{}{}",
                p.health.max(0.0),
                if p.shield_up { "  [SHIELD]" } else { "" },
                match p.armor_set {
                    ArmorSet::None => String::new(),
                    ArmorSet::RobotSuit => {
                        // §4.6/§5.3: hull, power, the 4-tube indicator,
                        // and the dismount hint. §0 (Brief VII): ASCII
                        // only - U+25AE/U+25AF had no font glyph.
                        let tubes: String = (0..POD_TUBES)
                            .map(|i| if i < p.pod_ammo { '#' } else { '.' })
                            .collect();
                        format!(
                            "  MECH - HULL {:.0} - POWER {:.0} - POD {tubes} - U: dismount",
                            p.hull, p.armor
                        )
                    }
                    ArmorSet::Pyro => format!("  PYRO - FUEL {:.1}s", p.fuel),
                    ArmorSet::Folk => {
                        if p.brace {
                            "  FOLK ARMOR [BRACED]".to_string()
                        } else {
                            // brace is the armor ability (C). F is the
                            // knife - this told Folk players to stab.
                            "  FOLK ARMOR (hold C: brace)".to_string()
                        }
                    }
                    ArmorSet::Recon => "  RECON WEAVE".to_string(),
                }
            )
        };
    }
    if let Ok(mut t) = texts.p6().get_single_mut() {
        **t = if !p.alive() {
            String::new()
        } else if p.shield_up {
            "SHIELD".to_string()
        } else if p.reload_t > 0.0 {
            "RELOADING".to_string()
        } else if p.cook_t > 0.0 {
            // §5: the fuse in your hand
            let k = ThrowKind::ALL[p.throw_sel as usize];
            let left = (throw_spec(k).fuse_s - p.cook_t).max(0.0);
            if k == ThrowKind::Frag {
                format!("COOKING {left:.1}")
            } else {
                format!("{} ARMED", k.name())
            }
        } else if p.ammo_full_t > 0.0 {
            // §3: the missing feedback that hid the pickup bug
            format!("AMMO FULL   {} / {}", p.ammo, p.reserve)
        } else if p.gun == GunKind::Minigun {
            // §7 (Brief IV): the heat readout IS the minigun's HUD -
            // rounds, heat percent, and the vent state
            let k = ThrowKind::ALL[p.throw_sel as usize];
            let heat_line = if p.vent_t > 0.0 {
                format!("VENTING {:.1}s", p.vent_t.max(0.0))
            } else {
                format!("{}  HEAT {:.0}%", p.ammo, p.heat)
            };
            format!(
                "{heat_line}\n{} x{}",
                k.name(),
                p.grenades[p.throw_sel as usize]
            )
        } else {
            // §3.2 (Brief VI): the ammo cluster - weapon name above,
            // mag / reserve as the numerals, throwable below
            let k = ThrowKind::ALL[p.throw_sel as usize];
            format!(
                "{} / {}\n{} x{}",
                p.ammo,
                p.reserve,
                k.name(),
                p.grenades[p.throw_sel as usize]
            )
        };
    }

    // §4.2: range readout while the trajectory preview is live
    if let Ok(mut t) = texts.p7().get_single_mut() {
        **t = match arc_state.range {
            Some(r) => format!("{r:.0} m"),
            None => String::new(),
        };
    }

    // crosshair flash: white → gold on a fresh headshot, red on a fresh
    // hit; §5.3 amber when the muzzle→crosshair path is blocked close-by
    // (the shot will hit YOUR cover - not a mystery, a warning)
    if let Ok(mut tc) = cross.get_single_mut() {
        let fresh = simr
            .hits
            .iter()
            .rev()
            .find(|(ev, ttl)| ev.shooter == simr.player && *ttl > 2.0);
        let fresh_kill = simr
            .kill_feed
            .iter()
            .rev()
            .any(|(ev, ttl)| ev.killer == simr.player && *ttl > 4.5);
        // §5.2 (Brief VI): scoped-class weapons draw NO crosshair while
        // unscoped - the no-scope prayer is the tradeoff
        let noscope_hidden = gun(p.gun).scoped && !cam.ads;
        *tc = TextColor(match fresh {
            _ if noscope_hidden => Color::srgba(0.0, 0.0, 0.0, 0.0),
            _ if fresh_kill => Color::srgb(1.0, 0.55, 0.2),
            Some((ev, _)) if ev.zone == HitZone::Head => Color::srgb(1.0, 0.85, 0.2),
            Some(_) => Color::srgb(1.0, 0.3, 0.25),
            None if cam.blocked && p.alive() => Color::srgba(1.0, 0.55, 0.1, 0.9),
            None => Color::srgba(1.0, 1.0, 1.0, 0.9),
        });
    }
}

/// §3 (Brief VI): the semantic color pass - vitals red at ≤25 / pulsing
/// at ≤20, ammo red at ≤25% of the magazine, the timer red inside the
/// final 0:10. ParamSet because three &mut TextColor queries in one
/// system would alias (the B0001 lesson, learned the hard way).
fn hud_colors(
    game: Res<Game>,
    mut q: ParamSet<(
        Query<&mut TextColor, With<PanelInfoText>>,
        Query<&mut TextColor, With<PanelAmmoText>>,
        Query<&mut TextColor, With<ScoreTimerText>>,
    )>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];
    if let Ok(mut c) = q.p0().get_single_mut() {
        *c = TextColor(vitals_color(p.health.max(0.0), simr.t));
    }
    if let Ok(mut c) = q.p1().get_single_mut() {
        *c = TextColor(if ammo_is_low(p.ammo, gun(p.gun).mag) {
            Color::srgb(1.0, 0.18, 0.15)
        } else {
            Color::WHITE
        });
    }
    if let Ok(mut c) = q.p2().get_single_mut() {
        *c = TextColor(if simr.match_t <= 10.0 && simr.round_over_t.is_none() {
            Color::srgb(1.0, 0.18, 0.15)
        } else {
            Color::srgb(0.95, 0.95, 1.0)
        });
    }
}

/// §1.2 (Brief VI): the on-weapon ammo bar - segments track the live
/// magazine fraction; a completed reload pulses the whole bar once.
/// Cosmetic only; reads the same sim ammo the HUD reads.
fn ammo_bar_sync(
    game: Res<Game>,
    time: Res<Time>,
    mut q: Query<(&AmmoBarSeg, &mut Visibility, &mut Transform)>,
    mut prev_reload: Local<f32>,
    mut pulse_t: Local<f32>,
) {
    let p = &game.sim.fighters[game.sim.player];
    // reload completion edge → one 0.5 s pulse
    if *prev_reload > 0.0 && p.reload_t <= 0.0 {
        *pulse_t = 0.5;
    }
    *prev_reload = p.reload_t;
    *pulse_t = (*pulse_t - time.delta_secs()).max(0.0);
    let mag = gun(p.gun).mag.max(1) as f32;
    let frac = (p.ammo as f32 / mag).clamp(0.0, 1.0);
    let pulse = if *pulse_t > 0.0 {
        1.0 + 0.35 * (PI * (1.0 - *pulse_t / 0.5)).sin()
    } else {
        1.0
    };
    for (seg, mut vis, mut tf) in &mut q {
        let lit = *pulse_t > 0.0
            || (seg.idx as f32 + 0.5) / AMMO_BAR_SEGS as f32 <= frac;
        *vis = if lit {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        tf.scale = Vec3::new(0.004, 0.009 * pulse, 0.012 * pulse);
    }
}

/// The kill-confirm glyph pop, in its OWN system: the crosshair `+`
/// becomes an X for half a second after your kill. Separate from
/// `hud_system` because two &mut Text accesses in one system alias.
fn crosshair_kill_pop(game: Res<Game>, mut q: Query<&mut Text, With<CrosshairText>>) {
    let simr = &game.sim;
    let fresh_kill = simr
        .kill_feed
        .iter()
        .rev()
        .any(|(ev, ttl)| ev.killer == simr.player && *ttl > 4.5);
    if let Ok(mut tx) = q.get_single_mut() {
        let want = if fresh_kill { "X" } else { "+" };
        if **tx != want {
            **tx = want.to_string();
        }
    }
}

/// §10: tint the edges below 35% health; pulse gently once the regen
/// timer has elapsed so the recovery is readable without a HUD element.
fn health_vignette(
    game: Res<Game>,
    mut q: Query<&mut BackgroundColor, With<HealthVignette>>,
) {
    let Ok(mut bg) = q.get_single_mut() else {
        return;
    };
    // §3.7 (Brief VI) DELIBERATELY OFF: no persistent low-HP screen
    // vignette - CS:GO doesn't have one and it keeps the center clean.
    // Danger reads through the vitals colour (`hud_colors`) instead.
    //
    // This used to be an `if true { ... return }` guard in front of a
    // full, unreachable implementation, while comments elsewhere still
    // described the vignette as a live feature. The overlay entity is
    // kept (and held transparent here) so the decision stays reversible
    // in one place instead of being re-derived from dead code.
    let _ = &game;
    *bg = BackgroundColor(Color::srgba(0.5, 0.02, 0.02, 0.0));
}

/// §7: the compass strip - a 9-slot cardinal window over the view yaw
/// (chevrons and objective markers ride the same ring later).
fn compass_system(cam: Res<CamCtl>, mut q: Query<&mut Text, With<CompassText>>) {
    let Ok(mut t) = q.get_single_mut() else {
        return;
    };
    const RING: [&str; 16] = [
        "N", "-", "-", "-", "E", "-", "-", "-", "S", "-", "-", "-", "W", "-", "-", "-",
    ];
    let idx = ((cam.yaw / std::f32::consts::TAU * 16.0).round() as i32).rem_euclid(16);
    let mut s = String::new();
    for k in -4i32..=4 {
        let i = (idx + k).rem_euclid(16) as usize;
        if k == 0 {
            s.push('[');
            s.push_str(RING[i]);
            s.push(']');
        } else {
            s.push_str(RING[i]);
        }
        s.push(' ');
    }
    **t = s;
}

/// §7: the stability bracket widens with the live spread - bloom, stance,
/// movement, ADS all feed it. The value already exists; this just shows it.
fn stability_bracket(
    game: Res<Game>,
    cam: Res<CamCtl>,
    mut q: Query<(&StabilityBracket, &mut Node, &mut Visibility)>,
) {
    let p = &game.sim.fighters[game.sim.player];
    // The sim's OWN cone. The copy that used to live here had drifted
    // three ways: it applied `spread_move` as a hard on/off at 0.5 m/s
    // instead of the sim's 34%->95% ramp, it had no airborne penalty at
    // all, and it missed the scoped override entirely - so an AWM player
    // saw a bracket that ignored the flat 0.002 laser value the sim
    // actually shoots with.
    let spread = game.sim.aim_spread_of(game.sim.player, cam.ads_t > 0.9);
    let px = 12.0 + spread * 2400.0;
    for (b, mut node, mut vis) in &mut q {
        *vis = if p.alive() && p.armed() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.margin = UiRect::left(Val::Px(if b.0 == 0 { -px - 8.0 } else { px }));
    }
}

/// §7 (Brief III): everything except the crosshair fades to 45% opacity
/// after 4 s without a state change, and snaps back the instant a value
/// moves - the HUD gets out of the way.
#[allow(clippy::type_complexity)]
fn hud_fade(
    time: Res<Time>,
    game: Res<Game>,
    mut last: Local<(u32, u32, i32, u8)>,
    mut idle_t: Local<f32>,
    mut q: Query<
        &mut TextColor,
        Or<(With<PanelInfoText>, With<PanelAmmoText>, With<HudText>)>,
    >,
) {
    let p = &game.sim.fighters[game.sim.player];
    let snap = (
        p.ammo,
        p.reserve,
        p.health as i32,
        p.throw_sel + if p.shield_up { 100 } else { 0 },
    );
    if snap != *last {
        *last = snap;
        *idle_t = 0.0;
    } else {
        *idle_t += time.delta_secs();
    }
    let alpha = if *idle_t > 4.0 { 0.45 } else { 1.0 };
    for mut tc in &mut q {
        let mut c = tc.0.to_srgba();
        c.alpha = alpha;
        *tc = TextColor(Color::Srgba(c));
    }
}

fn scoreboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    mut root: Query<&mut Visibility, With<ScoreboardRoot>>,
    mut text: Query<&mut Text, With<ScoreboardText>>,
) {
    let show = keys.pressed(KeyCode::Tab) || game.sim.round_over_t.is_some();
    if let Ok(mut v) = root.get_single_mut() {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !show {
        return;
    }
    if let Ok(mut t) = text.get_single_mut() {
        let mut s = String::new();
        for (team, label) in [(Team::Blue, "BLUE"), (Team::Red, "RED")] {
            s += &format!(
                "{label}  -  {} pts\n{:<12}{:>4}{:>4}{:>6}   {}\n",
                match game.sim.mode {
                    Mode::Tdm => format!("{:.0}", game.sim.score[TdmSim::team_idx(team)]),
                    Mode::Koth => format!("{:.0}s", game.sim.score[TdmSim::team_idx(team)]),
                    Mode::Extraction => format!("{} horde", game.sim.zombies.len()),
                },
                "NAME",
                "K",
                "D",
                "HITS",
                "WEAPON"
            );
            let mut rows: Vec<&Fighter> = game
                .sim
                .fighters
                .iter()
                .filter(|f| f.team == team)
                .collect();
            rows.sort_by(|a, b| b.kills.cmp(&a.kills).then(a.deaths.cmp(&b.deaths)));
            for f in rows {
                s += &format!(
                    "{:<12}{:>4}{:>4}{:>6}   {}\n",
                    f.name,
                    f.kills,
                    f.deaths,
                    f.hits_dealt,
                    gun(f.gun).name
                );
            }
            s += "\n";
        }
        **t = s;
    }
}

fn wrap_angle(a: f32) -> f32 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}

/// When you're hit, the screen edge facing the shooter flashes red - you
/// always know which way the bullet came from.
fn damage_indicator(
    game: Res<Game>,
    cam: Res<CamCtl>,
    mut edges: Query<(&DmgEdge, &mut BackgroundColor)>,
) {
    let pi = game.sim.player;
    let ppos = game.sim.fighters[pi].pos;
    let mut inten = [0.0_f32; 4]; // top(front) right bottom(behind) left
    for (ev, ttl) in &game.sim.hits {
        if ev.victim != pi {
            continue;
        }
        let w = (ttl / 2.2).clamp(0.0, 1.0);
        let dx = ev.from[0] - ppos[0];
        let dz = ev.from[2] - ppos[2];
        if dx * dx + dz * dz < 1e-4 {
            continue;
        }
        let rel = wrap_angle(dx.atan2(dz) - cam.yaw);
        let (c, s) = (rel.cos(), rel.sin());
        // screen-right is (−cos yaw, sin yaw) in this game's convention
        // (the playtest-verified A/D mapping), so a shooter at rel −π/2
        // sits screen-RIGHT: sin < 0 → the right strip, not the left
        let idx = if c > 0.5 {
            0
        } else if c < -0.5 {
            2
        } else if s < 0.0 {
            1
        } else {
            3
        };
        inten[idx] = inten[idx].max(w);
    }
    for (e, mut bg) in &mut edges {
        *bg = BackgroundColor(Color::srgba(
            0.85,
            0.08,
            0.08,
            inten[e.0 as usize] * 0.55,
        ));
    }
}

// ---------------------------------------------------------------- menus ---

fn esc_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next.set(GameState::Paused),
            GameState::Paused => next.set(GameState::Playing),
            GameState::Settings | GameState::Manual | GameState::Controls => {
                next.set(GameState::Paused)
            }
            GameState::Intro => {}
        }
    }
}

/// §1.2: the Controls screen - GENERATED from the keybind registry, so it
/// can never drift from what the game actually binds.
fn open_controls(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.92)),
            GlobalZIndex(30),
            ControlsRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("CONTROLS"),
                TextFont {
                    font_size: 34.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));
            let mut body = String::new();
            for b in BIND_REGISTRY {
                body += &format!("{:<12}  {}\n", b.key, b.action);
            }
            body += "\nESC - back";
            p.spawn((
                Text::new(body),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.93, 0.95)),
            ));
        });
}

fn close_controls(mut commands: Commands, q: Query<Entity, With<ControlsRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

/// §1.2: the one-time first-run card - the non-obvious binds, dismissed
/// by any key. A feature that ships without a way to learn it has not
/// shipped; this is the systemic fix.
fn first_run_card(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut card: ResMut<FirstRunCard>,
    q: Query<Entity, With<FirstRunRoot>>,
) {
    if card.dismissed {
        return;
    }
    if !card.shown {
        card.shown = true;
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(18.0),
                    top: Val::Percent(24.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.07, 0.10, 0.85)),
                GlobalZIndex(25),
                FirstRunRoot,
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("GOOD TO KNOW"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.8, 0.3)),
                ));
                let mut body = String::new();
                for b in BIND_REGISTRY.iter().filter(|b| b.essential) {
                    body += &format!("{:<10} {}\n", b.key, b.action);
                }
                body += "\nARMOR SETS lie on glowing pads - walk over one.\nFull list: ESC > Controls.  (any key to dismiss)";
                p.spawn((
                    Text::new(body),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.92, 0.95)),
                ));
            });
        return;
    }
    // any key or click dismisses
    if keys.get_just_pressed().next().is_some() || buttons.get_just_pressed().next().is_some() {
        card.dismissed = true;
        for e in &q {
            commands.entity(e).despawn_recursive();
        }
    }
}

/// §1.2 contextual prompts: near a pad → name it and say how to take it;
/// on equipping a set → 4 s of its ability binds.
fn contextual_prompts(
    time: Res<Time>,
    game: Res<Game>,
    mut toast: ResMut<Toast>,
    mut prev_set: Local<Option<ArmorSet>>,
    mut hint_t: Local<f32>,
    mut hint: Local<String>,
    mut q: Query<&mut Text, With<PromptText>>,
) {
    let Ok(mut t) = q.get_single_mut() else {
        return;
    };
    // toasts outrank everything - they confirm a player action
    if toast.t > 0.0 {
        toast.t -= time.delta_secs();
        **t = toast.text.clone();
        return;
    }
    let p = &game.sim.fighters[game.sim.player];
    // equip hint has priority, for 4 s
    if *prev_set != Some(p.armor_set) {
        if prev_set.is_some() && p.armor_set != ArmorSet::None {
            *hint = equip_hint(p.armor_set).to_string();
            *hint_t = 4.0;
        }
        *prev_set = Some(p.armor_set);
    }
    if *hint_t > 0.0 {
        *hint_t -= time.delta_secs();
        **t = hint.clone();
        return;
    }
    // otherwise: the nearest live pickup within 3 m announces itself
    let mut best: Option<(f32, PickupKind)> = None;
    for pk in &game.sim.pickups {
        if pk.respawn_t > 0.0 {
            continue;
        }
        let dx = p.pos[0] - pk.pos[0];
        let dz = p.pos[2] - pk.pos[2];
        let d2 = dx * dx + dz * dz;
        if d2 < 3.0 * 3.0 && best.map_or(true, |(b, _)| d2 < b) {
            best = Some((d2, pk.kind));
        }
    }
    **t = match best {
        Some((_, kind)) => pickup_prompt(kind).to_string(),
        None => String::new(),
    };
}

/// One row of pick-buttons on the loadout screen.
fn pick_row<C: Component + Copy>(
    p: &mut ChildBuilder,
    label: &str,
    items: &[(&str, C)],
    w: f32,
) {
    p.spawn((Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(8.0),
        align_items: AlignItems::Center,
        ..default()
    },))
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.7, 0.9)),
                Node {
                    width: Val::Px(110.0),
                    ..default()
                },
            ));
            for (name, comp) in items {
                row.spawn((
                    Button,
                    *comp,
                    Node {
                        width: Val::Px(w),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.14, 0.17, 0.22)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(name.to_string()),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

/// The intro is the LOADOUT SCREEN now (§6, §13): weapons per slot,
/// cosmetics, battlefield, difficulty, team size - then deploy.
/// §7.2 (Brief VII v2): the Forge's save/load - Ctrl+1/2/3 saves the
/// current cosmetic+loadout choices into a slot, 1/2/3 alone loads one
/// back. Reachable from the lobby (Intro state) where `Selected` is
/// being built, exactly where the brief wants it.
fn forge_keybinds(
    keys: Res<ButtonInput<KeyCode>>,
    mut sel: ResMut<Selected>,
    mut toast: ResMut<Toast>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    for (slot, key) in [(1usize, KeyCode::Digit1), (2, KeyCode::Digit2), (3, KeyCode::Digit3)] {
        if !keys.just_pressed(key) {
            continue;
        }
        if ctrl {
            let p = ForgeProfile::from_selected(&sel);
            match forge_save(slot, &p) {
                Ok(()) => {
                    toast.text = format!("FORGE: saved to slot {slot}");
                    toast.t = 1.8;
                }
                Err(_) => {
                    toast.text = format!("FORGE: could not save slot {slot}");
                    toast.t = 1.8;
                }
            }
        } else if let Some(p) = forge_load(slot) {
            p.apply_to(&mut sel);
            toast.text = format!("FORGE: loaded slot {slot}");
            toast.t = 1.8;
        } else {
            toast.text = format!("FORGE: slot {slot} is empty");
            toast.t = 1.8;
        }
    }
}

fn open_intro(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
    cam.ads = false;
    // §14: the tech readout - real numbers, pulled from the live table
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgb(0.62, 0.85, 0.90)),
        // capped width: the centered mode buttons are 620 wide, so on a
        // 1280-wide window they start around x=330 - an unconstrained
        // readout ran straight underneath them and the two texts
        // overlapped illegibly.
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            bottom: Val::Px(16.0),
            max_width: Val::Px(300.0),
            ..default()
        },
        GlobalZIndex(12),
        TechReadout,
    ));
    // the Forge's save/load confirmations, where the Forge actually runs
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 19.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.85, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            bottom: Val::Px(16.0),
            max_width: Val::Px(340.0),
            ..default()
        },
        GlobalZIndex(13),
        LobbyToast,
        IntroRoot, // despawns with the screen
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.94)),
            IntroRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("JOHN KINGDOM - ARENA"),
                TextFont { font_size: 40.0, ..default() },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
            ));
            p.spawn((
                Text::new("build your LOADOUT - the shield always rides in its own slot (E raises it)\nfull controls: ESC menu > RULES & MANUAL"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(Color::srgb(0.7, 0.78, 0.95)),
            ));
            let prim: Vec<(&str, LoadoutButton)> = PRIMARIES
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(0, *g)))
                .collect();
            pick_row(p, "PRIMARY", &prim, 128.0);
            let sec: Vec<(&str, LoadoutButton)> = SECONDARIES
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(1, *g)))
                .collect();
            pick_row(p, "SECONDARY", &sec, 128.0);
            let spc: Vec<(&str, LoadoutButton)> = SPECIALS
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(2, *g)))
                .collect();
            pick_row(p, "SPECIAL", &spc, 128.0);
            // §6 (Brief IV): the melee slot is a CHOICE now
            let melee: Vec<(&str, MeleeButton)> = vec![
                ("Combat Knife", MeleeButton(false)),
                ("War Axe", MeleeButton(true)),
            ];
            pick_row(p, "MELEE", &melee, 128.0);
            // §8 (Brief IV): 6-point grenade budget presets
            let nades: Vec<(&str, NadeButton)> = GRENADE_PRESETS
                .iter()
                .enumerate()
                .map(|(i, (_, n))| (*n, NadeButton(i)))
                .collect();
            pick_row(p, "GRENADES", &nades, 128.0);
            let hats: Vec<(&str, CosmeticButton)> = HAT_CHOICES
                .iter()
                .enumerate()
                .map(|(i, (n, _))| (*n, CosmeticButton(0, i)))
                .collect();
            pick_row(p, "HAT", &hats, 90.0);
            let tunics: Vec<(&str, CosmeticButton)> = TUNIC_CHOICES
                .iter()
                .enumerate()
                .map(|(i, (n, _))| (*n, CosmeticButton(1, i)))
                .collect();
            pick_row(p, "OUTFIT", &tunics, 90.0);
            let maps: Vec<(&str, MapButton)> =
                MapKind::ALL
                    .iter()
                    // §12: Battlefield left the PvP rotation - it is the
                    // zombie-extraction adventure map now
                    .filter(|m| **m != MapKind::Battlefield)
                    .map(|m| (m.name(), MapButton(*m)))
                    .collect();
            pick_row(p, "BATTLEFIELD", &maps, 160.0);
            let diffs: Vec<(&str, DiffButton)> = Difficulty::ALL
                .iter()
                .map(|d| (d.name(), DiffButton(*d)))
                .collect();
            pick_row(p, "DIFFICULTY", &diffs, 100.0);
            let sizes: Vec<(&str, SizeButton)> =
                vec![("5 v 5", SizeButton(5)), ("8 v 8", SizeButton(8))];
            pick_row(p, "BATTLE SIZE", &sizes, 100.0);
            p.spawn((
                Text::new("\nPICK A MODE TO DEPLOY"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.4)),
            ));
            for (label, which) in [
                ("TEAM DEATHMATCH - first to 30", ModeButton::Tdm),
                ("KING OF THE HILL - hold the center 90 s", ModeButton::Koth),
                ("ZOMBIE EXTRACTION - survive, then hold the ring", ModeButton::Extraction),
            ] {
                p.spawn((
                    Button,
                    which,
                    // 620 wide / 18pt: the longest label ("ZOMBIE
                    // EXTRACTION - survive, then hold the ring") is 46
                    // chars, which WRAPPED inside the old 420x48 box and
                    // overlapped the row beneath it. Sized so every label
                    // stays on one line.
                    Node {
                        width: Val::Px(620.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

/// Shared by a real button click AND the capture harness's quick-deploy -
/// one code path for "start a match from the current `Selected` choices".
fn start_match(sel: &Selected, mode: Mode, game: &mut Game, next: &mut NextState<GameState>) {
    // §12: extraction ALWAYS runs on the Battlefield - the adventure map;
    // PvP always runs on the tight maps
    let map = if mode == Mode::Extraction {
        MapKind::Battlefield
    } else {
        sel.map
    };
    game.sim = TdmSim::new(MatchConfig {
        seed: 0x7EA9,
        per_team: sel.per_team,
        mode,
        map,
        difficulty: sel.difficulty,
        loadout: sel.loadout,
        melee_axe: sel.melee_axe,
        grenade_preset: sel.grenade_preset,
    });
    game.accum = 0.0;
    game.last_t = 0.0;
    game.rebuild = true;
    next.set(GameState::Playing);
}

fn intro_buttons(
    mut q: Query<
        (&Interaction, &ModeButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    sel: Res<Selected>,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<GameState>>,
) {
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.24, 0.29, 0.40)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
            Interaction::Pressed => {
                let mode = match which {
                    ModeButton::Tdm => Mode::Tdm,
                    ModeButton::Koth => Mode::Koth,
                    ModeButton::Extraction => Mode::Extraction,
                };
                start_match(&sel, mode, &mut game, &mut next);
            }
        }
    }
}

/// Shared select-highlight painter for all the pick-rows.
fn paint(bg: &mut BackgroundColor, selected: bool, hovered: bool) {
    *bg = BackgroundColor(if selected {
        Color::srgb(0.20, 0.45, 0.24)
    } else if hovered {
        Color::srgb(0.22, 0.30, 0.24)
    } else {
        Color::srgb(0.14, 0.17, 0.22)
    });
}

fn intro_map_buttons(
    mut q: Query<(&Interaction, &MapButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, mb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.map = mb.0;
        }
    }
    for (i, mb, mut bg) in &mut q {
        paint(&mut bg, sel.map == mb.0, *i == Interaction::Hovered);
    }
}

fn intro_loadout_buttons(
    mut q: Query<(&Interaction, &LoadoutButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, lb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.loadout[lb.0] = lb.1;
        }
    }
    for (i, lb, mut bg) in &mut q {
        paint(&mut bg, sel.loadout[lb.0] == lb.1, *i == Interaction::Hovered);
    }
}

fn intro_cosmetic_buttons(
    mut q: Query<(&Interaction, &CosmeticButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, cb, _) in &mut q {
        if *i == Interaction::Pressed {
            if cb.0 == 0 {
                sel.hat = cb.1;
            } else {
                sel.tunic = cb.1;
            }
        }
    }
    for (i, cb, mut bg) in &mut q {
        let selected = if cb.0 == 0 {
            sel.hat == cb.1
        } else {
            sel.tunic == cb.1
        };
        paint(&mut bg, selected, *i == Interaction::Hovered);
    }
}

fn intro_melee_buttons(
    mut q: Query<(&Interaction, &MeleeButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, mb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.melee_axe = mb.0;
        }
    }
    for (i, mb, mut bg) in &mut q {
        paint(&mut bg, sel.melee_axe == mb.0, *i == Interaction::Hovered);
    }
}

fn intro_nade_buttons(
    mut q: Query<(&Interaction, &NadeButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, nb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.grenade_preset = nb.0;
        }
    }
    for (i, nb, mut bg) in &mut q {
        paint(&mut bg, sel.grenade_preset == nb.0, *i == Interaction::Hovered);
    }
}

fn intro_diff_buttons(
    mut q: Query<(&Interaction, &DiffButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, db, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.difficulty = db.0;
        }
    }
    for (i, db, mut bg) in &mut q {
        paint(&mut bg, sel.difficulty == db.0, *i == Interaction::Hovered);
    }
}

fn intro_size_buttons(
    mut q: Query<(&Interaction, &SizeButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, sb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.per_team = sb.0;
        }
    }
    for (i, sb, mut bg) in &mut q {
        paint(&mut bg, sel.per_team == sb.0, *i == Interaction::Hovered);
    }
}

/// The ONLY place the cursor gets captured: entering actual play.
/// Menus and screens (Intro/Paused/Settings/Manual) all need a live,
/// visible cursor - grabbing on OnExit(Paused) soft-locked the loadout
/// and settings screens whenever they were entered from the pause menu.
fn grab_cursor(mut cam: ResMut<CamCtl>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    cam.grabbed = true;
    if let Ok(mut w) = windows.get_single_mut() {
        w.cursor_options.grab_mode = CursorGrabMode::Locked;
        w.cursor_options.visible = false;
    }
}

fn release_cursor(cam: &mut CamCtl, windows: &mut Query<&mut Window, With<PrimaryWindow>>) {
    cam.grabbed = false;
    if let Ok(mut w) = windows.get_single_mut() {
        w.cursor_options.grab_mode = CursorGrabMode::None;
        w.cursor_options.visible = true;
    }
}

fn close_intro(
    mut commands: Commands,
    intro: Query<Entity, With<IntroRoot>>,
    // `open_intro` spawns these two as TOP-LEVEL entities, not children
    // of IntroRoot, so despawning the root alone left them on screen for
    // the whole match - the loadout spec sat in the corner during
    // gameplay (visible in every committed mech capture).
    readout: Query<Entity, With<TechReadout>>,
    toast: Query<Entity, With<LobbyToast>>,
) {
    for e in intro.iter().chain(readout.iter()).chain(toast.iter()) {
        commands.entity(e).despawn_recursive();
    }
}

fn open_menu(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
    cam.ads = false; // no stale scope glass / zoom over the menu
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.75)),
            MenuRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
                Node {
                    margin: UiRect::bottom(Val::Px(18.0)),
                    ..default()
                },
            ));
            for (label, which) in [
                ("Resume        (ESC)", MenuButton::Resume),
                ("Restart Match", MenuButton::Restart),
                ("Change Mode / Loadout", MenuButton::BackToLoadout),
                ("Settings", MenuButton::Settings),
                ("Controls", MenuButton::Controls),
                ("Rules & Manual", MenuButton::Manual),
                ("Quit", MenuButton::Quit),
            ] {
                p.spawn((
                    Button,
                    which,
                    Node {
                        width: Val::Px(280.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

fn close_menu(mut commands: Commands, menu: Query<Entity, With<MenuRoot>>) {
    // no cursor grab here: Paused can exit to Intro/Settings/Manual,
    // which all need the mouse - OnEnter(Playing) does the grabbing
    for e in &menu {
        commands.entity(e).despawn_recursive();
    }
}

// ---- settings page (§14) -------------------------------------------------

fn open_settings(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.92)),
            SettingsRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("SETTINGS"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
            ));
            p.spawn((
                Text::new("Click a row to change it.  Settings apply immediately.\nmode / map / difficulty / team size / loadout:  ESC menu > Change Mode / Loadout"),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.78, 0.95)),
            ));
            for (which, kind, label) in [
                (
                    SettingsButton::Sens,
                    Some(SettingsButtonKind::Sens),
                    settings_label_text(SettingsButtonKind::Sens, &settings),
                ),
                (
                    SettingsButton::Fov,
                    Some(SettingsButtonKind::Fov),
                    settings_label_text(SettingsButtonKind::Fov, &settings),
                ),
                (
                    SettingsButton::InvertY,
                    Some(SettingsButtonKind::InvertY),
                    settings_label_text(SettingsButtonKind::InvertY, &settings),
                ),
                (
                    SettingsButton::SwapMouse,
                    Some(SettingsButtonKind::SwapMouse),
                    settings_label_text(SettingsButtonKind::SwapMouse, &settings),
                ),
                (
                    SettingsButton::Minimap,
                    Some(SettingsButtonKind::Minimap),
                    settings_label_text(SettingsButtonKind::Minimap, &settings),
                ),
                (SettingsButton::Back, None, "Back (ESC)".to_string()),
            ] {
                p.spawn((
                    Button,
                    which,
                    // 620 wide / 18pt: the mouse-swap row is 48 chars and
                    // WRAPPED to two lines inside the old 420x48 box,
                    // spilling past the button's own background.
                    Node {
                        width: Val::Px(620.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
                ))
                .with_children(|b| {
                    let mut e = b.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    if let Some(k) = kind {
                        e.insert(SettingsLabel(k));
                    }
                });
            }
        });
}

fn settings_label_text(kind: SettingsButtonKind, s: &GameSettings) -> String {
    match kind {
        SettingsButtonKind::SwapMouse => {
            // derived from the SAME mapping input_and_step uses - both
            // arms of the old hand-written version were backwards, so the
            // toggle promised the opposite of what it did
            let (aim, fire) = mouse_map_names(s.swap_mouse);
            let tag = if s.swap_mouse { "swapped" } else { "default" };
            format!("Mouse: {fire} fire / {aim} aim  ({tag})")
        }
        SettingsButtonKind::Minimap => {
            format!("Minimap: {}  (M in game)", if s.minimap { "ON" } else { "OFF" })
        }
        SettingsButtonKind::Sens => {
            format!("Mouse sensitivity: {}", SENS_CHOICES[s.sens_idx].0)
        }
        SettingsButtonKind::Fov => {
            format!("Field of view: {} deg", FOV_CHOICES[s.fov_idx].0)
        }
        SettingsButtonKind::InvertY => {
            format!("Invert look Y: {}", if s.invert_y { "ON" } else { "OFF" })
        }
    }
}

fn close_settings(mut commands: Commands, q: Query<Entity, With<SettingsRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn settings_buttons(
    mut q: Query<
        (&Interaction, &SettingsButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<GameSettings>,
    mut labels: Query<(&SettingsLabel, &mut Text)>,
    mut next: ResMut<NextState<GameState>>,
) {
    let mut dirty = false;
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.24, 0.29, 0.40)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
            Interaction::Pressed => match which {
                SettingsButton::SwapMouse => {
                    settings.swap_mouse = !settings.swap_mouse;
                    dirty = true;
                }
                SettingsButton::Minimap => {
                    settings.minimap = !settings.minimap;
                    dirty = true;
                }
                // click cycles forward through the choice list and wraps
                SettingsButton::Sens => {
                    settings.sens_idx = (settings.sens_idx + 1) % SENS_CHOICES.len();
                    dirty = true;
                }
                SettingsButton::Fov => {
                    settings.fov_idx = (settings.fov_idx + 1) % FOV_CHOICES.len();
                    dirty = true;
                }
                SettingsButton::InvertY => {
                    settings.invert_y = !settings.invert_y;
                    dirty = true;
                }
                SettingsButton::Back => next.set(GameState::Paused),
            },
        }
    }
    if dirty {
        for (l, mut t) in &mut labels {
            **t = settings_label_text(l.0, &settings);
        }
    }
}

// ---- rules & manual (§14): generated from the live weapon table ----------

fn open_manual(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
    // from the shared mapping - this block had it inverted, and also
    // still listed T as a fire key (T is INSPECT) and C as crouch (C is
    // the armor ability; a player following it fired the flamethrower
    // while trying to duck)
    let (aim_b, fire_b) = mouse_map_names(settings.swap_mouse);
    let mut manual = format!(
        "CONTROLS\n\
         WASD move - SPACE jump - Q dodge roll (hard landings roll automatically)\n\
         E shield up/down - O or V first/third person - Z / X lean - M minimap\n\
         {aim_b} aim (bow/spear draw the flight arc; AWM opens the scope)\n\
         {fire_b} fire - 1/2/3 weapon slots - CTRL crouch - C armor ability\n\
         T inspect weapon\n\
         SHIFT sprint (tap crouch at a sprint to roll) - R reload - TAB scores\n\n\
         THE SHIELD\n\
         Always carried. Blocks the FRONT ARC only (+/-60deg): standing cuts\n\
         damage 65%, crouched 95%. Sides and rear ignore it - FLANK.\n\
         Shield up = no shooting, slow walk.\n\n\
         DAMAGE MODEL\n\
         100 HP. Zones: head x4, torso x1, arms/legs x0.75.\n\
         Baseline M4A1: 2 headshots / 8 body shots.\n\
         AWM: head instant; torso, arms, legs = 2 shots.\n\n\
         CHECKPOINTS ('check back')\n\
         Stand in a white ring uncontested to flip it; your team then\n\
         respawns AT the ring. Contested rings freeze.\n\n\
         MODES\n\
         TDM first to 30 - KOTH hold the center 90 s - 5-min clock,\n\
         80 s sudden-death overtime.\n\n\
         WEAPONS (torso dmg / shots to kill body / head / mag)\n",
    );
    for g in ALL_WEAPONS {
        let s = gun(g);
        let body_stk = if s.projectile.is_some() {
            ((MAX_HEALTH / s.damage).ceil()) as u32
        } else {
            (MAX_HEALTH / (s.damage * s.pellets.max(1) as f32)).ceil() as u32
        };
        let head_stk = if s.projectile.is_some() {
            body_stk
        } else {
            (MAX_HEALTH / (s.damage * HEAD_MULT * s.pellets.max(1) as f32)).ceil() as u32
        };
        let class = match s.class {
            GunClass::Primary => "PRIMARY  ",
            GunClass::Secondary => "SECONDARY",
            GunClass::Special => "SPECIAL  ",
        };
        manual += &format!(
            "{:<14} {class} {:>5.1}{}  body x{}  head x{}{}  mag {}\n",
            s.name,
            s.damage,
            if s.pellets > 1 {
                format!(" x{} pellets", s.pellets)
            } else {
                String::new()
            },
            body_stk,
            head_stk,
            // honesty clause: pellet numbers assume the WHOLE spread lands
            if s.pellets > 1 { " (full spread)" } else { "" },
            s.mag
        );
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.94)),
            ManualRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("RULES & MANUAL          (ESC to go back)"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
            ));
            p.spawn((
                Text::new(manual),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn close_manual(mut commands: Commands, q: Query<Entity, With<ManualRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn menu_buttons(
    mut q: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.24, 0.29, 0.40)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
            Interaction::Pressed => match which {
                MenuButton::Resume => next.set(GameState::Playing),
                MenuButton::Restart => {
                    // same match again - mode, map, difficulty, size kept
                    game.sim = TdmSim::new(game.sim.cfg);
                    game.accum = 0.0;
                    game.last_t = 0.0;
                    game.rebuild = true;
                    next.set(GameState::Playing);
                }
                MenuButton::BackToLoadout => next.set(GameState::Intro),
                MenuButton::Settings => next.set(GameState::Settings),
                MenuButton::Manual => next.set(GameState::Manual),
                MenuButton::Controls => next.set(GameState::Controls),
                MenuButton::Quit => {
                    exit.send(AppExit::Success);
                }
            },
        }
    }
}

#[cfg(test)]
mod band_tests {
    use super::*;

    /// §1.4 (Brief VI) - the no-bounce gates, measured on the render
    /// path's OWN `carry_offset` (not a copy of its math).
    #[test]
    fn vm_never_bounces() {
        // standing still: ZERO positional motion at every bob phase
        for th in 0..100 {
            let o = carry_offset(0.0, th as f32 * 0.37, true, 0.0, 0.0, 0.0, 0.0);
            assert_eq!(o, Vec3::ZERO, "standing = frozen bob: {o:?}");
        }
        // ...and anywhere below the dead-zone
        let o = carry_offset(VM_BOB_DEADZONE * 0.9, 1.3, true, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(o, Vec3::ZERO, "sub-deadzone speed must not bob");
        // bounce meter: the whole fire-kick envelope at a standstill -
        // no lateral or vertical translation, rear slide ≤ 2 cm
        for ph in 0..=20 {
            let kick = ph as f32 / 20.0;
            let o = carry_offset(0.0, 0.0, true, kick, 0.0, 0.0, 0.0);
            assert!(
                o.x == 0.0 && o.y == 0.0,
                "firing adds ZERO lateral/vertical translation: {o:?}"
            );
            assert!(
                (0.0..=0.02).contains(&o.z),
                "back-slide stays ≤ 2 cm, rearward only: {}",
                o.z
            );
        }
        // after the envelope: exactly rest (≤ 2 mm demanded, 0 delivered)
        let rest = carry_offset(0.0, 0.0, true, 0.0, 0.0, 0.0, 0.0);
        assert!(rest.length() <= 0.002, "rest within 2 mm after the spray");
        // the kick return window is the ≤ 120 ms contract
        assert!(VM_KICK_RETURN_S <= 0.12 + 1e-6);
        // run-lower: full sprint pulls the weapon DOWN, never up
        for th in 0..50 {
            let o = carry_offset(1.0, th as f32 * 0.41, true, 0.0, 1.0, 0.0, 0.0);
            assert!(o.y < 0.0, "sprint must lower the weapon: {o:?}");
        }
        // airborne: bob is exactly ÷ 5
        let g = carry_offset(0.8, 1.1, true, 0.0, 0.0, 0.0, 0.0);
        let a = carry_offset(0.8, 1.1, false, 0.0, 0.0, 0.0, 0.0);
        assert!(
            (a.x / g.x - VM_AIR_BOB).abs() < 1e-5
                && (a.y / g.y - VM_AIR_BOB).abs() < 1e-5,
            "airborne bob must be exactly the CS:GO ÷5"
        );
    }

    /// §1.4a screen-intrusion: across stances × phases × fire × air, the
    /// root motion can never carry the weapon envelope (receiver box +
    /// sight mast) across the vertical midline nor into the central
    /// circle of radius 12% of screen height (vm FOV 68°).
    #[test]
    fn vm_envelope_clears_midline_and_center() {
        // circle radius in camera space at depth z: 12% of the full
        // screen height = 0.12 - 2-z-tan(fov/2)
        let r_at = |z: f32| 0.24 * z * (VM_FOV_DEG.to_radians() * 0.5).tan();
        for sf in [0.0_f32, 0.3, 0.6, 1.0] {
            for th in 0..80 {
                for (kick, sp) in [(0.0_f32, 0.0_f32), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
                    for grounded in [true, false] {
                        let o = carry_offset(
                            sf,
                            th as f32 * 0.173,
                            grounded,
                            kick,
                            sp,
                            0.04,
                            0.0,
                        );
                        let (x, y, z) = (0.11 + o.x, -0.13 + o.y, 0.32 + o.z);
                        let sway = z * VM_SWAY_CAP_DEG.to_radians().tan();
                        // midline: the widest part stays right of center
                        let recv_left = x - VM_RECEIVER_LEFT - sway;
                        let mast_left = x - VM_MAST_LEFT - sway;
                        assert!(
                            recv_left > 0.0,
                            "receiver crosses the midline: sf {sf} th {th}"
                        );
                        // center circle: nearest corner of each envelope
                        // part stays outside the 12% circle
                        let r = r_at(z);
                        for (dx, dy) in [
                            (recv_left, y + VM_RECEIVER_UP),
                            (mast_left, y + VM_MAST_UP),
                        ] {
                            let d = (dx * dx + dy * dy).sqrt();
                            assert!(
                                d > r,
                                "envelope corner inside the center circle: \
                                 d {d:.3} ≤ r {r:.3} (sf {sf} th {th})"
                            );
                        }
                    }
                }
            }
        }
    }

    /// §3.9 (Brief VI): the four-corner layout holds at 1920×1080,
    /// 2560×1440, and 1280×720 - every cluster inside the 5% safe area,
    /// anchored to its own quadrant, no two clusters colliding.
    #[test]
    fn hud_layout_holds_at_three_resolutions() {
        for (w, h) in [(1920.0_f32, 1080.0_f32), (2560.0, 1440.0), (1280.0, 720.0)] {
            let mut pts: Vec<(&str, f32, f32)> = Vec::new();
            for (name, anchor, off) in HUD_ANCHORS {
                let x = (anchor[0] + off[0]) * w;
                let y = (anchor[1] + off[1]) * h;
                // 5% safe area on both axes
                assert!(
                    x >= w * HUD_SAFE_FRAC - 1.0 && x <= w * (1.0 - HUD_SAFE_FRAC) + 1.0,
                    "{name} x={x} outside safe area at {w}x{h}"
                );
                assert!(
                    y >= h * HUD_SAFE_FRAC - 1.0 && y <= h * (1.0 - HUD_SAFE_FRAC) + 1.0,
                    "{name} y={y} outside safe area at {w}x{h}"
                );
                // the offset must pull INTO the screen from its anchor
                if anchor[0] == 0.0 {
                    assert!(off[0] > 0.0, "{name} must hang rightward");
                }
                if anchor[0] == 1.0 {
                    assert!(off[0] < 0.0, "{name} must hang leftward");
                }
                if anchor[1] == 1.0 {
                    assert!(off[1] < 0.0, "{name} must hang upward");
                }
                pts.push((name, x, y));
            }
            // no two clusters within 12% of screen width of each other
            for i in 0..pts.len() {
                for j in (i + 1)..pts.len() {
                    let dx = pts[i].1 - pts[j].1;
                    let dy = pts[i].2 - pts[j].2;
                    let d = (dx * dx + dy * dy).sqrt();
                    assert!(
                        d > w * 0.12,
                        "{} and {} collide at {w}x{h}: {d:.0}px apart",
                        pts[i].0,
                        pts[j].0
                    );
                }
            }
        }
    }

    /// §3.9: the semantic thresholds flip at EXACTLY the specified
    /// values, and the killfeed glyph mapping renders from a scripted
    /// event stream.
    #[test]
    fn hud_thresholds_and_glyphs() {
        // vitals: white above 25, red at ≤25, pulsing (alpha < 1) at ≤20
        assert_eq!(vitals_color(25.1, 0.0), Color::srgb(0.95, 0.96, 0.98));
        assert_eq!(vitals_color(25.0, 0.0), Color::srgb(1.0, 0.18, 0.15));
        let pulsing = vitals_color(20.0, 0.4);
        assert!(pulsing.alpha() < 1.0, "≤20 HP must PULSE");
        // ammo: red at exactly ≤25% of the magazine
        assert!(!ammo_is_low(8, 30)); // 26.7%
        assert!(ammo_is_low(7, 30)); // 23.3%
        assert!(ammo_is_low(5, 20)); // exactly 25%
        assert!(!ammo_is_low(0, 0)); // fists: never "low"
        // killfeed glyphs from a scripted stream
        let stream = [(true, " * "), (false, "  ")];
        for (hs, want) in stream {
            assert_eq!(feed_glyphs(hs), want);
        }
    }

    /// §1.4 Rule-2 gate: scoped + zoomed = the viewmodel is not rendered.
    #[test]
    fn vm_hides_while_scoped() {
        assert!(vm_hidden_while_scoped(true, true));
        assert!(!vm_hidden_while_scoped(true, false));
        assert!(!vm_hidden_while_scoped(false, true));
        assert!(!vm_hidden_while_scoped(false, false));
    }

    /// §5 (Brief IV): the interpenetration sweep - for every weapon in
    /// every firearm stance, the rear point (grip + stock) must land
    /// OUTSIDE the chest ellipse. Same static-guarantee machinery as the
    /// §1.3 gap test: the offsets are constants, so this holds per-frame.
    #[test]
    fn weapon_stock_clears_the_chest_in_every_stance() {
        // chest ellipse half-extents at weapon height (x, z)
        let (cx, cz) = (0.20_f32, 0.15_f32);
        for gun_k in ALL_WEAPONS {
            if matches!(gun_k, GunKind::Bow | GunKind::Spear) {
                continue; // carried on other mounts, cleared vertically
            }
            let rear = weapon_rear_extent(gun_k);
            if rear < 0.25 {
                continue; // no stock - a pistol grip NEAR the chest is
                          // a grip, not a clip
            }
            for z_root in [WR_Z_HIP, WR_Z_ADS] {
                let rz = z_root - rear;
                let inside = (WR_X / cx).powi(2) + (rz / cz).powi(2) < 1.0 - 0.05;
                assert!(
                    !inside,
                    "{gun_k:?}: stock at (x {WR_X}, z {rz:.3}) pierces the chest"
                );
            }
        }
    }

    /// §1.3 (Brief IV): connectivity - every parent–child pair overlaps
    /// its joint geometry by ≥5 mm. Bone lengths are rotation-invariant,
    /// so these static assertions hold at every phase of every clip:
    /// the overlay finds gaps, this test keeps them fixed.
    #[test]
    fn rig_joints_bridge_with_no_daylight() {
        let min = 0.005_f32;
        // NECK: sunk into the yoke below, past the head pivot above,
        // and still inside the head across the full ±48° pitch
        let yoke_top = 0.625 + 0.07;
        assert!(yoke_top - NECK_BOT >= 0.02, "neck must sink into the yoke");
        assert!(NECK_TOP - 0.846 >= 0.015, "neck must pierce the head base");
        assert!(
            NECK_TOP - 0.846 >= NECK_R * 0.75,
            "neck crossing survives full head pitch"
        );
        // YOKE reaches past the shoulder pivots
        assert!(YOKE_HALF_W - SHOULDER_X >= min, "yoke must reach the shoulders");
        // ELBOW: the upper shell reaches through the ball's span, and the
        // forearm starts inside it
        let upper_end = UPPER_CENTER - UPPER_HALF;
        assert!(
            upper_end <= ELBOW_Y + ELBOW_R - min,
            "upper shell reaches the elbow ball: end {upper_end}"
        );
        let fore_start = FORE_CENTER + FORE_HALF;
        assert!(
            fore_start >= -ELBOW_R + min,
            "forearm starts inside the elbow ball: start {fore_start}"
        );
        // WRIST: forearm reaches the wrist ball; the mitten overlaps it
        let fore_end = FORE_CENTER - FORE_HALF;
        assert!(
            fore_end <= WRIST_Y + WRIST_R - min,
            "forearm reaches the wrist ball: end {fore_end}"
        );
        let mitten_top = -0.02 + 0.05;
        assert!(mitten_top >= -WRIST_R + min, "mitten overlaps the wrist ball");
        // LEGS (spawn literals): hip ball ↔ pelvis, thigh ↔ knee ball,
        // shin ↔ ankle ball
        let pelvis_bottom = 0.09 - 0.08;
        assert!(0.055 - pelvis_bottom >= min, "hip ball meets the pelvis");
        let thigh_end = -0.145 - (0.15 / 2.0 + 0.072);
        assert!(thigh_end <= -0.29 + 0.065 - min, "thigh reaches the knee");
        let shin_end = -0.14 - (0.15 / 2.0 + 0.060);
        assert!(shin_end <= -0.28 + 0.045 - min, "shin reaches the ankle");
    }

    /// §0.2 (Brief II): the head's minimum world-Y across the FULL
    /// animation phase must stay at or above the 0.82 hit-band line for
    /// every grounded gait - idle, walk, run, sprint, strafe, backpedal,
    /// crouch-walk. A gait that dips the head below the line silently
    /// converts headshots into arm hits; this test makes that a failure,
    /// not a mystery. It samples the SAME pure pose function the renderer
    /// uses, so it cannot drift from what's on screen.
    #[test]
    fn head_never_leaves_its_band_in_any_gait() {
        for crouch in [false, true] {
            let band = HEAD_BAND_FRAC * if crouch { CROUCH_HEIGHT } else { BODY_HEIGHT };
            for amp in [0.0_f32, 0.2, 0.36, 0.6] {
                for lean in [-0.07_f32, 0.0, 0.07] {
                    // the post-roll weight-absorb dip is part of the pose
                    // and must be swept too - it moves the WHOLE rig down,
                    // and was previously applied outside gait_pose where
                    // this test could not see it at all
                    for settle in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
                        for k in 0..=128 {
                            let th = k as f32 / 128.0 * std::f32::consts::TAU;
                            let y = head_base_y(crouch, th, amp, lean, settle);
                            assert!(
                                y >= band - 1e-3,
                                "crouch={crouch} amp={amp} lean={lean} settle={settle} \
                                 theta={th:.2}: head base {y:.4} below band {band:.4}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// The settle dip must still DO something when there is headroom for
    /// it - a clamp that silently zeroed the whole effect would pass the
    /// band test above while deleting the feature.
    #[test]
    fn roll_settle_still_dips_when_the_band_allows_it() {
        let standing = gait_pose(false, 0.0, 0.0, 0.0, 0.0).0;
        let settled = gait_pose(false, 0.0, 0.0, 0.0, 1.0).0;
        assert!(
            settled < standing,
            "a standing fighter has band headroom, so the settle must visibly dip: \
             {standing} -> {settled}"
        );
    }
}

/// §1.4 (Brief VII) - the living-motion layer's completion gate. The
/// pure functions tested here are the SAME ones `sync_fighters` calls,
/// not copies - a passing statue test is a guarantee about the real
/// render path, not a parallel model of it.
#[cfg(test)]
mod living_motion_tests {
    use super::*;

    #[test]
    fn id_period_stays_in_range() {
        for id in 0..64u32 {
            let p = id_period(id, 6.0, 12.0);
            assert!((6.0..=12.0).contains(&p), "id {id}: period {p} out of [6,12]");
        }
    }

    #[test]
    fn breath_never_negative_and_bounded() {
        for i in 0..200 {
            let tnow = i as f32 * 0.15;
            for heat in [0.0_f32, 0.5, 1.0] {
                let b = breath_offset(tnow, 1.7, heat);
                assert!(b >= 0.0, "breath must never sink below rest: {b}");
                assert!(b <= 0.006, "breath amplitude budget exceeded: {b}");
            }
        }
    }

    #[test]
    fn breath_rate_ramps_with_heat() {
        // the same phase point, sampled while calm vs. just after a
        // sprint, must not be in lockstep - the rate genuinely changed
        let calm: Vec<f32> = (0..40).map(|i| breath_offset(i as f32 * 0.05, 0.0, 0.0)).collect();
        let hot: Vec<f32> = (0..40).map(|i| breath_offset(i as f32 * 0.05, 0.0, 1.0)).collect();
        assert_ne!(calm, hot, "breathing heat must change the rate, not just amplitude");
    }

    #[test]
    fn weight_shift_is_silenced_by_full_gait_amplitude() {
        for i in 0..50 {
            let tnow = i as f32 * 0.3;
            assert_eq!(
                weight_shift(tnow, 0.9, 8.0, 1.0),
                0.0,
                "full-speed gait must silence the idle weight shift"
            );
        }
    }

    #[test]
    fn weight_shift_moves_while_idle() {
        // over one full 6-12s period, idle weight shift must be nonzero
        // somewhere - a fighter standing still still weight-shifts
        let period = id_period(3, 6.0, 12.0);
        let any_nonzero = (0..200)
            .map(|i| i as f32 / 200.0 * period)
            .any(|t| weight_shift(t, 0.5, period, 0.0).abs() > 1e-4);
        assert!(any_nonzero, "an idle fighter must weight-shift at some point in its period");
    }

    #[test]
    fn grip_fidget_is_a_brief_blip_not_a_constant_twitch() {
        let period = 10.0;
        let samples: Vec<f32> = (0..2000)
            .map(|i| grip_fidget(i as f32 / 2000.0 * period * 3.0, 0.0, period))
            .collect();
        let nonzero = samples.iter().filter(|v| v.abs() > 1e-6).count();
        // the blip window (0.35s) out of a 10s period is ~3.5% duty
        let frac = nonzero as f32 / samples.len() as f32;
        assert!(frac < 0.15, "grip fidget must be a brief blip, not sustained: duty {frac:.3}");
        assert!(frac > 0.0, "grip fidget must actually fire at least once");
    }

    #[test]
    fn head_glance_never_exceeds_25_degrees() {
        for i in 0..500 {
            let t = i as f32 * 0.03;
            let g = head_glance(t, 0.6, 5.0 /* way past the clamp */);
            assert!(g.abs() <= 0.436 + 1e-4, "head glance exceeded +/-25deg: {g}");
        }
    }

    #[test]
    fn head_glance_is_a_glance_not_a_stare() {
        // a target held fixed for a full 20s should NOT keep the head
        // turned the whole time - the brief's "every ~4s" cadence
        let samples: Vec<f32> = (0..2000).map(|i| head_glance(i as f32 / 100.0, 0.0, 0.4)).collect();
        let looking = samples.iter().filter(|v| v.abs() > 0.05).count();
        let frac = looking as f32 / samples.len() as f32;
        assert!(frac < 0.5, "head must return to neutral between glances: on {frac:.2} of the time");
        assert!(frac > 0.05, "the glance must actually happen sometimes: {frac:.3}");
    }

    /// THE statue test (§1.4): 30 simulated seconds of a stationary,
    /// out-of-combat fighter. At no point may every live layer of the
    /// idle stack go quiet for more than 2 continuous seconds - some
    /// combination of breathing/weight-shift/grip-fidget/head-glance
    /// must always keep the body in motion.
    #[test]
    fn statue_test_idle_layer_never_holds_still_for_2s() {
        for id in [0u32, 1, 7, 13] {
            let ph = id as f32 * 2.399;
            let wperiod = id_period(id, 6.0, 12.0);
            let gperiod = id_period(id.wrapping_add(41), 8.0, 15.0);
            let dt = 0.05;
            let steps = (30.0 / dt) as usize;
            let mut still_for = 0.0_f32;
            let mut max_still = 0.0_f32;
            let mut prev = 0.0_f32;
            for i in 0..steps {
                let t = i as f32 * dt;
                let pose = breath_offset(t, ph, 0.0)
                    + weight_shift(t, ph, wperiod, 0.0)
                    + grip_fidget(t, ph, gperiod)
                    + head_glance(t, ph, 0.3);
                let delta = (pose - prev).abs();
                prev = pose;
                if delta < 1e-5 {
                    still_for += dt;
                    max_still = max_still.max(still_for);
                } else {
                    still_for = 0.0;
                }
            }
            assert!(
                max_still <= 2.0,
                "id {id}: idle layer held perfectly still for {max_still:.2}s (> 2s budget)"
            );
        }
    }
}
/// §2.7 (Brief VII v2) - the hand/arm craft pass's completion gate.
#[cfg(test)]
mod hand_craft_tests {
    use super::*;

    /// Joint-limit fuzz: 10,000 seeded random IK targets - the solved
    /// elbow flex must never exceed the biomechanical clamp, and never
    /// produce NaN (a degenerate target is a real risk this close to a
    /// two-bone solver's reach limit).
    #[test]
    fn elbow_flex_fuzz_never_exceeds_clamp_or_nans() {
        // a plain seeded LCG - this test doesn't need the sim's Pcg32
        // (main.rs has no reason to import it), just deterministic fuzz
        let mut seed = 0xE1B0_1234u32;
        let mut next01 = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (u32::MAX >> 8) as f32
        };
        let mut range = |lo: f32, hi: f32| lo + (hi - lo) * next01();
        for _ in 0..10_000 {
            let s = Vec3::new(range(-0.3, 0.3), range(1.3, 1.7), range(-0.1, 0.1));
            let t = Vec3::new(range(-0.5, 0.5), range(0.9, 1.8), range(-0.5, 0.5));
            let pole = Vec3::new(range(-1.0, 1.0), range(-1.0, 0.0), range(-1.0, 1.0));
            let (rot, flex) = solve_arm_ik(s, t, pole);
            assert!(!flex.is_nan(), "flex went NaN: s={s:?} t={t:?} pole={pole:?}");
            assert!(
                flex.to_degrees() >= ELBOW_FLEX_MIN_DEG - 0.01
                    && flex.to_degrees() <= ELBOW_FLEX_MAX_DEG + 0.01,
                "flex {:.1}deg outside clamp: s={s:?} t={t:?}",
                flex.to_degrees()
            );
            assert!(rot.is_finite(), "rotation went non-finite: s={s:?} t={t:?}");
        }
    }

    /// §2.2 coupling: the DIP-equivalent joint must track the driving
    /// joint at exactly the 0.7x ratio across the full curl range, and
    /// must never exceed it (a real fingertip cannot out-curl its own
    /// knuckle).
    #[test]
    fn dip_coupling_tracks_driving_joint_at_seven_tenths() {
        for i in 0..=20 {
            let curl = i as f32 / 20.0 * 2.0;
            let driving = -1.15 * curl;
            let dip = dip_from_driving_joint(driving);
            assert!((dip - driving * 0.7).abs() < 1e-5, "coupling ratio drifted at curl={curl}");
            assert!(
                dip.abs() <= driving.abs() + 1e-5,
                "DIP ({dip}) out-curled its driving joint ({driving}) at curl={curl}"
            );
        }
    }

    /// §2.5: the shared spring is critically damped - released from an
    /// offset with zero velocity, it must never overshoot the target
    /// (that's the entire definition of critical, vs. under- or over-
    /// damped), and it must actually converge.
    #[test]
    fn damped_spring_is_critical_never_overshoots() {
        for k in [45.0_f32, 60.0, 90.0, 120.0, 220.0] {
            let mut x = Vec2::new(1.0, 0.0);
            let mut v = Vec2::ZERO;
            let target = Vec2::ZERO;
            let dt = 1.0 / 240.0;
            let mut max_abs = 0.0_f32;
            for _ in 0..600 {
                let (nx, nv) = damped_spring(x, v, target, k, dt);
                x = nx;
                v = nv;
                max_abs = max_abs.max(x.x.abs());
            }
            assert!(max_abs <= 1.0 + 1e-3, "k={k}: spring overshot its 1.0 release point");
            assert!(x.length() < 0.01, "k={k}: spring failed to converge: {x:?}");
        }
    }

    #[test]
    fn damped_spring_agrees_at_60_and_240_fps() {
        // closed-form: the SAME wall-clock trajectory regardless of step
        // size - this is the entire reason it's closed-form, not Euler.
        let run = |dt: f32, steps: usize| {
            let mut x = Vec2::new(1.0, 0.0);
            let mut v = Vec2::ZERO;
            for _ in 0..steps {
                let (nx, nv) = damped_spring(x, v, Vec2::ZERO, 90.0, dt);
                x = nx;
                v = nv;
            }
            x
        };
        let at_60 = run(1.0 / 60.0, 30); // 0.5s of settle
        let at_240 = run(1.0 / 240.0, 120); // same 0.5s, 4x the steps
        assert!(
            (at_60 - at_240).length() < 0.01,
            "60fps ({at_60:?}) and 240fps ({at_240:?}) disagree"
        );
    }

    /// §2.4: the trigger finger is a brief travel-and-return, not a
    /// sustained press - it must be at rest well before the NEXT shot on
    /// every gun's fire cycle, even the fastest.
    #[test]
    fn trigger_finger_timing_matches_the_06_10_spec() {
        assert_eq!(trigger_finger_press(0.0), 0.0, "at rest the instant a shot resolves... ");
        assert!(trigger_finger_press(0.03) > 0.0 && trigger_finger_press(0.03) < 1.0);
        assert!(
            (trigger_finger_press(TRIGGER_OUT_S) - 1.0).abs() < 1e-4,
            "full travel exactly at 0.06s"
        );
        assert!(
            (trigger_finger_press(TRIGGER_OUT_S + TRIGGER_BACK_S)).abs() < 1e-4,
            "fully returned exactly at 0.06+0.10s"
        );
        assert_eq!(trigger_finger_press(10.0), 0.0, "long idle: at rest");
    }
}

/// §5.3 (Brief VII v2) - the camera rig's completion gate.
#[cfg(test)]
mod camera_v2_tests {
    use super::*;

    /// §B.1: `visor_ready` must distinguish "fully entered" from "never
    /// boarded" — the two cases `mech_enter_stage_for` collapses into
    /// the SAME `None`. The naive `matches!(stage, None | Some(HudBoot))`
    /// the plan warns about would put an infantryman inside a visor,
    /// because a plain foot soldier also reports `None`.
    #[test]
    fn visor_ready_tells_fully_entered_apart_from_never_boarded() {
        use sim::MechEnterStage::*;

        // A fighter who never boarded: None forever, never ready.
        let mut ready = false;
        for _ in 0..5 {
            ready = visor_ready_after(None, None, ready);
        }
        assert!(
            !ready,
            "a fighter who never boarded reports None and must NEVER be visor-ready \
             - this is the exact case the naive `matches!(None | HudBoot)` gets wrong"
        );

        // A full boarding sequence, stage by stage. The camera must stay
        // OUTSIDE until the very last stage.
        let mut ready = false;
        let mut prev = None;
        for (i, s) in sim::MECH_ENTER_STAGES.iter().enumerate() {
            ready = visor_ready_after(prev, Some(*s), ready);
            prev = Some(*s);
            if *s == HudBoot {
                assert!(ready, "HudBoot must make the visor available");
            } else {
                assert!(
                    !ready,
                    "stage {}/8 ({s:?}) made the camera cut to the visor early - \
                     boarding should still be shown from outside",
                    i + 1
                );
            }
        }

        // Once earned, it survives the transition ending (stage -> None,
        // i.e. boarding complete and the timer hit zero).
        let held = visor_ready_after(Some(HudBoot), None, true);
        assert!(held, "the visor must not switch off the instant boarding completes");

        // A FRESH boarding clears it again — re-entering starts outside.
        let recleared = visor_ready_after(None, Some(CockpitOpen), true);
        assert!(
            !recleared,
            "a new boarding must put the camera back outside, not leave it in the \
             visor of the mech being climbed into"
        );
    }

    /// §B.2: the idle-life terms must actually keep the hull alive while
    /// standing still, must stay SMALL enough to read as machinery
    /// rather than a wobble, and must not secretly be the stride cycle
    /// wearing a different name.
    #[test]
    fn mech_idle_life_never_goes_perfectly_inert() {
        // 1. Never dead. Sample a long window and require real motion in
        //    every term - the whole point is that a stopped mech is not
        //    a statue. (mech_bob is zero at a standstill by design; these
        //    are what fill that silence.)
        let mut tremor_x_max = 0.0_f32;
        let mut tremor_z_max = 0.0_f32;
        let mut breath_max = 0.0_f32;
        let mut t = 0.0_f32;
        while t < 30.0 {
            let (tx, tz) = mech_servo_tremor(t);
            tremor_x_max = tremor_x_max.max(tx.abs());
            tremor_z_max = tremor_z_max.max(tz.abs());
            breath_max = breath_max.max(mech_hull_breath(t).abs());
            t += 0.01;
        }
        assert!(tremor_x_max > 0.002, "pitch tremor never moves: {tremor_x_max}");
        assert!(tremor_z_max > 0.0015, "roll tremor never moves: {tremor_z_max}");
        assert!(breath_max > 0.007, "the hull never breathes: {breath_max}");

        // 2. Small enough to read as machinery. A tremor a player can
        //    consciously SEE is a camera bug, not idle life.
        assert!(
            tremor_x_max < 0.005 && tremor_z_max < 0.005,
            "tremor is visible as motion ({tremor_x_max}, {tremor_z_max} rad) - \
             it should sit at the edge of perception"
        );
        assert!(breath_max < 0.02, "hull breath {breath_max} m reads as a bounce");

        // 3. NOT the stride cycle in disguise. mech_bob runs at 0.9 Hz;
        //    if a tremor frequency were a small multiple of that it would
        //    beat against the walk and read as bob, not tremor. Checked
        //    as a real ratio rather than trusting the literals.
        for hz in [3.1 / std::f32::consts::TAU, 2.3 / std::f32::consts::TAU] {
            let ratio = hz / 0.9;
            let nearest_harmonic = ratio.round();
            assert!(
                (ratio - nearest_harmonic).abs() > 0.1,
                "a tremor at {hz:.3} Hz is {ratio:.2}x the 0.9 Hz stride - too \
                 close to a harmonic, it will read as the walk cycle"
            );
        }

        // 4. The two tremor axes must not move as one. Identical phase
        //    would read as a single diagonal rock rather than machinery.
        let (ax, az) = mech_servo_tremor(0.0);
        assert!(
            (ax - az).abs() > 1e-4 || ax == 0.0,
            "both tremor axes start identical - they will look like one axis"
        );
    }

    #[test]
    fn torso_aim_limit_matches_60_degrees() {
        assert_eq!(torso_aim_offset(30.0), 30.0, "within the clamp, passes through");
        assert_eq!(torso_aim_offset(-30.0), -30.0);
        assert_eq!(torso_aim_offset(90.0), 60.0, "clamped at +60deg");
        assert_eq!(torso_aim_offset(-90.0), -60.0, "clamped at -60deg");
        assert_eq!(torso_aim_offset(60.0), 60.0, "exactly at the boundary passes through");
    }

    #[test]
    fn camera_rig_offsets_match_brief_vii_v2_table() {
        assert_eq!(TP_BOOM, 2.2, "hip boom 2.2m");
        assert_eq!(TP_UP, 0.12, "hip up 0.12m");
        assert_eq!(TP_RIGHT, 0.45, "hip right 0.45m");
        assert_eq!(TP_BOOM_SPRINT, 2.5, "sprint boom eases to 2.5m");
        assert_eq!(TP_BOOM_AIM, 1.35, "aim boom 1.35m");
        assert_eq!(TP_RIGHT_AIM, 0.55, "aim right 0.55m");
    }

    /// §2.5's own claim ("this is the ONE spring primitive behind every
    /// secondary-motion element... camera boom k=90") was false when
    /// checked: `damped_spring` had exactly one real call site (the
    /// viewmodel sway, using its own k=196, not any of the five named
    /// constants) plus a test. This is the boom-recovery fix that makes
    /// the claim true for at least this one consumer.
    #[test]
    fn boom_recover_converges_without_overshoot() {
        let (mut b, mut v) = (1.0_f32, 0.0_f32);
        let allowed = 2.2_f32;
        for _ in 0..300 {
            let (nb, nv) = boom_recover(b, v, allowed, 1.0 / 120.0);
            assert!(
                nb <= allowed + 1e-3,
                "critically damped: must never overshoot the target ({nb} > {allowed})"
            );
            b = nb;
            v = nv;
        }
        assert!((b - allowed).abs() < 1e-3, "must converge to the allowed distance, got {b}");
    }

    #[test]
    fn boom_recover_moves_meaningfully_within_100ms_at_k90() {
        let (b, _) = boom_recover(1.0, 0.0, 2.2, 0.1);
        assert!(
            b > 1.2 && b < 2.2,
            "k=90 should move well past the start but not fully settle in 100ms, got {b}"
        );
    }

    /// Task 3 rule 5: the landing must actually REBOUND - the camera has
    /// to cross above neutral, not merely decay toward it more slowly.
    /// The previous shape (dip and rebound decaying independently, the
    /// rebound both smaller and faster) made this provably impossible.
    #[test]
    fn landing_rebound_actually_lifts_the_camera_past_neutral() {
        // a real 10 m/s impact, using the same scalings camera_system uses
        let dip_amp = ((10.0_f32 - 3.0) * 0.016).min(0.15);
        let reb_amp = landing_rebound_vy(-10.0) * 0.05;

        assert!(
            landing_offset(dip_amp, reb_amp, 0.0) > 0.0,
            "the frame of impact must be a DIP (positive = camera pushed down)"
        );

        let mut min_offset = f32::INFINITY;
        let mut t_min = 0.0_f32;
        for i in 0..600 {
            let t = i as f32 * 0.002;
            let o = landing_offset(dip_amp, reb_amp, t);
            if o < min_offset {
                min_offset = o;
                t_min = t;
            }
        }
        assert!(
            min_offset < -1e-4,
            "the rebound must carry the camera ABOVE neutral, best was {min_offset} at {t_min}s"
        );
        assert!(
            t_min > 0.05,
            "the rebound is a delayed counter-push, not simultaneous with the dip (peaked at {t_min}s)"
        );
        assert!(
            landing_offset(dip_amp, reb_amp, 1.0).abs() < 1e-3,
            "and it must settle back to neutral"
        );
    }

    /// The k=90 spring must govern ONLY collision recovery. Applying it
    /// to every boom increase also filtered the sprint ease, the ADS
    /// blend, and plain mouse-look - two filters in series, heavier one
    /// wins. These pin the three cases apart.
    #[test]
    fn boom_step_tracks_free_space_directly_and_springs_only_out_of_occlusion() {
        let dt = 1.0 / 120.0;

        // free space, no cover hit, target grows: must track it EXACTLY.
        // This is the case the old distance-only heuristic got wrong - it
        // looked identical to a collision recovery and got sprung.
        let (b, v, occ) = boom_step(2.20, 0.0, false, 2.50, 2.50, false, dt);
        assert!((b - 2.50).abs() < 1e-6, "free-space growth must track directly, got {b}");
        assert_eq!(v, 0.0, "no spring velocity should accumulate in free space");
        assert!(!occ, "nothing was hit, so nothing is occluded");

        // contact: pull in immediately
        let (b, v, occ) = boom_step(2.50, 0.0, false, 1.10, 2.50, true, dt);
        assert!((b - 1.10).abs() < 1e-6, "pull-in must be instant, got {b}");
        assert_eq!(v, 0.0);
        assert!(occ, "a ray hit means occluded");

        // cleared the corner (no hit this frame, but we WERE occluded):
        // spring back out rather than popping
        let (b, _, occ) = boom_step(1.10, 0.0, true, 2.50, 2.50, false, dt);
        assert!(
            b > 1.10 && b < 2.50,
            "recovery must be sprung, not instant: got {b}"
        );
        assert!(occ, "still recovering, so still flagged occluded");

        // ...and once recovered, the flag clears and it tracks again
        let (b, _, occ) = boom_step(2.50, 0.0, true, 2.50, 2.50, false, dt);
        assert!((b - 2.50).abs() < 1e-6);
        assert!(!occ, "fully recovered: back to free-space tracking");
    }

    #[test]
    fn boom_step_sprint_ease_is_not_re_filtered_by_the_spring() {
        // Simulate the documented sprint boom-out (2.2 -> 2.5 on the
        // 0.12s first-order lag) in FREE SPACE and confirm the boom
        // arrives on the ease's own schedule. Before the fix the spring
        // stretched this to roughly double.
        let dt = 1.0 / 120.0;
        let (mut eased, mut boom, mut vel) = (2.2_f32, 2.2_f32, 0.0_f32);
        let mut t = 0.0_f32;
        let mut t_90 = None;
        while t < 1.0 {
            eased += (2.5 - eased) * (dt / 0.12_f32).min(1.0);
            // no cover anywhere: allowed == free_len, hit == false
            let (nb, nv, _) = boom_step(boom, vel, false, eased, eased, false, dt);
            boom = nb;
            vel = nv;
            t += dt;
            if t_90.is_none() && boom >= 2.2 + 0.9 * (2.5 - 2.2) {
                t_90 = Some(t);
            }
        }
        let t90 = t_90.expect("boom must reach 90% of the sprint ease within 1s");
        assert!(
            t90 < 0.35,
            "sprint boom-out should follow the 0.12s ease (~0.25s to 90%), took {t90}s"
        );
    }
}

/// R4 - config externalization's completion gate (camera-tuning slice).
#[cfg(test)]
mod config_tuning_tests {
    use super::*;

    #[test]
    fn empty_or_missing_text_yields_exactly_the_compiled_in_defaults() {
        let t = parse_camera_tuning("");
        let d = CameraTuning::default();
        assert_eq!(t.tp_boom, d.tp_boom);
        assert_eq!(t.tp_up, d.tp_up);
        assert_eq!(t.tp_right, d.tp_right);
        assert_eq!(t.tp_boom_sprint, d.tp_boom_sprint);
        assert_eq!(t.tp_sprint_lag_s, d.tp_sprint_lag_s);
        assert_eq!(t.tp_boom_aim, d.tp_boom_aim);
        assert_eq!(t.tp_right_aim, d.tp_right_aim);
    }

    #[test]
    fn a_real_edit_overrides_exactly_that_one_key() {
        let t = parse_camera_tuning("tp_boom = 9.5\n");
        assert_eq!(t.tp_boom, 9.5, "the edited key must take effect");
        assert_eq!(t.tp_up, CameraTuning::default().tp_up, "untouched keys keep their default");
    }

    #[test]
    fn comments_blank_lines_and_whitespace_are_all_ignored() {
        let t = parse_camera_tuning(
            "\n  # a comment\n   tp_right   =   1.25   \n\n# tp_up = 99.0 (commented out)\n",
        );
        assert_eq!(t.tp_right, 1.25);
        assert_eq!(t.tp_up, CameraTuning::default().tp_up, "a commented-out line must not apply");
    }

    #[test]
    fn garbage_lines_and_unknown_keys_never_panic_and_never_apply() {
        let t = parse_camera_tuning("not a valid line at all\ntp_boom = not_a_number\nfake_key = 5.0\n");
        assert_eq!(t, CameraTuning::default(), "nothing here should have parsed");
    }
}

/// §7.4 (Brief VII v2) - the Forge's completion gate.
#[cfg(test)]
mod forge_tests {
    use super::*;

    #[test]
    fn profile_line_round_trips_every_field() {
        for p in [
            ForgeProfile { hat: 0, tunic: 0, melee_axe: false, grenade_preset: 0 },
            ForgeProfile { hat: 3, tunic: 2, melee_axe: true, grenade_preset: 3 },
            ForgeProfile { hat: 1, tunic: 3, melee_axe: false, grenade_preset: 2 },
        ] {
            let line = p.to_line();
            let back = ForgeProfile::from_line(&line).expect("must parse what it wrote");
            assert_eq!(p, back, "round-trip must be exact: {line}");
        }
    }

    #[test]
    fn from_line_rejects_garbage() {
        assert!(ForgeProfile::from_line("").is_none());
        assert!(ForgeProfile::from_line("not,a,valid,profile").is_none());
        assert!(ForgeProfile::from_line("1,2,3").is_none(), "too few fields");
    }

    #[test]
    fn save_then_load_round_trips_through_the_real_filesystem() {
        let slot = 99; // a slot no real save will ever use
        let p = ForgeProfile { hat: 2, tunic: 1, melee_axe: true, grenade_preset: 1 };
        forge_save(slot, &p).expect("save must succeed");
        let back = forge_load(slot).expect("load must find what was saved");
        assert_eq!(p, back);
        let _ = std::fs::remove_file(forge_slot_path(slot)); // clean up after itself
    }

    /// Task 3.3 sprint-start: the head arrives at a new lean LAST, one
    /// tip-onset behind the pelvis, and the transient vanishes at steady
    /// state - a held sprint must look exactly as before.
    #[test]
    fn the_head_trails_a_sprint_start_then_settles() {
        let dt = 1.0 / 120.0;
        let lean = 0.07_f32; // a hard sprint start's full lean
        let mut lag = 0.0_f32;
        // one tip-onset in, the head must still be visibly behind
        let onset_ticks = (CHAIN_ONSET_OFFSETS[7] / dt) as usize;
        for _ in 0..onset_ticks {
            lag = chain_lag_chase(lag, lean, dt);
        }
        let behind = lean - lag;
        assert!(
            behind > lean * 0.2,
            "one onset in, the head should still trail: {behind} of {lean}"
        );
        // ...and well before half a second, it has fully arrived
        for _ in 0..(0.5 / dt) as usize {
            lag = chain_lag_chase(lag, lean, dt);
        }
        assert!(
            (lean - lag).abs() < lean * 0.02,
            "at steady state the transient must be gone: {}",
            lean - lag
        );
        // the ripple is strictly monotic toward the target - no wobble
        let mut lag2 = 0.0_f32;
        let mut prev_gap = lean;
        for _ in 0..200 {
            lag2 = chain_lag_chase(lag2, lean, dt);
            let gap = lean - lag2;
            assert!(gap <= prev_gap + 1e-7, "the chase must never overshoot");
            prev_gap = gap;
        }
    }

    /// D7 (Thor, 2026-08-03): `the_head_trails_a_sprint_start_then_settles`
    /// above derives its tick count FROM `CHAIN_ONSET_OFFSETS[7]`, so it
    /// self-adjusts to whatever that constant says and passes for any
    /// value of it. Nothing in the suite pinned the one behaviour BRIEF
    /// VIII_B Step 1 actually changed: the head-lag time constant moved
    /// 0.125 -> 0.130 s.
    ///
    /// This pins it. Every number below is HAND-COMPUTED from the literal
    /// 0.130 - never read from the table - so if index 7 moves, this test
    /// fails and the change has to be deliberate.
    ///
    ///   alpha        = dt / 0.130 = (1/120) / 0.130 = 0.064102564
    ///   first tick   = lean * alpha
    ///   gap after n  = lean * (1 - alpha)^n
    ///
    /// FALSIFIABILITY: set `CHAIN_ONSET_OFFSETS[7]` back to 0.125 and the
    /// first tick becomes 0.00466667 (want 0.00448718, 1.8e-4 off = 180x
    /// the tolerance) and the 15-tick gap fraction becomes 0.35526440
    /// (want 0.37018930, 1.5e-2 off = 15000x). Delete the `.min(1.0)` and
    /// the clamp case below overshoots to 0.1077 rad against a 0.07 target.
    /// Measured f32-vs-f64 drift on the gap fractions is <= 8e-8, so the
    /// 1e-6 tolerance has >= 12x headroom; it is a regression pin on an
    /// exact arithmetic identity, NOT a claim of sub-millisecond accuracy
    /// (the source data is 50 fps - see the table's precision-ceiling note).
    #[test]
    fn head_lag_chase_pins_the_measured_tip_onset() {
        let dt = 1.0 / 120.0;
        let lean = 0.07_f32;

        // one tick from rest closes exactly alpha of the gap
        let first = chain_lag_chase(0.0, lean, dt);
        assert!(
            (first - 0.004_487_179_5).abs() < 1e-6,
            "one tick at 120 Hz must close dt/0.130 of the lean: want 0.0044871795, got {first}"
        );

        // and the gap decays geometrically at (1 - alpha) per tick
        let mut lag = 0.0_f32;
        for n in 1..=30 {
            lag = chain_lag_chase(lag, lean, dt);
            let gap_frac = (lean - lag) / lean;
            match n {
                // 15 ticks = 0.125 s, ~one time constant: (1-alpha)^15
                15 => assert!(
                    (gap_frac - 0.370_189_30).abs() < 1e-6,
                    "after 15 ticks the head must still hold 37.019% of the gap \
                     ((1 - (1/120)/0.130)^15); got {gap_frac}"
                ),
                // two time constants: the same number squared
                30 => assert!(
                    (gap_frac - 0.137_040_12).abs() < 1e-6,
                    "after 30 ticks: want 0.13704012, got {gap_frac}"
                ),
                _ => {}
            }
        }

        // a frame longer than the whole time constant must ARRIVE, not
        // overshoot - this is what `.min(1.0)` is for
        let big = chain_lag_chase(0.0, lean, 0.2);
        assert!(
            (big - lean).abs() < 1e-6,
            "a dt past the time constant must land ON the target, not sail through it: got {big}"
        );
    }

    /// §5.2: the turn-in-place. Brief VII v2 shipped `torso_aim_offset`
    /// built and tested with ZERO production call sites - the clamp
    /// existed but nothing ever separated the legs from the aim, so
    /// there was nothing to clamp. `step_leg_yaw` is the missing half.
    #[test]
    fn the_legs_lag_the_aim_and_the_torso_covers_the_difference() {
        let dt = 1.0 / 60.0;
        // first frame SNAPS - a fresh fighter must not spin up from 0
        let (leg, off) = step_leg_yaw(f32::NAN, 2.0, dt);
        assert_eq!(leg, 2.0, "uninitialised legs snap to the aim");
        assert_eq!(off, 0.0, "and need no torso compensation");

        // A small flick is covered by the TORSO immediately, while the
        // legs only creep. (This originally asserted the legs must not
        // move AT ALL within the clamp - which is what left a permanent
        // 60deg twist standing forever. The convergence check below is
        // what caught it, so the assertion was the thing that was wrong.)
        let small = 30.0_f32.to_radians();
        let (leg, off) = step_leg_yaw(0.0, small, dt);
        assert!(
            leg < small * 0.5,
            "one tick must not swing the legs most of the way, got {} deg",
            leg.to_degrees()
        );
        assert!(
            (off + leg - small).abs() < 1e-4,
            "torso + legs must together land exactly on the aim: {} + {} vs {}",
            off.to_degrees(),
            leg.to_degrees(),
            small.to_degrees()
        );

        // a big flick EXCEEDS the clamp, so the legs start catching up
        let big = 140.0_f32.to_radians();
        let (leg, off) = step_leg_yaw(0.0, big, dt);
        assert!(leg > 0.0, "past the clamp the legs must turn, got {leg}");
        assert!(
            off.to_degrees() <= TORSO_AIM_LIMIT_DEG + 1e-3,
            "the torso can never exceed its clamp, got {}",
            off.to_degrees()
        );

        // and they converge: hold the aim and the legs arrive, with the
        // torso returning to neutral
        let (mut leg, mut off) = (0.0_f32, 0.0_f32);
        for _ in 0..600 {
            let (l, o) = step_leg_yaw(leg, big, dt);
            leg = l;
            off = o;
        }
        assert!(
            (wrap_pi(big - leg)).abs() < 1e-3,
            "the legs must finish facing the aim, off by {} deg",
            wrap_pi(big - leg).to_degrees()
        );
        assert!(off.abs() < 1e-3, "and the torso unwinds to neutral");
    }

    /// Turning 350 deg left is really 10 deg right - without wrapping,
    /// crossing the yaw seam sends the legs the long way around.
    #[test]
    fn leg_turn_takes_the_short_way_around_the_seam() {
        let dt = 1.0 / 60.0;
        let from = 3.0_f32; // just under +PI
        let to = -3.0_f32; // just over -PI: 0.28 rad away the SHORT way
        let (leg, _) = step_leg_yaw(from, to, dt);
        // moving the short way means yaw INCREASES past PI (wrapping),
        // never a long sweep back down through zero
        let moved = wrap_pi(leg - from);
        assert!(
            moved > 0.0,
            "must cross the seam forwards, not sweep the long way: moved {moved}"
        );
        assert!(
            moved.abs() < 0.2,
            "and must not overshoot the 0.28 rad gap in one 60fps tick"
        );
    }

    /// The mouse mapping must come from ONE place. The settings label
    /// and the manual each derived it independently and BOTH had it
    /// backwards - on the very control that changes it.
    #[test]
    fn the_mouse_map_label_matches_the_actual_binding() {
        for swap in [false, true] {
            let (aim_btn, fire_btn) = mouse_map(swap);
            let (aim_name, fire_name) = mouse_map_names(swap);
            let name_of = |b: MouseButton| match b {
                MouseButton::Left => "LEFT CLICK",
                MouseButton::Right => "RIGHT CLICK",
                _ => "OTHER",
            };
            assert_eq!(
                name_of(aim_btn), aim_name,
                "swap={swap}: the aim NAME must match the aim BUTTON"
            );
            assert_eq!(
                name_of(fire_btn), fire_name,
                "swap={swap}: the fire NAME must match the fire BUTTON"
            );
            assert_ne!(aim_btn, fire_btn, "aim and fire cannot be the same button");
        }
        // the default is the conventional LEFT-fires mapping
        assert_eq!(mouse_map(false).1, MouseButton::Left, "default: LEFT fires");
        // and the settings row says so
        let s = GameSettings::default();
        let label = settings_label_text(SettingsButtonKind::SwapMouse, &s);
        assert!(
            label.contains("LEFT CLICK fire"),
            "the default label must advertise LEFT as fire, got {label:?}"
        );
    }

    /// Settings persistence: set -> serialize -> parse -> identical, and
    /// a hostile/stale file can never index out of bounds or crash. The
    /// audit table named "not persisted" as an honest gap; this is the
    /// gap closing WITH its round-trip proof, not just an fs::write.
    #[test]
    fn settings_round_trip_and_hostile_files_are_safe() {
        // round-trip every non-default value
        let mut s = GameSettings::default();
        s.swap_mouse = true;
        s.minimap = false;
        s.sens_idx = SENS_CHOICES.len() - 1;
        s.fov_idx = 0;
        s.invert_y = true;
        let back = parse_settings(&settings_to_text(&s));
        assert_eq!(back.swap_mouse, s.swap_mouse);
        assert_eq!(back.minimap, s.minimap);
        assert_eq!(back.sens_idx, s.sens_idx);
        assert_eq!(back.fov_idx, s.fov_idx);
        assert_eq!(back.invert_y, s.invert_y);

        // hostile: out-of-range indices clamp instead of panicking later
        let evil = "sens_idx = 999\nfov_idx = -5\nswap_mouse = 7\n";
        let p = parse_settings(evil);
        assert_eq!(p.sens_idx, SENS_CHOICES.len() - 1, "oversize index clamps to last");
        assert_eq!(p.fov_idx, 0, "negative index clamps to first");
        assert!(p.swap_mouse, "any nonzero reads as true");
        // and the clamped values actually index safely
        let _ = p.sens_mult();
        let _ = p.fov_deg();

        // garbage lines are ignored, defaults survive
        let junk = "!!!\nsens_idx = banana\n= 3\nfov_idx\n";
        let j = parse_settings(junk);
        assert_eq!(j.sens_idx, GameSettings::default().sens_idx);
        assert_eq!(j.fov_idx, GameSettings::default().fov_idx);
    }

    /// Settings must be real: every choice list has to be non-empty,
    /// ordered, and indexable by the default, or the settings screen
    /// panics or silently shows the wrong row.
    #[test]
    fn settings_choice_lists_are_valid_and_defaults_are_in_range() {
        assert!(SENS_DEFAULT_IDX < SENS_CHOICES.len());
        assert!(FOV_DEFAULT_IDX < FOV_CHOICES.len());
        let s = GameSettings::default();
        assert!((s.sens_mult() - 1.0).abs() < 1e-6, "default sensitivity is 1.00x");
        assert_eq!(s.fov_deg(), 90.0, "default FOV is the recommended 90");
        // strictly increasing, so cycling reads as a real scale
        for w in SENS_CHOICES.windows(2) {
            assert!(w[1].1 > w[0].1, "sensitivity choices must ascend");
        }
        for w in FOV_CHOICES.windows(2) {
            assert!(w[1].1 > w[0].1, "FOV choices must ascend");
        }
        // and every index is reachable by cycling, landing back at the start
        let mut idx = 0usize;
        for _ in 0..SENS_CHOICES.len() {
            idx = (idx + 1) % SENS_CHOICES.len();
        }
        assert_eq!(idx, 0, "cycling must wrap exactly");
    }

    /// A settings value that cannot be read back is a dead control.
    #[test]
    fn every_settings_row_renders_a_distinct_live_label() {
        let mut s = GameSettings::default();
        let kinds = [
            SettingsButtonKind::Sens,
            SettingsButtonKind::Fov,
            SettingsButtonKind::InvertY,
            SettingsButtonKind::SwapMouse,
            SettingsButtonKind::Minimap,
        ];
        for k in kinds {
            assert!(!settings_label_text(k, &s).is_empty());
        }
        // and each row's label actually CHANGES when its value does
        let before = settings_label_text(SettingsButtonKind::Sens, &s);
        s.sens_idx = (s.sens_idx + 1) % SENS_CHOICES.len();
        assert_ne!(before, settings_label_text(SettingsButtonKind::Sens, &s));
        let before = settings_label_text(SettingsButtonKind::Fov, &s);
        s.fov_idx = (s.fov_idx + 1) % FOV_CHOICES.len();
        assert_ne!(before, settings_label_text(SettingsButtonKind::Fov, &s));
        let before = settings_label_text(SettingsButtonKind::InvertY, &s);
        s.invert_y = !s.invert_y;
        assert_ne!(before, settings_label_text(SettingsButtonKind::InvertY, &s));
    }

    #[test]
    fn apply_to_selected_only_touches_the_forges_own_fields() {
        let mut sel = Selected::default();
        sel.map = MapKind::Bailey; // untouched by the Forge - must survive
        let p = ForgeProfile { hat: 3, tunic: 0, melee_axe: true, grenade_preset: 3 };
        p.apply_to(&mut sel);
        assert_eq!(sel.hat, 3);
        assert_eq!(sel.tunic, 0);
        assert!(sel.melee_axe);
        assert_eq!(sel.grenade_preset, 3);
        assert_eq!(sel.map, MapKind::Bailey, "the Forge must not touch match config");
    }
}

/// §2 (MISSION doc rig audit) - the separation test. The render root
/// (parent of legs) carries exactly `f.yaw`; torso is the root's CHILD
/// with `torso_coil_yaw(..)` as its OWN additional local Y-rotation - so
/// this function's return value literally IS "thorax yaw minus pelvis
/// yaw" (composition of a child's local rotation onto its parent), not
/// an estimate of it. This test is the direct rebuttal-or-confirmation
/// of the document's premise that a single trunk bone forces this to 0.
#[cfg(test)]
mod rig_separation_tests {
    use super::*;

    #[test]
    fn hip_shoulder_separation_reaches_35_to_45_degrees_at_windup() {
        // sweep the windup progress (spear_wind_t counting DOWN from
        // SPEAR_WINDUP_S to 0) and track the peak |separation|
        let mut peak_deg = 0.0_f32;
        for i in 0..=100 {
            let wind_t = SPEAR_WINDUP_S * (1.0 - i as f32 / 100.0);
            let sep = torso_coil_yaw(GunKind::Spear, wind_t, 0.0, false, 0.0);
            peak_deg = peak_deg.max(sep.abs().to_degrees());
        }
        assert!(
            (35.0..=45.0).contains(&peak_deg),
            "separation peak {peak_deg:.1}deg outside the 35-45deg target"
        );
    }

    #[test]
    fn separation_is_genuinely_nonzero_not_a_fused_bone() {
        // the document's exact failure mode to rule out: if root and
        // torso were the SAME rotation (a fused single trunk bone), this
        // would be identically 0.0 at every sample - it is not.
        let mid_wind = SPEAR_WINDUP_S * 0.5;
        let sep = torso_coil_yaw(GunKind::Spear, mid_wind, 0.0, false, 0.0);
        assert_ne!(sep, 0.0, "torso and root must NOT share one fused rotation");
    }

    #[test]
    fn no_gun_no_twist() {
        // separation is a THROW-specific coil, not a permanent offset -
        // resting state must be neutral. The last parameter is the
        // follow-through clock; NEGATIVE is "nothing thrown yet", which
        // is the actual resting state this test means. (It used to pass
        // 0.0 and still read as rest only because the old curve was
        // silent at t=0 - which was itself the bug: a real release
        // starts at the release yaw, not at neutral.)
        assert_eq!(torso_coil_yaw(GunKind::M4, 0.0, 0.0, false, -1.0), 0.0);
        assert_eq!(torso_coil_yaw(GunKind::Spear, 0.0, 0.0, false, -1.0), 0.0);
        // a gun that is not a spear never twists, whatever the clock says
        assert_eq!(torso_coil_yaw(GunKind::M4, 0.0, 0.0, false, 0.0), 0.0);
        // and long after a throw, the follow-through has fully settled
        assert!(torso_coil_yaw(GunKind::Spear, 0.0, 0.0, false, 3.0).abs() < 1e-3);
    }
}

/// Task 3 (MISSION doc) - the elastic load model's completion gate.
#[cfg(test)]
mod elastic_load_tests {
    use super::*;

    #[test]
    fn load_release_ratio_matches_spec_examples() {
        // the spec's own worked example: 0.4s wind-up -> 0.15-0.20s release
        let spear_throw = ElasticMove {
            load_s: 0.4,
            release_s: 0.18,
            stored_energy: 1.0,
            return_efficiency: 0.92,
        };
        assert!(spear_throw.load_release_ok(), "0.4s/0.18s must satisfy the 2x+ rule");
        let too_slow = ElasticMove {
            load_s: 0.4,
            release_s: 0.25,
            stored_energy: 1.0,
            return_efficiency: 0.92,
        };
        assert!(!too_slow.load_release_ok(), "0.25s release from a 0.4s load is a SHOVE, not a strike");
    }

    #[test]
    fn stored_energy_scales_output_by_exactly_the_spec_formula() {
        let base = 22.0; // e.g. the spear's throw v0
        let dead = ElasticMove { load_s: 0.4, release_s: 0.18, stored_energy: 0.0, return_efficiency: 0.92 };
        let full = ElasticMove { load_s: 0.4, release_s: 0.18, stored_energy: 1.0, return_efficiency: 0.92 };
        assert_eq!(dead.release_velocity(base), base, "zero stored energy = base, unscaled");
        assert!(
            (full.release_velocity(base) - base * 1.35).abs() < 1e-4,
            "full stored energy must be exactly base × 1.35"
        );
    }

    #[test]
    fn counter_movement_grants_the_bonus_a_dead_start_does_not() {
        // moving down then releasing up = counter-movement = bonus
        assert_eq!(counter_movement_bonus(-1.0, 1.0, 0.35), 0.35);
        // moving up then releasing up = no counter-movement = no bonus
        assert_eq!(counter_movement_bonus(1.0, 1.0, 0.35), 0.0);
        // starting from rest = no counter-movement = no bonus
        assert_eq!(counter_movement_bonus(0.0, 1.0, 0.35), 0.0);
    }

    #[test]
    fn landing_rebound_never_reaches_exactly_zero_for_a_real_impact() {
        for impact in [-3.0_f32, -6.0, -9.5, -15.0] {
            let reb = landing_rebound_vy(impact);
            assert!(reb > 0.0, "a real impact ({impact} m/s) must return SOME upward rebound");
            assert!((reb - (-impact) * 0.08).abs() < 1e-4, "must be exactly 8% of impact speed");
        }
        assert_eq!(landing_rebound_vy(0.0), 0.0, "no impact = no rebound, not a negative dip");
    }

    /// Task 3.4 chain-timing test: angular-velocity peaks occur in strict
    /// order pelvis -> lumbar -> thorax -> shoulder -> elbow -> wrist ->
    /// tip, each with a minimum onset gap. A failure names which segment
    /// fired early (out of order relative to the one before it).
    #[test]
    fn kinetic_chain_peaks_fire_in_strict_proximal_to_distal_order() {
        let ramp_s = 0.05;
        let names = ["pelvis", "lumbar", "thorax", "clavicle", "upper_arm", "forearm", "hand", "tip"];
        let mut prev_peak_tick = -1.0_f32;
        for i in 0..8 {
            let peak_tick = chain_peak_tick(i, ramp_s);
            assert!(
                peak_tick > prev_peak_tick,
                "{} peaked at {peak_tick:.3}s, not after the previous segment ({prev_peak_tick:.3}s) - fired early",
                names[i]
            );
            prev_peak_tick = peak_tick;
        }
    }

    /// §3 (BRIEF_VIII_B): the chain table must still hit the MEASURED
    /// javelin anchors exactly. `JAVELIN_ANCHOR_S` is held as its own
    /// table so this compares two independent things rather than a
    /// constant against itself - if someone retunes the interpolated
    /// indices (1,2,5,6) that is a judgement call, but silently moving
    /// a measured anchor is a factual error and fails here.
    #[test]
    fn the_kinetic_chain_still_hits_every_measured_javelin_anchor() {
        for (idx, want) in JAVELIN_ANCHOR_S {
            let got = CHAIN_ONSET_OFFSETS[idx];
            assert!(
                (got - want).abs() < 1e-6,
                "index {idx} is a MEASURED anchor (Campos 2004 Table 3): \
                 expected {want}s, table says {got}s"
            );
        }
        // the interpolated indices must stay strictly inside their
        // bracketing anchors - an interpolation that escapes its own
        // window is not an interpolation
        assert!(
            CHAIN_ONSET_OFFSETS[1] > CHAIN_ONSET_OFFSETS[0]
                && CHAIN_ONSET_OFFSETS[2] > CHAIN_ONSET_OFFSETS[1]
                && CHAIN_ONSET_OFFSETS[2] < CHAIN_ONSET_OFFSETS[3],
            "trunk interpolation escaped the pelvis..clavicle window"
        );
        assert!(
            CHAIN_ONSET_OFFSETS[5] > CHAIN_ONSET_OFFSETS[4]
                && CHAIN_ONSET_OFFSETS[6] > CHAIN_ONSET_OFFSETS[5]
                && CHAIN_ONSET_OFFSETS[6] < CHAIN_ONSET_OFFSETS[7],
            "arm interpolation escaped the upper-arm..tip window"
        );
        // D5 (Thor, 2026-08-03): what stood here compared
        // OFFSETS[3]-OFFSETS[0] against OFFSETS[4]-OFFSETS[3] and called
        // it "distal compression". It was UNFALSIFIABLE - all three
        // indices are measured anchors already pinned to 1e-6 by the loop
        // twenty lines above, so no edit could ever fail it - and its
        // comment was backwards: 0->3 spans THREE hops and 3->4 spans
        // ONE, so per hop that is 13.3 ms then 30.0 ms. Across that
        // boundary the chain EXPANDS, not compresses, because the 5 ms
        // clavicle floor squeezes the trunk hops. Real distal compression
        // is a claim about the ARM window, and it is the only place where
        // an INTERPOLATED index (5, 6) can make it fail.
        //
        // F3 (Thor, 2026-08-03): the failure message used to end "indices
        // 5 and 6 are the interpolated ones", which reads as a claim that
        // this assertion CONSTRAINS them. It constrains them, but only
        // coarsely, and the bounds are cheap to write down. Holding the
        // other seven rows fixed and stepping one index a millisecond at
        // a time, per-hop monotonicity survives across:
        //
        //     idx5 in [0.093 .. 0.098]   (shipped 0.094: -1 / +4 ms slack)
        //     idx6 in [0.112 .. 0.117]   (shipped 0.114: -2 / +3 ms slack)
        //
        // So it fires on idx5 moved DOWN >=2 ms or UP >=5 ms, and on idx6
        // moved DOWN >=3 ms or UP >=4 ms - and it does NOT fire on a 1 ms
        // move of either. Thor measured exactly that: idx6 0.114 -> 0.115
        // PASSES here. The 1 ms guard is
        // `the_arm_onsets_reproduce_an_independently_solved_geometric_root`,
        // which rejects 0.115 by 9.75e-4 against its 5e-4 gate.
        //
        // MESSAGE NARROWED, ASSERTION LEFT ALONE - deliberately. The only
        // way to broaden this check to 1 ms resolution is to re-derive the
        // geometric ratio right here, and that is the other test's entire
        // job. A second copy of it would be redundancy wearing the costume
        // of coverage, which is the same error class in a new place. What
        // was false was the CLAIM, so the claim is what changed.
        let hop = |a: usize, b: usize| CHAIN_ONSET_OFFSETS[b] - CHAIN_ONSET_OFFSETS[a];
        let arm_hops = [hop(3, 4), hop(4, 5), hop(5, 6), hop(6, 7)];
        for w in arm_hops.windows(2) {
            assert!(
                w[1] < w[0],
                "the ARM window must compress hop by hop, got {arm_hops:?}. \
                 This is a SHAPE check on the interpolated indices 5 and 6, \
                 not a millisecond one: it only fires outside \
                 idx5 in [0.093, 0.098] / idx6 in [0.112, 0.117]. A 1 ms \
                 move of either passes HERE and is caught by \
                 the_arm_onsets_reproduce_an_independently_solved_geometric_root"
            );
        }
        // The honest version of the trunk-vs-arm statement, recorded as a
        // FACT and deliberately NOT asserted: 0->3 is 40 ms over three
        // hops (13.3 ms each), 3->4 is 30 ms over one, so leaving the
        // trunk the chain steps UP. It is not asserted because indices
        // 0, 3 and 4 are all measured anchors already pinned to 1e-6 by
        // the loop at the top of this test - an assertion over them
        // cannot fail, and an assertion that cannot fail is worse than a
        // comment, because it looks like coverage. That is what was here.
    }

    /// D3 (Thor, 2026-08-03): the spec's §3.3 arm derivation printed
    /// `q = 0.8107` for `q + q^2 + q^3 = 2`. That is wrong - the root is
    /// 0.81053571; 0.8107 sums to 2.0007545, so the three arm hops come to
    /// 60.023 ms and the tip lands at 130.023 ms, not "exactly" 130. The
    /// SHIPPED TABLE IS UNAFFECTED (the true root still rounds to
    /// 0.094/0.114/0.130), which is why no constant moved - but the spec
    /// said "exactly" while its own printed sum said 60.03, and nothing in
    /// the suite would have noticed either way.
    ///
    /// So: solve the root HERE, by bisection, from the MEASURED anchors
    /// only. Nothing in this test reads the spec's q, and nothing reads
    /// indices 5 or 6 except to check them. That makes it an independent
    /// source of truth for the two interpolated arm indices, which the
    /// anchor loop above cannot touch.
    #[test]
    fn the_arm_onsets_reproduce_an_independently_solved_geometric_root() {
        let anchor = |i: usize| {
            JAVELIN_ANCHOR_S.iter().find(|(k, _)| *k == i).expect("measured anchor").1 as f64
        };
        // seed and span come from the measurement, not from the table
        let base = anchor(4) - anchor(3); // 30 ms, the last MEASURED gap
        let span = anchor(7) - anchor(4); // 60 ms of arm left to fill
        let target = span / base; // == 2.0
        // solve base*(q + q^2 + q^3) == span for q
        let (mut lo, mut hi) = (0.0_f64, 1.5_f64);
        for _ in 0..200 {
            let m = 0.5 * (lo + hi);
            if m + m * m + m * m * m < target {
                lo = m;
            } else {
                hi = m;
            }
        }
        let q = 0.5 * (lo + hi);
        // The exact root of q + q^2 + q^3 = 2 is 0.8105357138. This
        // bisection cannot reach it: it solves for `target`, and `target`
        // is built from f32 consts - `0.130f32` is really 0.129999995231,
        // so `target` is 1.9999998 rather than 2, which walks the root
        // back by ~5e-8. That is a fact about reading the anchors instead
        // of hardcoding them, and reading them is the entire point. 1e-6
        // absorbs it and is still 164x tighter than the spec's 0.8107.
        assert!(
            (q - 0.810_535_713_8).abs() < 1e-6,
            "the geometric root is 0.8105357138, not {q} (the spec said 0.8107, \
             which sums to 2.0007545 and puts the tip at 130.023 ms)"
        );
        // the shipped table is these hops accumulated and rounded to the
        // nearest MILLISECOND - so 5e-4 is the exact rounding claim, not a
        // slack tolerance. Index 5's margin is the tight one: it lands
        // 3.16e-4 from 0.094, i.e. 1.6x inside the half-millisecond.
        let want = [anchor(4) + base * q, anchor(4) + base * (q + q * q), anchor(7)];
        for (i, w) in [(5usize, want[0]), (6, want[1]), (7, want[2])] {
            let got = CHAIN_ONSET_OFFSETS[i] as f64;
            assert!(
                (got - w).abs() < 5e-4,
                "index {i}: the geometric compression puts it at {w:.7}s, which \
                 rounds to {:.3}s; the table says {got}s",
                (w * 1000.0).round() / 1000.0
            );
        }
        // this test CANNOT tell 0.8107 from 0.81053571 at the table's 1 ms
        // resolution - both round to the same three values. That is D3's
        // point, and the reason no constant changed. What it CAN catch is
        // any 1 ms move of index 5 or 6.
    }

    /// D6 (Thor, 2026-08-03) - the worst of the seven. What stood here
    /// NEVER CALLED `spear_followthrough_yaw`. It retyped that function's
    /// internal drive expression and then asserted the retyped copy
    /// equalled the algebra, so it was guarding a LEMMA under the name of
    /// the THEOREM: delete the `+ onset` from the real function and this
    /// test stayed GREEN. That is not hypothetical - it is the exact bug
    /// already shipped once in this file (handback/AUDIT.md, "bugs I
    /// introduced this session" #1: the follow-through went silent for a
    /// whole tip-onset and then swung the wrong way).
    ///
    /// SCOPE OF THAT CLAIM, stated precisely because the loose version of
    /// it travelled (Thor, 2026-08-03): the sentence above is about THIS
    /// test, not about the suite. On the pre-change tree that mutation
    /// gave **144 passed, 1 failed** - `spear_followthrough_carries_past_
    /// the_release_then_settles` did catch it. The rewrite took detection
    /// from ONE test to THREE and moved it onto the function's own
    /// contract; it did not rescue the bug from zero coverage, and nobody
    /// should repeat it as though it had.
    ///
    /// Now it calls the real function. `spear_followthrough_yaw_from`
    /// takes the tip's two table rows as parameters, so the test can feed
    /// rows the consts do not contain - which is what makes "invariant to
    /// the tables" a statement the code can actually violate.
    ///
    /// FALSIFIABILITY: drop `+ tip_onset` and the (0.0, 1.0) variant
    /// diverges from the (0.500, 5.222) variant by ~0.39 rad at small
    /// `release_t`. Drop `/ tip_peak` and the peak variants diverge by
    /// ~0.3 rad. Both are ~5 orders over the tolerance.
    ///
    /// NOT BIT-IDENTICAL. The spec's Step 1 test table specified `==`;
    /// that is false in f32, because `(t + onset) - onset` and
    /// `peak * x / peak` each round. Measured worst divergence across
    /// these six variants over 0..0.6 s is 2.98e-8 rad - real, tiny, and
    /// **34x** inside the tolerance below (1e-6 / 2.98e-8 = 33.6). That
    /// is the margin for the 1 ms grid THIS test sweeps; refine the grid
    /// and it drops - 5.96e-8, 17x, on a 1 us grid. Before touching the
    /// tolerance or the step, read the UNITS paragraph in
    /// `spear_followthrough_yaw`'s doc block: the "5.6x" that used to be
    /// quoted for this same quantity was a drive-term residual divided by
    /// a yaw tolerance, and it is gone.
    ///
    /// Invariance alone is vacuous (a function returning 0.0 is invariant
    /// to everything). `spear_followthrough_matches_its_hand_computed_curve`
    /// is the other half: it pins the curve itself to numbers derived
    /// outside this file.
    #[test]
    fn spear_followthrough_is_invariant_to_the_chain_tables() {
        let variants: [(f32, f32); 6] = [
            (CHAIN_ONSET_OFFSETS[7], CHAIN_PEAK_SCALE[7]), // shipped
            (0.125, CHAIN_PEAK_SCALE[7]),                  // the pre-BRIEF_VIII_B onset
            (0.0, 1.0),                                    // no chain offset at all
            (0.500, 5.222),                                // ~4x onset, 2x peak
            (0.001, 0.25),                                 // a peak BELOW 1.0
            (0.250, 100.0),                                // absurd peak
        ];
        for step in 0..=600 {
            let release_t = step as f32 * 0.001;
            let base = spear_followthrough_yaw_from(release_t, variants[0].0, variants[0].1);
            // the shipped wrapper must BE the shipped-table variant, bit
            // for bit - same inputs, same arithmetic, no excuse to differ
            assert_eq!(
                spear_followthrough_yaw(release_t).to_bits(),
                base.to_bits(),
                "at release_t={release_t}: the public wrapper is not the \
                 parameterised function at the shipped table rows"
            );
            for (onset, peak) in &variants[1..] {
                let got = spear_followthrough_yaw_from(release_t, *onset, *peak);
                assert!(
                    (got - base).abs() < 1e-6,
                    "at release_t={release_t} with (onset {onset}, peak {peak}): \
                     the tables did NOT cancel (shipped {base}, substituted {got}) \
                     - the zero-risk argument for retuning the chain no longer holds"
                );
            }
        }
    }

    /// The independent half of D6's fix, and the reason the invariance
    /// test above is not vacuous.
    ///
    /// `RAMP_S`, `OVERSHOOT_RAD`, `HOLD_S` and `SETTLE_RATE` are function-
    /// local consts, so this test CANNOT reference them. It therefore has
    /// to carry the spec in some form, and the form matters enormously.
    ///
    /// **F1 (Thor, 2026-08-03) - THE HAZARD THIS STRUCTURE EXISTS TO
    /// KILL.** What stood here was seven literal output values with a
    /// comment saying they had been computed outside the crate. Nothing
    /// ENFORCED that. The cheapest move available to the next maintainer
    /// who breaks this curve is to run the code, paste its output over the
    /// seven literals, and watch the test go green - at which point the
    /// test has silently become a change-detector that pins the bug in
    /// place. That is D6's defect class exactly: a test named after the
    /// theorem that no longer tests it. A comment cannot prevent it,
    /// because the comment is the thing being ignored.
    ///
    /// So the curve is now pinned by a TRIANGLE, and every side is checked:
    ///
    /// 1. `closed_form` - the spec as an expression, in f64, containing
    ///    **no crate item whatsoever**: not `SPEAR_RELEASE_YAW`, not the
    ///    chain tables, not the local consts. Literally
    ///    `(0.35 + 0.10*min(t/0.12, 1)) * exp(-6*max(t - 0.05, 0))`.
    /// 2. `ANCHORS` - seven f64 values at 15 decimal places, computed by
    ///    evaluating that same expression outside this crate.
    ///    Asserted against `closed_form` at **1e-12**.
    /// 3. `spear_followthrough_yaw` - the real, shipped f32 function,
    ///    swept against `closed_form` on a 1 ms grid over 0..1.2 s, and
    ///    checked against `ANCHORS` directly, both at 1e-6.
    ///
    /// **Why regeneration-from-code now fails loudly.** The f32 function
    /// and the f64 closed form agree only to ~4e-8. So anchors pasted from
    /// this crate's own output miss the 1e-12 gate in step 2 by a factor
    /// of **41,697** - measured, not estimated - and the failure message
    /// says so by name. There is no table of numbers anywhere in this test
    /// that can be regenerated from the code to silence a real failure:
    /// break the function and step 3 fails; paste the broken output into
    /// `ANCHORS` and step 2 fails as well. The only way through is to edit
    /// `closed_form` itself, which is visibly editing the specification
    /// rather than refreshing a table - the distinction the old shape
    /// could not make. (The sibling
    /// `the_arm_onsets_reproduce_an_independently_solved_geometric_root`
    /// already worked this way, deriving its expectation in-test; this is
    /// that pattern applied here, with the external anchors kept on top.)
    ///
    /// MEASURED MARGINS (F4, Thor, 2026-08-03 - this block previously said
    /// "worst gap 5.5e-8, ~18x", and both figures were stale):
    ///
    /// - real f32 fn vs `closed_form`, 1 ms sweep 0..1.2 s: worst
    ///   **6.00e-8 rad** at t = 0.115, so 1e-6 leaves **16.7x**.
    /// - real f32 fn vs `ANCHORS` at the seven points: worst **4.17e-8**
    ///   at t = 0.06, i.e. 24x. (Against the OLD 7-significant-digit f32
    ///   literals it was 5.96e-8 / **16.8x** - Thor's figure, confirmed to
    ///   the digit. Carrying the anchors at f64 precision recovers the
    ///   rounding those literals threw away.)
    ///
    /// FALSIFIABILITY: drop `+ tip_onset` from `spear_followthrough_yaw_from`
    /// and t=0.03 returns 0.350 instead of 0.375 (the drive is silent until
    /// t >= 0.130) - 2.5e-2 off, 25000x the tolerance. That single mutation
    /// is the bug in AUDIT.md #1, and it is what the pre-D6 test could not
    /// see. Any retune of the four local consts, or of `SPEAR_RELEASE_YAW`,
    /// also fails here - deliberately. Retuning the feel means editing
    /// `closed_form` AND recomputing `ANCHORS` from it outside the crate,
    /// and having to do both, in that order, is the whole point.
    #[test]
    fn spear_followthrough_matches_its_hand_computed_curve() {
        // ---- 1. the spec, as an expression. NOTHING from this crate. ----
        // If you change a number in here you are changing the SPECIFICATION
        // of the follow-through, not fixing a test. `ANCHORS` below will
        // stop matching, and that is correct: recompute them from the new
        // expression OUTSIDE this crate before you touch them.
        fn closed_form(t: f64) -> f64 {
            let drive = (t / 0.12_f64).clamp(0.0, 1.0); //      RAMP_S
            let decay = (-6.0_f64 * (t - 0.05_f64).max(0.0)).exp(); // SETTLE_RATE, HOLD_S
            (0.35_f64 + 0.10_f64 * drive) * decay //  release yaw, OVERSHOOT_RAD
        }

        // ---- 2. the external anchors. NOT REGENERABLE FROM THIS CRATE. ----
        // f64, 15 decimal places, from the expression above. The shipped
        // f32 function cannot produce these: at t=0.06 it returns
        // 0.376705855131, which differs in the 8th decimal. If a diff ever
        // shows this table moving to values like that, someone pasted the
        // code's output in - the assert below is what catches it.
        const ANCHORS: [(f64, f64); 7] = [
            (0.00, 0.350_000_000_000_000), // starts exactly on the release yaw
            (0.03, 0.375_000_000_000_000), // drive 0.25, no decay yet
            (0.05, 0.391_666_666_666_667), // drive 5/12, last frame before the settle
            (0.06, 0.376_705_813_433_699), // drive 0.50, decay exp(-0.06)
            (0.12, 0.295_671_068_916_776), // drive saturated, decay exp(-0.42)
            (0.30, 0.100_408_572_066_793), // decay exp(-1.5)
            (1.00, 0.001_505_684_455_862), // decay exp(-5.7)
        ];
        for (t, want) in ANCHORS {
            let derived = closed_form(t);
            assert!(
                (derived - want).abs() < 1e-12,
                "anchor at release_t={t} is {want}, but the closed form gives \
                 {derived} (gap {:.3e}). Either the closed form was edited \
                 without recomputing the anchors, or the anchors were \
                 REGENERATED FROM THIS CRATE'S OWN f32 OUTPUT - which is the \
                 one thing they must never be, and which shows up as a gap \
                 near 4e-8 rather than near 1e-16",
                (derived - want).abs()
            );
        }

        // ---- 3. the real, shipped function against the spec ----
        // dense enough that no feature of the curve hides between samples:
        // the ramp (0..0.12), the hold corner (0.05), the clamp corner
        // (0.12) and the long decay tail all get swept.
        for step in 0..=1200 {
            let t = step as f32 * 0.001;
            let got = spear_followthrough_yaw(t) as f64;
            let want = closed_form(t as f64);
            assert!(
                (got - want).abs() < 1e-6,
                "follow-through at release_t={t}: the closed form says {want}, \
                 the shipped function returns {got} (gap {:.3e})",
                (got - want).abs()
            );
        }

        // ---- and directly against the external anchors ----
        for (t, want) in ANCHORS {
            let got = spear_followthrough_yaw(t as f32) as f64;
            assert!(
                (got - want).abs() < 1e-6,
                "follow-through at release_t={t}: externally computed {want}, got {got}"
            );
        }
    }

    #[test]
    fn kinetic_chain_segment_is_silent_before_its_own_onset() {
        // the tip (segment 7) must show ZERO activation while only the
        // pelvis (segment 0) has begun - proximal-to-distal, not all-at-once
        let t_early = CHAIN_ONSET_OFFSETS[1] * 0.5; // between pelvis onset and lumbar onset
        assert!(chain_segment_scale(0, t_early, 0.05) > 0.0, "pelvis should already be moving");
        assert_eq!(chain_segment_scale(7, t_early, 0.05), 0.0, "tip must still be silent");
    }

    /// Task 3.3 real consumer test: `spear_followthrough_yaw` is the
    /// spear throw-release AND thrust-recovery curve, routed through
    /// `torso_coil_yaw`'s final branch.
    ///
    /// This test replaces an earlier one that asserted the follow-through
    /// was SILENT at release. That was encoding a bug, not a spec: the
    /// old curve sampled the chain's tip from zero, so it returned 0 for
    /// the first 0.125 s (a hard snap to neutral from the release angle)
    /// and then swung NEGATIVE - back toward the coil - which is the
    /// opposite of the "carries past" the docs promise.
    #[test]
    fn spear_followthrough_carries_past_the_release_then_settles() {
        // 1. nothing thrown yet = no twist at all (a fighter merely
        //    holding a spear must not be born mid-unwind)
        assert_eq!(spear_followthrough_yaw(-1.0), 0.0, "no release yet: silent");

        // 2. it BEGINS at the release yaw - no snap to neutral
        assert!(
            (spear_followthrough_yaw(0.0) - SPEAR_RELEASE_YAW).abs() < 1e-5,
            "follow-through must start exactly where the windup ended, got {}",
            spear_followthrough_yaw(0.0)
        );

        // 3. handoff continuity: the last windup frame and the first
        //    follow-through frame must be within a couple of degrees
        let last_windup = torso_coil_yaw(GunKind::Spear, DT, 0.0, false, -1.0);
        let first_follow = torso_coil_yaw(GunKind::Spear, 0.0, 0.0, false, 0.0);
        assert!(
            (last_windup - first_follow).abs() < 0.09, // ~5 deg
            "release must not pop: windup ended at {last_windup}, follow-through starts at {first_follow}"
        );

        // 4. it CARRIES PAST the release angle (same sign, larger
        //    magnitude) rather than reversing through neutral
        let mut peak = 0.0_f32;
        for i in 0..400 {
            let y = spear_followthrough_yaw(i as f32 * 0.002);
            assert!(
                y >= -1e-4,
                "must never swing back the other way (that is the coil direction), got {y}"
            );
            peak = peak.max(y);
        }
        assert!(
            peak > SPEAR_RELEASE_YAW,
            "must carry PAST the release yaw {SPEAR_RELEASE_YAW}, peaked at only {peak}"
        );

        // 5. and it actually relaxes back to neutral
        assert!(
            spear_followthrough_yaw(1.5).abs() < 0.001,
            "must settle to neutral, not hold the carry forever"
        );
    }
}
