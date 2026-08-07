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
//! mouse look - LEFT CLICK fire - RIGHT CLICK focus/aim (hold; a two-stage
//! zoom cycle on scoped guns, a draw on bow/spear) - CTRL/C crouch -
//! SHIFT sprint - RIGHT CLICK + SHIFT drops into a steady SILENT walk that
//! stays on when you release right click - R reload - TAB scoreboard -
//! ESC menu. Mouse buttons swap in Settings.

// A release build is what the desktop shortcut launches - it should look
// like a game, not a dev tool with a terminal parked behind it. Debug
// builds (`cargo build`, `cargo test`) keep the console, since that is
// where panics and eprintln diagnostics still need to land.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod branding;
mod menu_ui;
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
    /// §5: the throwable is in hand (G). RMB aims it, LMB throws it.
    nade_ready: bool,
}

#[derive(Resource)]
struct CamCtl {
    yaw: f32,
    pitch: f32,
    grabbed: bool,
    ads: bool,
    /// §3.6: the LATCHED steady stance. Entered by holding focus and
    /// pressing Shift; survives releasing focus; ends when Shift is
    /// released. While it is set, Shift walks instead of sprinting.
    steady: bool,
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
            steady: false,
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
/// How far the player may look up or down (radians, ±87.7°).
///
/// `cam.pitch` has TWO writers — the mouse (`mouse_look`) and the recoil
/// kick (`input_and_step`) — and this is the one limit both must obey.
/// They used to disagree: the mouse clamped to ±1.53 while recoil clamped
/// to (-0.7, 0.8), and because recoil clamps the ACCUMULATED pitch rather
/// than its own delta, firing while aimed steeply up snapped the view down
/// by as much as 0.73 rad in a single frame. A clamp is not a place for a
/// second opinion: one constant, both call sites.
const LOOK_PITCH_LIMIT: f32 = 1.53;

/// One shot's worth of muzzle climb applied to the look pitch.
///
/// Pure, and separate from the system that calls it, so the clamp can be
/// tested without standing up a Bevy world — the reason the disagreement
/// above went unnoticed is that this arithmetic only ever existed inside
/// a 40-argument system nothing could call.
///
/// Lower pitch is higher aim, so the kick SUBTRACTS. The result is
/// clamped to the same range the mouse may reach, which means the kick
/// can push the aim to the ceiling but can never relocate an aim the
/// player was already legitimately holding.
fn recoil_kicked_pitch(pitch: f32, kick: f32, bloom: f32, brace: f32) -> f32 {
    (pitch - (kick * 6.0 + bloom * 1.5) * brace).clamp(-LOOK_PITCH_LIMIT, LOOK_PITCH_LIMIT)
}

/// How far the bowstring is pulled back, 0..1, for rendering only.
///
/// ONE function for both views. The third-person rig and the first-person
/// viewmodel used to answer this question separately - third person from
/// `cam_ctl.ads` (so a binary 0.25 / 1.0), first person not at all - while
/// the sim was keeping a real 0.15s..0.7s clock the whole time. That is
/// the split brain from ANTI_PATTERNS.md: the client re-deriving what the
/// sim already knows, then drifting from it. Two callers, one source.
///
/// The player's pull is the sim's clock. Note this deliberately has no
/// dead zone below `BOW_DRAW_MIN_S`: the string really is moving in that
/// window, you simply cannot loose a useful arrow yet, and freezing the
/// visual there would misreport what the sim is doing.
///
/// Bots never enter `step_bow_draw` - they fire through `try_fire`, so
/// their `bow_draw_t` is always 0 and reading it would leave every bot
/// bow permanently slack. Their pull comes from the shot cadence
/// instead: `fire_cd` runs down from `fire_period`, so the string draws
/// back as the next arrow approaches and springs forward when it looses.
/// That is a closer account of what a bot is doing than the fixed 0.6
/// this replaces.
fn bow_draw_visual(bow_draw_t: f32, fire_cd: f32, fire_period: f32, is_player: bool) -> f32 {
    if is_player {
        (bow_draw_t / sim::BOW_DRAW_FULL_S).clamp(0.0, 1.0)
    } else if fire_period > 0.0 {
        (1.0 - fire_cd / fire_period).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
// ---- the war bow's STRING, as geometry ----------------------------------
//
// The bow turned HORIZONTAL and the draw did not follow it. Every number
// that placed a hand, an arrow, or the string itself was a separate magic
// literal tuned against the old VERTICAL bow, and a vertical bow hides the
// error: its string spans Y, so a draw hand could ride UP toward the cheek
// and still be touching string at every height. Turn the limbs sideways
// and the string spans X at y = 0 - and the same hand is now a hand's
// width ABOVE the string it is supposedly pulling.
//
// The string was worse than the hand. It was ONE STATIC BOX from tip to
// tip and nothing ever moved it, so at full draw the archer's fingers sat
// 18 cm behind a string that had not budged. The nocked arrow was parented
// to the BOW hand, not to the bow, so it did not track the draw either.
// Three things that must agree were three independent guesses.
//
// So the nock is a FUNCTION now, and the string halves, the arrow and the
// draw hand are all placed from it. They cannot drift apart because there
// is nothing left to drift.

/// Limb tip centre, ±X. The string leaves the tip here.
const BOW_TIP_X: f32 = 0.392;
/// Tip depth. The recurve steps backward, so the tips sit behind the riser.
const BOW_TIP_Z: f32 = -0.088;
/// The nock at REST - a hair behind the tips, which is what gives an
/// undrawn string its slight tension rather than a dead straight line.
const BOW_STRING_Z: f32 = -0.098;
/// How far back the nock travels at full draw. Scaled to this bow's
/// 0.78 m span rather than to a real 0.7 m draw length: at true scale the
/// hand ends up behind the shoulder and the IK chain runs out of arm.
const BOW_DRAW_PULL: f32 = 0.20;
/// String thickness. Thin enough to read as cord, thick enough to survive
/// being one pixel wide at range.
const BOW_STRING_R: f32 = 0.007;
/// The arrow runs BESIDE the riser, not through it. The riser is a solid
/// block 0.052 wide; at this offset the shaft clears it with 2 mm to spare
/// and still sits close enough to read as nocked.
const BOW_ARROW_X: f32 = 0.036;
/// Nocked-arrow length. Shorter than the flying arrow's envelope so the
/// head does not stand a full metre past the bow at rest.
const BOW_ARROW_LEN: f32 = 0.66;
/// Where the draw HAND sits relative to the nock: the fingers hook the
/// string from behind and slightly outboard, they do not occupy it.
const BOW_HAND_OFF: Vec3 = Vec3::new(0.018, 0.0, -0.030);

/// Where the nock is this frame, in the bow model's own local space.
///
/// The single source every other bow placement reads. `draw` is
/// `bow_draw_visual`'s 0..1.
fn bow_nock_local(draw: f32) -> Vec3 {
    Vec3::new(0.0, 0.0, BOW_STRING_Z - BOW_DRAW_PULL * draw.clamp(0.0, 1.0))
}

/// Pose one half of the string - from a limb tip to the nock.
///
/// A drawn string is a V, and a V is two segments, which is why the single
/// tip-to-tip box could never have been animated: there was no vertex at
/// the nock to pull. `side` is -1.0 or +1.0.
///
/// Both the length AND the angle come out of the same subtraction, so a
/// half can never be pointing somewhere its own endpoints are not.
fn bow_string_half(side: f32, draw: f32) -> Transform {
    let tip = Vec3::new(side * BOW_TIP_X, 0.0, BOW_TIP_Z);
    let nock = bow_nock_local(draw);
    let d = nock - tip;
    let len = d.length().max(1e-4);
    // The cube's long axis is +X. Rotating about +Y by θ maps +X to
    // (cos θ, 0, −sin θ), so θ = atan2(−d.z, d.x) aims it down the span.
    Transform {
        translation: (tip + nock) * 0.5,
        rotation: Quat::from_rotation_y((-d.z).atan2(d.x)),
        scale: Vec3::new(len, BOW_STRING_R, BOW_STRING_R),
    }
}

/// Pose the nocked arrow so its NOCK sits on the string.
///
/// `ARROW_NOCK_Z` is where the tail is in the arrow model's own unit
/// envelope, so the root has to sit that far FORWARD of the nock point.
/// Placing the root at the nock - the obvious version - buries a third of
/// the shaft behind the string.
fn bow_nocked_arrow(draw: f32) -> Transform {
    let nock = bow_nock_local(draw);
    Transform::from_xyz(
        BOW_ARROW_X,
        0.0,
        nock.z - ARROW_NOCK_Z * BOW_ARROW_LEN,
    )
    .with_scale(Vec3::splat(BOW_ARROW_LEN))
}

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
/// §5.1: the third-person aim FOV pull-in. This line of the table was
/// DOCUMENTED for a long time and never implemented - the comment above
/// promised -12 degrees while the FOV path only ever lerped toward the
/// held gun's own `zoom_deg`, which for a projectile weapon is no zoom
/// at all. An audit against the brief is what surfaced it.
///
/// Applied on top of whatever the weapon asks for, and only in third
/// person, so first-person focus is untouched.
const TP_FOV_AIM_DELTA: f32 = -12.0;
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
         swap_mouse = {}\nminimap = {}\nsens_idx = {}\nfov_idx = {}\ninvert_y = {}\n\
         # §4.1 vitals: 0 = numbers + bars, 1 = numbers only\n\
         hud_vitals_style = {}\n\
         # §4.3 minimap: rotate with facing, and scale as a percent\n\
         minimap_rotate = {}\nminimap_scale = {}\n\
         # §4.6 crosshair - gap may be NEGATIVE (the arms cross the centre)\n\
         cross_size = {}\ncross_gap = {}\ncross_thickness = {}\n\
         cross_dot = {}\ncross_outline = {}\ncross_outline_px = {}\n\
         cross_color_idx = {}\ncross_r = {}\ncross_g = {}\ncross_b = {}\n\
         cross_alpha = {}\ncross_t_shape = {}\ncross_dynamic = {}\n",
        s.swap_mouse as u8,
        s.minimap as u8,
        s.sens_idx,
        s.fov_idx,
        s.invert_y as u8,
        s.hud_vitals_style,
        s.minimap_rotate as u8,
        s.minimap_scale,
        s.cross_size,
        s.cross_gap,
        s.cross_thickness,
        s.cross_dot as u8,
        s.cross_outline as u8,
        s.cross_outline_px,
        s.cross_color_idx,
        s.cross_rgb.0,
        s.cross_rgb.1,
        s.cross_rgb.2,
        s.cross_alpha,
        s.cross_t_shape as u8,
        s.cross_dynamic as u8,
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
            // §4.1/§4.3: clamped on the way in, same rule as every other
            // index here - the HUD must never have to defend itself
            // against a hand-edited file.
            "hud_vitals_style" => s.hud_vitals_style = v.clamp(0, 1) as u8,
            "minimap_rotate" => s.minimap_rotate = v != 0,
            "minimap_scale" => s.minimap_scale = clamp_setting_i32(v, MINIMAP_SCALE_RANGE),
            // §4.6: same rule as the indices above - every crosshair
            // number is CLAMPED into a drawable range on the way in, so
            // the renderer never has to defend itself against the file.
            "cross_size" => s.cross_size = clamp_setting_i32(v, CROSS_SIZE_RANGE),
            "cross_gap" => s.cross_gap = clamp_setting_i32(v, CROSS_GAP_RANGE),
            "cross_thickness" => s.cross_thickness = clamp_setting_i32(v, CROSS_THICK_RANGE),
            "cross_dot" => s.cross_dot = v != 0,
            "cross_outline" => s.cross_outline = v != 0,
            "cross_outline_px" => s.cross_outline_px = clamp_setting_i32(v, CROSS_OUTLINE_RANGE),
            "cross_color_idx" => {
                s.cross_color_idx = (v.max(0) as usize).min(CROSS_COLOR_CHOICES.len() - 1)
            }
            "cross_r" => s.cross_rgb.0 = v.clamp(0, 255) as u8,
            "cross_g" => s.cross_rgb.1 = v.clamp(0, 255) as u8,
            "cross_b" => s.cross_rgb.2 = v.clamp(0, 255) as u8,
            "cross_alpha" => s.cross_alpha = v.clamp(0, 255) as u8,
            "cross_t_shape" => s.cross_t_shape = v != 0,
            "cross_dynamic" => s.cross_dynamic = v != 0,
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
    ///
    /// `return_efficiency` was DECLARED here and read nowhere, which made
    /// §C.3's "steel springs are worse than tendons, and the mech should
    /// feel it" a comment rather than a mechanic - a mech and a man got
    /// identical output from identical load.
    ///
    /// It is normalised against the human tendon rather than applied
    /// raw, so both halves of the brief hold at once: a human (0.92) gets
    /// Rule 2's stated `× 1.35` EXACTLY, and a mech (0.55) gets
    /// `0.35 × 0.55/0.92 = 0.209` - visibly less spring for the same
    /// coil. Applying it raw would have quietly made the human 1.322 and
    /// broken Rule 2's own worked number.
    fn release_velocity(&self, base: f32) -> f32 {
        let eff = self.return_efficiency / HUMAN_RETURN_EFFICIENCY;
        base * (1.0 + self.stored_energy.clamp(0.0, 1.0) * 0.35 * eff)
    }
}

/// §C.2: "tendons give back ~90-95%". The reference the elastic model
/// normalises against - a human is the 1.0 case by definition.
const HUMAN_RETURN_EFFICIENCY: f32 = 0.92;
/// §C.3: "steel springs are worse than tendons, and the mech should feel it."
const MECH_RETURN_EFFICIENCY: f32 = 0.55;

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

// ---- §B THE 20-SEGMENT MASS-BEARING BODY ---------------------------------
//
// Brief VIII-B §B: "a human body is 14-16 rigid segments, and every one has
// published mass, length, and inertia data" - extended to 20 by splitting
// the trunk in three and adding clavicles and toes.
//
// This table is the DATA half (§B.3 mass fractions, §B.4 lengths, §B.5
// inertia). It is worth landing on its own, ahead of any rig surgery,
// because it is the half that makes the other half checkable: §B.6 asks
// for a mass-closure test and a proportion test, and neither needs a
// single new bone to exist. It also settles the argument §B.5 actually
// cares about - "spring stiffness per segment is DERIVED from mass, not
// hand-guessed - that single change removes most of the 'why does this
// arm feel wrong' tuning loop."
//
// Sources are the standard body-segment-parameter models the brief names:
// Dempster 1955 / Winter 1990 / de Leva 1996. Nothing here is invented;
// where the brief gives a number this table carries the brief's number.

/// One rigid, mass-bearing segment. Fingers are deliberately absent -
/// they are a sub-rig on the hands (§2), and counting them would push
/// this past fifty and miss the point of the list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Segment {
    HeadNeck,
    Thorax,
    Lumbar,
    Pelvis,
    ClavicleL,
    ClavicleR,
    UpperArmL,
    UpperArmR,
    ForearmL,
    ForearmR,
    HandL,
    HandR,
    ThighL,
    ThighR,
    ShankL,
    ShankR,
    FootL,
    FootR,
    ToeL,
    ToeR,
}

const N_SEGMENTS: usize = 20;

/// §B.1's table, in its own order: trunk from the head down, then the
/// arm chain, then the leg chain.
const SEGMENTS: [Segment; N_SEGMENTS] = [
    Segment::HeadNeck,
    Segment::Thorax,
    Segment::Lumbar,
    Segment::Pelvis,
    Segment::ClavicleL,
    Segment::ClavicleR,
    Segment::UpperArmL,
    Segment::UpperArmR,
    Segment::ForearmL,
    Segment::ForearmR,
    Segment::HandL,
    Segment::HandR,
    Segment::ThighL,
    Segment::ThighR,
    Segment::ShankL,
    Segment::ShankR,
    Segment::FootL,
    Segment::FootR,
    Segment::ToeL,
    Segment::ToeR,
];

/// What the standard models publish about one segment.
struct SegmentData {
    name: &'static str,
    /// §B.3: fraction of total body mass, for THIS segment (not the pair).
    mass_frac: f32,
    /// §B.4: length as a fraction of total height H. Zero for the trunk
    /// segments and the clavicle, which §B.4 does not give a long axis
    /// for - it gives shoulder width, hip width and shoulder height
    /// instead, and inventing lengths for them to fill the column would
    /// be fabricating data the brief deliberately did not state.
    len_frac: f32,
    /// §B.5: centre of mass along the segment, from the PROXIMAL joint.
    com_frac: f32,
    /// §B.5: radius of gyration about the segment's own CoM, as a
    /// fraction of its length. This is the number that makes a heavy limb
    /// FEEL heavy - resistance to being whipped is m·(k·L)².
    gyration_frac: f32,
}

fn segment_data(s: Segment) -> SegmentData {
    use Segment::*;
    // the pairs share a row - a left thigh and a right thigh are the same
    // segment on opposite sides, and giving them separate numbers would
    // be an invitation for them to drift apart
    match s {
        HeadNeck => SegmentData { name: "head_neck", mass_frac: 0.081, len_frac: 0.0, com_frac: 0.5, gyration_frac: 0.303 },
        // §B.3: trunk 0.497 total, split thorax 0.216 / lumbar 0.139 /
        // pelvis 0.142 - and the clavicles are CARVED FROM the thorax
        // ("~0.005 (carve from thorax)"), not added beside it, which is
        // what keeps the whole-body sum at exactly 1.000.
        Thorax => SegmentData { name: "thorax", mass_frac: 0.216 - 0.010, len_frac: 0.0, com_frac: 0.5, gyration_frac: 0.33 },
        Lumbar => SegmentData { name: "lumbar", mass_frac: 0.139, len_frac: 0.0, com_frac: 0.5, gyration_frac: 0.33 },
        Pelvis => SegmentData { name: "pelvis", mass_frac: 0.142, len_frac: 0.0, com_frac: 0.5, gyration_frac: 0.33 },
        ClavicleL | ClavicleR => SegmentData { name: "clavicle", mass_frac: 0.005, len_frac: 0.0, com_frac: 0.5, gyration_frac: 0.33 },
        UpperArmL | UpperArmR => SegmentData { name: "upper_arm", mass_frac: 0.028, len_frac: 0.186, com_frac: 0.436, gyration_frac: 0.322 },
        ForearmL | ForearmR => SegmentData { name: "forearm", mass_frac: 0.016, len_frac: 0.146, com_frac: 0.430, gyration_frac: 0.303 },
        HandL | HandR => SegmentData { name: "hand", mass_frac: 0.006, len_frac: 0.108, com_frac: 0.506, gyration_frac: 0.297 },
        ThighL | ThighR => SegmentData { name: "thigh", mass_frac: 0.100, len_frac: 0.245, com_frac: 0.433, gyration_frac: 0.323 },
        ShankL | ShankR => SegmentData { name: "shank", mass_frac: 0.0465, len_frac: 0.246, com_frac: 0.433, gyration_frac: 0.302 },
        // §B.3 splits the foot: hindfoot 0.011 / toe 0.0035. §B.4 gives
        // one foot LENGTH of 0.152 H for the pair, split here in the same
        // proportion the mass is - the forefoot is about a quarter of the
        // foot, which is also what makes the toe-off lever the right size.
        FootL | FootR => SegmentData { name: "foot", mass_frac: 0.011, len_frac: 0.152 * 0.75, com_frac: 0.50, gyration_frac: 0.475 },
        ToeL | ToeR => SegmentData { name: "toe", mass_frac: 0.0035, len_frac: 0.152 * 0.25, com_frac: 0.50, gyration_frac: 0.475 },
    }
}

/// §B.4's non-length proportions, as fractions of total height H.
/// §owner MECH REFIT: the hull is built at full size and worn at 85%.
///
/// "Reduce the box-shaped main body by ~15% while maintaining its heavy
/// appearance" - and those two halves are answered separately. This
/// constant is the first half; the DENSITY pass in `spawn_armor_rig` is
/// the second. A slab is heavy because it is big, which is the cheap
/// kind; a machine is heavy because every surface on it is doing a job,
/// and that survives being made smaller.
///
/// Applied as a UNIFORM scale on the rig root rather than by editing a
/// hundred plate coordinates. Uniform matters: the hull is full of
/// rotated cylinders, and a non-uniform parent scale shears every one of
/// them. It also means every hardpoint - shoulder housings, rocket pod,
/// gatling arm - moves inboard by exactly the same factor for free,
/// where hand-editing would have left them floating off a narrower hull.
const MECH_HULL_SCALE: f32 = 0.85;

/// The waist - where the trunk meets the legs, in root space.
///
/// Named because §B.2's trunk split made it load-bearing in two places
/// at once: the lumbar SPAWNS here, and `sync_fighters` has to subtract
/// it when writing the thorax's own translation, which is still
/// expressed in root space. Two bare 0.63s that have to agree is one
/// too many.
const WAIST_Y: f32 = 0.63;

/// The thorax's LOCAL height under the lumbar, given the root-space
/// height it needs to end up at.
///
/// A two-line function for a subtraction, and it earns its place: the
/// first version of the trunk split left the thorax carrying `WAIST_Y`
/// while its new parent carried it too, which lifted the entire upper
/// body a full waist off the legs. The rig visibly came apart in a
/// capture and NOT ONE test failed, because every rig test in this file
/// measures either an ANGLE (separation, the kinetic chain) or the head
/// BAND (derived from `gait_pose`, not from the transform hierarchy) -
/// and neither notices a torso floating in the air.
///
/// `thorax_height_is_conserved_across_the_trunk_split` is the guard that
/// was missing.
fn thorax_local_y(hip_y: f32, crouch_dip: f32, breath: f32) -> f32 {
    hip_y - WAIST_Y - crouch_dip + breath
}

/// §B.2: how much of the trunk's twist the LUMBAR carries, the thorax
/// taking the remainder.
///
/// Below half on purpose. Human axial rotation is not evenly distributed
/// - the thoracic spine contributes appreciably more range than the
/// lumbar, which is built to resist twist far more than it permits it.
/// A 50/50 split would read as a body hinged at the belt.
const LUMBAR_TWIST_SHARE: f32 = 0.38;

/// §B.1 #19-20: how far the forefoot plantar-flexes at full toe-off.
///
/// 40 degrees. Human sprint toe-off runs to roughly 25-30 deg of
/// metatarsophalangeal extension plus the ankle's own plantar flexion;
/// this rig hinges the whole forefoot at one joint, so it carries the
/// combined angle rather than the joint's own.
const TOE_OFF_MAX: f32 = 40.0 * PI / 180.0;

/// §B.1 #19-20: the forefoot's plantar flexion at a point in the gait.
///
/// Pure, and extracted for the same reason `carry_offset` is: §B.6 asks
/// for a toe-off test - "assert the toe segment rotates through its
/// plantar-flexion range at contact-exit; no toe rotation means the run
/// is still a glide" - and a test that has to stand up Bevy to sample one
/// angle is a test nobody will keep.
///
/// `phase` is the leg's own gait phase (the rig's phase plus the leg's
/// half-cycle offset); `amp` is the speed fraction. Rectified and
/// squared: a toe PUSHES and never pulls, and the square is what makes
/// the drive a snap at the end of stance instead of a slow roll across
/// the whole cycle.
fn toe_off_angle(phase: f32, amp: f32) -> f32 {
    ((phase - FRAC_PI_2).sin().max(0.0).powi(2) * TOE_OFF_MAX * amp)
        .clamp(0.0, TOE_OFF_MAX)
}

const SHOULDER_WIDTH_FRAC: f32 = 0.259;
const HIP_WIDTH_FRAC: f32 = 0.191;
const SHOULDER_HEIGHT_FRAC: f32 = 0.818;

/// The height the rig is actually built at, in metres.
///
/// §B.4 gives every length as a fraction of H and works its example at
/// H = 1.8 m. This is the H the proportion test measures the LIVE rig
/// against, so it has to be the rig's real height rather than a nominal
/// one - see `the_rig_matches_the_published_proportions`.
const RIG_HEIGHT_M: f32 = 1.8;

/// §B.5: a segment's resistance to being whipped about its proximal
/// joint - `m·(k·L)²`, in body-mass × H² units.
///
/// The whole point of the inertia column. A spring driving a segment
/// should be stiffer for a heavier, longer one, and this is the number
/// that says by how much - so `derived_spring_k` can stop guessing.
fn segment_inertia(s: Segment) -> f32 {
    let d = segment_data(s);
    let r = d.gyration_frac * d.len_frac;
    d.mass_frac * r * r
}

/// §B.5: spring stiffness for a segment, DERIVED from its inertia.
///
/// A critically-damped spring's stiffness for a target natural frequency
/// is `k = I·ω²`. So one tuning knob - the frequency you want the segment
/// to settle at - replaces a per-segment guess, and the mass model
/// supplies the rest. A forearm and a thigh driven at the same frequency
/// get different k because they ARE different, which is exactly the
/// "why does this arm feel wrong" loop §B.5 says this removes.
///
/// Returned in the same units the existing `damped_spring` constants use,
/// scaled by `SPRING_K_REFERENCE` so the numbers land in the range this
/// file already works in rather than in body-mass·H² units nobody can
/// read.
const SPRING_K_REFERENCE: f32 = 1.0e4;
fn derived_spring_k(s: Segment, omega: f32) -> f32 {
    segment_inertia(s) * omega * omega * SPRING_K_REFERENCE
}

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
    /// §owner: the standing CLASS pick - who you are for the match.
    class: sim::Class,
    /// §owner: kills to win a TDM - a quick 30 or a long 60.
    tdm_target: u32,
    /// cosmetic only: hat + tunic colors picked before the match
    hat: usize,
    tunic: usize,
    /// §8.1: the helmet SHAPE, orthogonal to `hat` (which is its tint).
    helmet: usize,
    /// §C tier 2: the plate assembled in the Forge.
    armor: sim::ArmorLoadout,
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
    /// §8.1. Added AFTER slots shipped, which is why `from_line` treats it
    /// as optional - see there.
    helmet: usize,
    /// §C tier 2: the plate bitmask. Added after the helmet, same
    /// optional-trailing-field treatment and the same reason.
    armor: u32,
}

impl ForgeProfile {
    fn from_selected(sel: &Selected) -> Self {
        ForgeProfile {
            hat: sel.hat,
            tunic: sel.tunic,
            melee_axe: sel.melee_axe,
            grenade_preset: sel.grenade_preset,
            helmet: sel.helmet,
            armor: sel.armor.0,
        }
    }
    fn apply_to(&self, sel: &mut Selected) {
        sel.hat = self.hat;
        sel.tunic = self.tunic;
        sel.helmet = self.helmet;
        sel.melee_axe = self.melee_axe;
        sel.grenade_preset = self.grenade_preset;
        sel.armor = sim::ArmorLoadout(self.armor);
    }
    /// A compact one-line format - no serde dependency needed for six
    /// small fields. `hat,tunic,melee_axe,grenade_preset,helmet,armor`.
    fn to_line(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.hat,
            self.tunic,
            self.melee_axe as u8,
            self.grenade_preset,
            self.helmet,
            self.armor
        )
    }
    /// Parses every format that has shipped: the original four fields,
    /// the five with a helmet, and the six with a harness.
    ///
    /// Trailing fields are OPTIONAL rather than required because slot
    /// files already exist on disk from before each of them, and the
    /// alternative - rejecting them - would silently wipe someone's saved
    /// profiles the first time they launched an updated build. A missing
    /// helmet reads as index 0, the FIELD CAP; a missing harness reads as
    /// LINE's default plate. Both are exactly what those profiles were
    /// wearing when they were written.
    ///
    /// Everything up to the comma count is still strict: a malformed
    /// field is still `None`. A field that is PRESENT and garbage is an
    /// error, and only an ABSENT one is a default - otherwise this would
    /// quietly accept a corrupt file as an old one.
    fn from_line(s: &str) -> Option<Self> {
        let mut it = s.trim().split(',');
        Some(ForgeProfile {
            hat: it.next()?.parse().ok()?,
            tunic: it.next()?.parse().ok()?,
            melee_axe: it.next()?.parse::<u8>().ok()? != 0,
            grenade_preset: it.next()?.parse().ok()?,
            helmet: match it.next() {
                None => 0,
                Some(v) => v.trim().parse().ok()?,
            },
            armor: match it.next() {
                None => sim::default_harness(sim::Class::Line).0,
                Some(v) => v.trim().parse().ok()?,
            },
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
            class: sim::Class::Line,
            tdm_target: sim::TDM_TARGET,
            hat: 0,
            tunic: 0,
            helmet: 0,
            // the class default, so a player who never opens the armour
            // rows is exactly as fast and as tough as before they existed
            armor: sim::default_harness(sim::Class::Line),
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

    // ---- §4.1 / §4.3 (Brief VIII): HUD readout options -------------
    /// §4.1 `hud_vitals_style`: 0 = numbers + bars (default), 1 = numbers
    /// only. The brief names this setting explicitly; it existed in the
    /// spec and nowhere in the code, because the bars it switches off had
    /// never been built.
    hud_vitals_style: u8,
    /// §4.3: the minimap rotates with facing. Tunable per the brief -
    /// a rotating map is easier for some players to read and actively
    /// disorienting for others, which is exactly what a setting is for.
    minimap_rotate: bool,
    /// §4.3: minimap scale, PERCENT (25..=100, default 70). Stored as an
    /// integer because the whole settings file is `key = <i64>` - a float
    /// here would need a second parser for one value.
    minimap_scale: i32,

    // ---- §4.6 (Brief VIII): the crosshair family -------------------
    // Every one of these is clamped on load (`parse_settings`) to the
    // range constants below, so a hand-edited or stale file can only
    // ever produce a DRAWABLE crosshair - never a zero-size rect, never
    // an inverted one, never an out-of-range colour channel.
    /// Arm length in pixels.
    cross_size: i32,
    /// Distance from the exact centre to each arm's INNER edge.
    /// **Negative is legal and specified** - the arms then cross the
    /// centre, which is a real and common preference.
    cross_gap: i32,
    /// Arm width in pixels (the cross-axis of each arm).
    cross_thickness: i32,
    /// Centre dot. Off by default.
    cross_dot: bool,
    /// Dark backing outline. On by default, `cross_outline_px` wide.
    cross_outline: bool,
    cross_outline_px: i32,
    /// Index into `CROSS_COLOR_CHOICES`; the last entry is CUSTOM and
    /// reads `cross_rgb` instead of a fixed triple.
    cross_color_idx: usize,
    /// The custom colour, 0-255 per channel. Spec default green.
    cross_rgb: (u8, u8, u8),
    /// Opacity 0-255. Spec default 200.
    cross_alpha: u8,
    /// T-shape: drop the TOP arm so nothing occludes the target's head.
    cross_t_shape: bool,
    /// Dynamic (arms bloom with the live aim cone) vs the spec default,
    /// **classic static**.
    cross_dynamic: bool,
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
            // §4.1/§4.3 defaults, verbatim from the brief: bars ON,
            // rotate ON ("rotates with facing (tunable)"), scale 0.7.
            hud_vitals_style: 0,
            minimap_rotate: true,
            minimap_scale: MINIMAP_SCALE_DEFAULT,
            // §4.6 defaults, verbatim from the brief: size 5, gap 0,
            // thickness 1, dot off, outline on at 1, green 50/250/50,
            // alpha 200, no T-shape, classic static.
            cross_size: CROSS_SIZE_DEFAULT,
            cross_gap: CROSS_GAP_DEFAULT,
            cross_thickness: CROSS_THICK_DEFAULT,
            cross_dot: false,
            cross_outline: true,
            cross_outline_px: CROSS_OUTLINE_DEFAULT,
            cross_color_idx: CROSS_COLOR_DEFAULT_IDX,
            cross_rgb: CROSS_RGB_DEFAULT,
            cross_alpha: CROSS_ALPHA_DEFAULT,
            cross_t_shape: false,
            cross_dynamic: false,
        }
    }
}

// ---- §4.6 (Brief VIII): the crosshair, as DRAWN GEOMETRY ------------------
//
// It used to be a single hardcoded `+` text glyph: one size, one colour,
// no gap, no thickness, no dot, no outline - and, because `GameSettings`
// had no crosshair fields at all, §4.9's "crosshair settings round-trip
// through the settings file" was not a failing test, it was an
// unwriteable one. This section is the whole family: the clamp ranges,
// the pure geometry, and the pure colour decision. Everything here is a
// free function so it is testable without spinning up Bevy - the same
// extraction this codebase already made for `view_recoil_offset`,
// `bow_sway_deg` and `splash_alpha`.

/// Arm length, px. Lower bound is 1: a zero-length arm is not a
/// preference, it is an invisible crosshair.
/// §4.1: how many segments the health bar is cut into. Ten, so one
/// segment is exactly 10 HP against a 100 HP pool and the player can
/// count damage off the bar without reading the number.
const VITALS_SEGMENTS: usize = 10;
/// §4.1: armour pips. Four - enough granularity to separate the five
/// sets, few enough to count without looking.
const ARMOR_PIPS: usize = 4;
/// §4.1: the flat-torso value a FULL pip cluster represents. Folk armour
/// (45) is the heaviest set in `armor_spec`, so a full cluster means
/// "the best protection in the game" rather than an invented ceiling.
/// If a heavier set is ever added, this is the one number to raise.
const ARMOR_PIP_REFERENCE: f32 = 45.0;
/// §4.3: minimap scale as a PERCENT of the base size. The brief's
/// 0.25-1.0 range and 0.7 default, in integer percent.
const MINIMAP_SCALE_RANGE: (i32, i32) = (25, 100);
const MINIMAP_SCALE_DEFAULT: i32 = 70;
const CROSS_SIZE_RANGE: (i32, i32) = (1, 12);
/// Centre-to-inner-edge gap, px. **The low bound is negative on
/// purpose** (§4.6) - at gap < 0 the arms overlap through the centre.
const CROSS_GAP_RANGE: (i32, i32) = (-5, 12);
/// Arm width, px. Also the diameter of the centre dot.
const CROSS_THICK_RANGE: (i32, i32) = (1, 5);
/// Outline width, px. 0 is legal and means "outline on, but hairline-off".
const CROSS_OUTLINE_RANGE: (i32, i32) = (0, 3);
const CROSS_SIZE_DEFAULT: i32 = 5;
const CROSS_GAP_DEFAULT: i32 = 0;
const CROSS_THICK_DEFAULT: i32 = 1;
const CROSS_OUTLINE_DEFAULT: i32 = 1;
const CROSS_ALPHA_DEFAULT: u8 = 200;
/// Spec default: green 50,250,50. This is a SIGNAL colour, deliberately
/// outside `branding::palette` - the art palette is warm dust/gold/bronze
/// and a crosshair drawn in it would vanish against this game's own
/// ground. Readability beats theme at the centre of the screen.
const CROSS_RGB_DEFAULT: (u8, u8, u8) = (50, 250, 50);

/// Colour presets. The LAST entry is the custom slot - it ignores the
/// triple stored here and reads `GameSettings::cross_rgb`.
const CROSS_COLOR_CHOICES: [(&str, (u8, u8, u8)); 8] = [
    ("GREEN", CROSS_RGB_DEFAULT),
    ("WHITE", (255, 255, 255)),
    ("CYAN", (40, 235, 245)),
    ("YELLOW", (250, 235, 60)),
    ("MAGENTA", (245, 70, 220)),
    ("RED", (240, 60, 55)),
    ("BLUE", (70, 130, 255)),
    ("CUSTOM", CROSS_RGB_DEFAULT),
];
const CROSS_COLOR_DEFAULT_IDX: usize = 0;
const CROSS_COLOR_CUSTOM_IDX: usize = CROSS_COLOR_CHOICES.len() - 1;
/// Opacity presets the settings row cycles through. The stored value is
/// a raw u8, so a hand-edited file may sit anywhere in 0..=255.
const CROSS_ALPHA_CHOICES: [u8; 6] = [80, 120, 160, 200, 230, 255];
/// Pixels of extra gap per radian of live aim cone, for the DYNAMIC
/// crosshair. Shared with `stability_bracket` so the two readouts of the
/// same spread can never drift apart (OPERATION.md rule 6).
const CROSS_SPREAD_PX_PER_RAD: f32 = 2400.0;
/// How far the arms kick outward during the kill-confirm pop, px.
const CROSS_KILL_POP_PX: f32 = 3.0;

/// One axis-aligned crosshair rectangle, in PIXELS relative to the exact
/// screen centre. `+left` is right, `+top` is down - Bevy UI's own sign
/// convention, so these drop straight into a `Node`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CrossRect {
    left: f32,
    top: f32,
    w: f32,
    h: f32,
}

/// Arm indices. 4 is the centre dot; the renderer uses the same numbering.
const CROSS_ARM_TOP: usize = 0;
const CROSS_ARM_RIGHT: usize = 1;
const CROSS_ARM_BOTTOM: usize = 2;
const CROSS_ARM_LEFT: usize = 3;
const CROSS_PIECES: usize = 5;

/// The four arms, from (size, gap, thickness).
///
/// Each arm runs OUTWARD from `gap` to `gap + size`, so `size` is the
/// arm's own length and `gap` only moves it - the two never fight. That
/// is what makes a negative gap safe: it slides the inner edge past the
/// centre without ever shortening the arm, so no rect can invert or
/// collapse (§4.6 explicitly allows negative gap).
fn crosshair_arm_rects(size: f32, gap: f32, thickness: f32) -> [CrossRect; 4] {
    let len = size.max(1.0);
    let thick = thickness.max(1.0);
    let half = thick * 0.5;
    let mut out = [CrossRect { left: 0.0, top: 0.0, w: 0.0, h: 0.0 }; 4];
    out[CROSS_ARM_TOP] = CrossRect { left: -half, top: -(gap + len), w: thick, h: len };
    out[CROSS_ARM_RIGHT] = CrossRect { left: gap, top: -half, w: len, h: thick };
    out[CROSS_ARM_BOTTOM] = CrossRect { left: -half, top: gap, w: thick, h: len };
    out[CROSS_ARM_LEFT] = CrossRect { left: -(gap + len), top: -half, w: len, h: thick };
    out
}

/// The centre dot - a square of the arm thickness, centred exactly.
fn crosshair_dot_rect(thickness: f32) -> CrossRect {
    let d = thickness.max(1.0);
    CrossRect { left: -d * 0.5, top: -d * 0.5, w: d, h: d }
}

/// The dark backing rect for a piece: the same rect grown `px` on every
/// side. Grown, never shrunk, so an outline can never eat its own fill.
fn crosshair_outline_rect(r: CrossRect, px: f32) -> CrossRect {
    let px = px.max(0.0);
    CrossRect { left: r.left - px, top: r.top - px, w: r.w + 2.0 * px, h: r.h + 2.0 * px }
}

/// §4.6 T-shape: the TOP arm is dropped so the crosshair never sits on
/// the head of what you are aiming at. Every other piece is unaffected.
fn crosshair_arm_shown(arm: usize, t_shape: bool) -> bool {
    !(t_shape && arm == CROSS_ARM_TOP)
}

/// The gap actually drawn this frame. **Classic static** (the default)
/// ignores the aim cone entirely - the gap you set is the gap you get.
/// Dynamic adds the live spread, so the arms bloom while moving/firing.
fn crosshair_gap_px(base_gap: i32, spread: f32, dynamic: bool) -> f32 {
    if dynamic {
        base_gap as f32 + spread.max(0.0) * CROSS_SPREAD_PX_PER_RAD
    } else {
        base_gap as f32
    }
}

/// The player's chosen colour: a preset, or the custom triple.
fn crosshair_rgb(s: &GameSettings) -> (u8, u8, u8) {
    let idx = s.cross_color_idx.min(CROSS_COLOR_CUSTOM_IDX);
    if idx == CROSS_COLOR_CUSTOM_IDX {
        s.cross_rgb
    } else {
        CROSS_COLOR_CHOICES[idx].1
    }
}

/// What the crosshair is SAYING this frame, in strict priority order.
/// Extracted from `hud_system`'s old inline match so the drawn geometry
/// and any future readout cannot disagree about the same event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrossFeedback {
    /// §5.2 (Brief VI): a scoped-class weapon, unscoped - draw NOTHING.
    Hidden,
    Kill,
    Headshot,
    Hit,
    /// §5.3: the muzzle→aim-point segment is blocked close-by.
    Blocked,
    Idle,
}

/// The priority ladder, unchanged from the glyph version: hiding wins
/// over everything (a no-scope must not leak an aim point through a
/// hitmarker), then kill, then headshot, then hit, then blocked.
fn crosshair_feedback(
    noscope_hidden: bool,
    fresh_kill: bool,
    fresh_hit_head: Option<bool>,
    blocked: bool,
) -> CrossFeedback {
    if noscope_hidden {
        CrossFeedback::Hidden
    } else if fresh_kill {
        CrossFeedback::Kill
    } else if let Some(head) = fresh_hit_head {
        if head {
            CrossFeedback::Headshot
        } else {
            CrossFeedback::Hit
        }
    } else if blocked {
        CrossFeedback::Blocked
    } else {
        CrossFeedback::Idle
    }
}

/// The colour to paint the geometry. Only `Idle` uses the player's
/// settings - every feedback state keeps its own full-strength signal
/// colour, so turning the alpha down cannot mute a hitmarker.
fn crosshair_color(fb: CrossFeedback, rgb: (u8, u8, u8), alpha: u8) -> Color {
    match fb {
        CrossFeedback::Hidden => Color::srgba(0.0, 0.0, 0.0, 0.0),
        CrossFeedback::Kill => Color::srgb(1.0, 0.55, 0.2),
        CrossFeedback::Headshot => Color::srgb(1.0, 0.85, 0.2),
        CrossFeedback::Hit => Color::srgb(1.0, 0.3, 0.25),
        CrossFeedback::Blocked => Color::srgba(1.0, 0.55, 0.1, 0.9),
        CrossFeedback::Idle => Color::srgba(
            rgb.0 as f32 / 255.0,
            rgb.1 as f32 / 255.0,
            rgb.2 as f32 / 255.0,
            alpha as f32 / 255.0,
        ),
    }
}

/// Click-to-cycle for a bounded integer settings row: step forward one,
/// wrap to the low end. One helper, so no row can wrap differently.
fn cycle_i32(v: i32, range: (i32, i32)) -> i32 {
    if v >= range.1 {
        range.0
    } else {
        (v + 1).max(range.0)
    }
}

/// Click-to-cycle alpha: the next preset strictly above the current
/// value, wrapping to the lowest. Defined against the VALUE, not an
/// index, so it still behaves from a hand-edited file's 137.
fn cycle_alpha(a: u8) -> u8 {
    CROSS_ALPHA_CHOICES
        .iter()
        .copied()
        .find(|&c| c > a)
        .unwrap_or(CROSS_ALPHA_CHOICES[0])
}

/// Clamp a value read off disk into a settings range.
fn clamp_setting_i32(v: i64, range: (i32, i32)) -> i32 {
    v.clamp(range.0 as i64, range.1 as i64) as i32
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

// ---- §8.1 (Brief VIII): the HELMET part library -------------------------
//
// Until now "hat" was a COLOUR pick: four entries in `HAT_CHOICES`, all
// wearing the identical brim-crown-band. Two players who picked
// differently had the same silhouette in a different tint, and at the
// ranges this game is played at, tint is the first thing the fog and the
// team-signal wash out. Silhouette is what survives.
//
// So a helmet is now GEOMETRY. Each entry below is a list of primitive
// pieces placed in the hat socket's local space, and the four colours
// remain, orthogonal - five shapes x four tints.
//
// The library is plain data on purpose. A helmet that is a `&[HelmetPiece]`
// can be checked by a test that never opens a window: bounds, piece count,
// and the frozen-shape guarantee below are all decidable from the table.
// A helmet that was a hand-written run of `commands.spawn` calls - which is
// exactly what this replaces - could only be checked by looking at it.
//
// COSMETIC. A helmet never changes a hitbox. In particular it does not
// change the HEAD BAND: `sim` classifies a head hit above height fraction
// 0.82 of the fighter capsule, computed from the capsule alone, and a tall
// crest sticking out above that line is decoration hanging in free air. Do
// not "fix" that by growing the capsule - the band is a gameplay contract
// shared with every hit test and every bot's aim.

/// Which unit primitive a helmet piece is built from. The three the
/// `ModelKit` already carries - the library deliberately invents no new
/// meshes, since the whole point is that shape comes from ARRANGEMENT.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Prim {
    Cyl,
    Cube,
    Ball,
}

/// Which material slot a piece paints with. `Tint` is the player's
/// `HAT_CHOICES` pick; the rest are fixed kit greys, so every helmet keeps
/// some metal on it and no tint choice can turn one into a flat blob.
#[derive(Clone, Copy, PartialEq, Debug)]
enum HelmetMat {
    Tint,
    Gunmetal,
    Steel,
    Glow,
}

/// One primitive in a helmet, in hat-socket local space.
#[derive(Clone, Copy, PartialEq, Debug)]
struct HelmetPiece {
    prim: Prim,
    mat: HelmetMat,
    pos: (f32, f32, f32),
    scale: (f32, f32, f32),
    /// Lean about X (nose-down positive) and about Z (splay outward),
    /// radians. Two axes because the library needs both: a brow slab
    /// overhangs FORWARD and a horn sweeps SIDEWAYS, and one angle cannot
    /// do both. Most pieces use neither.
    pitch: f32,
    roll: f32,
}

const fn hp(
    prim: Prim,
    mat: HelmetMat,
    pos: (f32, f32, f32),
    scale: (f32, f32, f32),
) -> HelmetPiece {
    HelmetPiece { prim, mat, pos, scale, pitch: 0.0, roll: 0.0 }
}

/// Same, leaning. `pitch` tips it forward/back, `roll` splays it out.
const fn hpr(
    prim: Prim,
    mat: HelmetMat,
    pos: (f32, f32, f32),
    scale: (f32, f32, f32),
    pitch: f32,
    roll: f32,
) -> HelmetPiece {
    HelmetPiece { prim, mat, pos, scale, pitch, roll }
}

/// The antenna, worn by EVERY helmet.
///
/// Shared rather than repeated per entry because it is not decoration: the
/// glowing tip is how you read a robot's facing at distance, and a helmet
/// that dropped it would be strictly harder to fight against than the
/// others. Cosmetic choices must not change how legible a target is.
const HELMET_ANTENNA: [HelmetPiece; 2] = [
    hp(Prim::Cyl, HelmetMat::Steel, (0.13, 1.22, 0.0), (0.015, 0.13, 0.015)),
    hp(Prim::Cube, HelmetMat::Glow, (0.13, 1.30, 0.0), (0.035, 0.035, 0.035)),
];

/// FIELD CAP - the original. FROZEN: these three pieces carry the exact
/// positions and scales the body had before the library existed, so index
/// 0 is a byte-for-byte continuation of the old look and nobody's saved
/// profile silently changes shape. `helmet_zero_is_the_frozen_field_cap`
/// pins them.
const HELM_CAP: [HelmetPiece; 3] = [
    hp(Prim::Cyl, HelmetMat::Tint, (0.0, 1.02, 0.0), (0.72, 0.028, 0.72)),
    hp(Prim::Cyl, HelmetMat::Tint, (0.0, 1.11, 0.0), (0.36, 0.18, 0.36)),
    hp(Prim::Cyl, HelmetMat::Gunmetal, (0.0, 1.045, 0.0), (0.365, 0.04, 0.365)),
];

/// VISOR - a closed combat helm. Round dome, a brow slab that overhangs
/// the eyes, and two cheek plates. Reads as the heaviest of the five from
/// the front and is the one that most changes the head's outline.
const HELM_VISOR: [HelmetPiece; 5] = [
    hp(Prim::Ball, HelmetMat::Tint, (0.0, 1.08, 0.0), (0.44, 0.40, 0.44)),
    hpr(Prim::Cube, HelmetMat::Gunmetal, (0.0, 1.03, -0.19), (0.40, 0.075, 0.10), -0.28, 0.0),
    hp(Prim::Cube, HelmetMat::Tint, (-0.19, 0.97, -0.06), (0.055, 0.16, 0.20)),
    hp(Prim::Cube, HelmetMat::Tint, (0.19, 0.97, -0.06), (0.055, 0.16, 0.20)),
    hp(Prim::Cyl, HelmetMat::Steel, (0.0, 1.155, 0.0), (0.30, 0.030, 0.30)),
];

/// CREST - officer's helm. Low dome with a fore-and-aft blade running over
/// the crown, the classic parade silhouette; the tallest entry.
const HELM_CREST: [HelmetPiece; 4] = [
    hp(Prim::Ball, HelmetMat::Tint, (0.0, 1.06, 0.0), (0.42, 0.34, 0.42)),
    hp(Prim::Cube, HelmetMat::Gunmetal, (0.0, 1.23, 0.0), (0.045, 0.19, 0.52)),
    hp(Prim::Cube, HelmetMat::Tint, (0.0, 1.31, 0.02), (0.075, 0.055, 0.40)),
    hp(Prim::Cyl, HelmetMat::Steel, (0.0, 1.015, 0.0), (0.44, 0.035, 0.44)),
];

/// HOOD - a scout's soft cowl. No brim and no crown: a shallow shell with
/// a flap down the back of the neck. The lowest-profile entry, and the one
/// that leaves the most of the head shell showing.
const HELM_HOOD: [HelmetPiece; 3] = [
    hp(Prim::Ball, HelmetMat::Tint, (0.0, 1.02, 0.0), (0.40, 0.24, 0.44)),
    hpr(Prim::Cube, HelmetMat::Tint, (0.0, 0.98, 0.17), (0.30, 0.20, 0.045), 0.22, 0.0),
    hp(Prim::Cube, HelmetMat::Gunmetal, (0.0, 1.04, -0.17), (0.16, 0.045, 0.06)),
];

/// HORNS - the intimidation pick. Band and dome with two swept horns. The
/// widest entry, which is the point: it is unmistakable in peripheral
/// vision even when the tint is lost to fog.
const HELM_HORNS: [HelmetPiece; 5] = [
    hp(Prim::Cyl, HelmetMat::Tint, (0.0, 1.10, 0.0), (0.38, 0.16, 0.38)),
    hp(Prim::Cyl, HelmetMat::Gunmetal, (0.0, 1.03, 0.0), (0.40, 0.045, 0.40)),
    hpr(Prim::Cube, HelmetMat::Steel, (-0.22, 1.20, 0.0), (0.05, 0.26, 0.05), 0.0, 0.38),
    hpr(Prim::Cube, HelmetMat::Steel, (0.22, 1.20, 0.0), (0.05, 0.26, 0.05), 0.0, -0.38),
    hp(Prim::Ball, HelmetMat::Tint, (0.0, 1.17, 0.0), (0.30, 0.14, 0.30)),
];

/// How many helmets the library holds.
///
/// Spelled out rather than taken as `HELMET_CHOICES.len()`: the turntable
/// mounts them in a fixed-size array, and using the table's own length in
/// that array TYPE makes rustc const-evaluate a static full of `&str` and
/// `&[HelmetPiece]` references inside a const-generic argument, which it
/// crashes on (STATUS_STACK_BUFFER_OVERRUN) in a file this size. One
/// literal, with `helmet_library_is_the_declared_size` pinning the two
/// together so they cannot drift apart silently.
const N_HELMETS: usize = 5;

/// The library. Name plus pieces; the tint is chosen separately from
/// `HAT_CHOICES`, so this is 5 shapes x 4 colours = 20 heads.
const HELMET_CHOICES: [(&str, &[HelmetPiece]); N_HELMETS] = [
    ("FIELD CAP", &HELM_CAP),
    ("VISOR", &HELM_VISOR),
    ("CREST", &HELM_CREST),
    ("HOOD", &HELM_HOOD),
    ("HORNS", &HELM_HORNS),
];

/// Every piece must sit inside this box, in hat-socket local space.
///
/// These are not style guidance - each bound is a failure the geometry can
/// actually have. Below `Y_MIN` a piece is inside the head shell (z-fighting
/// through the face); above `Y_MAX` it reads as floating above the fighter;
/// outside `XZ_MAX` it pokes through a wall the player thinks they are
/// safely behind. `helmet_pieces_stay_in_the_socket_envelope` checks the
/// whole library against them, so a new helmet cannot be added wrong.
const HELMET_Y_MIN: f32 = 0.86;
const HELMET_Y_MAX: f32 = 1.36;
const HELMET_XZ_MAX: f32 = 0.42;

/// Hang one helmet under the hat socket, and return the GROUP entity it
/// hangs from.
///
/// Replaces the five hand-written `commands.spawn` chains this used to be.
/// The antenna is appended to whichever helmet was picked rather than
/// living in the table, so it cannot be forgotten by a new entry.
///
/// The group exists so the Forge turntable can mount all five at once and
/// switch between them with one `Visibility` write - the same trick the
/// preview already plays with class rigs and the weapon rack. A live
/// fighter mounts exactly one and never touches it again.
fn spawn_helmet(
    commands: &mut Commands,
    kit: &ModelKit,
    look: &SoldierLook,
    socket: Entity,
    helmet: usize,
) -> Entity {
    let group = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .set_parent(socket)
        .id();
    let (_, pieces) = HELMET_CHOICES[helmet % HELMET_CHOICES.len()];
    for p in pieces.iter().chain(HELMET_ANTENNA.iter()) {
        let mesh = match p.prim {
            Prim::Cyl => kit.cyl.clone(),
            Prim::Cube => kit.cube.clone(),
            Prim::Ball => kit.ball.clone(),
        };
        let mat = match p.mat {
            HelmetMat::Tint => look.hat.clone(),
            HelmetMat::Gunmetal => kit.gunmetal.clone(),
            HelmetMat::Steel => kit.steel.clone(),
            HelmetMat::Glow => kit.core_glow.clone(),
        };
        let mut t = Transform::from_xyz(p.pos.0, p.pos.1, p.pos.2)
            .with_scale(Vec3::new(p.scale.0, p.scale.1, p.scale.2));
        if p.pitch != 0.0 || p.roll != 0.0 {
            t.rotation = Quat::from_rotation_z(p.roll) * Quat::from_rotation_x(p.pitch);
        }
        commands
            .spawn((Mesh3d(mesh), MeshMaterial3d(mat), t))
            .set_parent(group);
    }
    group
}


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
    /// §2.5 HAND FOLLOW (k=120) and ELBOW POLE (k=60). The IK target is
    /// a hard snap - it is wherever the weapon's grip socket is THIS
    /// frame - so driving the arm straight from it makes a hand that
    /// teleports between poses. These carry the sprung position the arm
    /// actually reaches for, per side, so a hand SETTLES onto its grip
    /// and the elbow swings after it instead of with it.
    ///
    /// `f32::NAN` marks "no pose yet"; the first frame snaps rather than
    /// springing in from the origin, which would fling every arm across
    /// the body on spawn.
    hand_r: Vec3,
    hand_r_v: Vec3,
    hand_l: Vec3,
    hand_l_v: Vec3,
    pole_r_s: Vec3,
    pole_r_v: Vec3,
    pole_l_s: Vec3,
    pole_l_v: Vec3,
    /// §2.5 CLAVICLE (k=45) - the shoulder's own sprung offset.
    clav: Vec3,
    clav_v: Vec3,
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
    /// §B.1 #19-20: the forefeet, [left, right] - the toe-off snap.
    toes: [Entity; 2],
    /// §B.2: the twist segment between pelvis and thorax.
    lumbar: Entity,
    /// §B.1 #5-6: the shoulder girdle, [left, right].
    clavicles: [Entity; 2],
    weapon_root: Entity,
    /// held weapon models, indexed by `weapon_slot`
    weapons: [Entity; N_WEAPONS],
    /// the always-carried shield, shown raised on the left arm
    shield: Entity,
    armor_rig: Entity,
    /// D.1: mech leg armour roots, [left, right] x [thigh, shin, foot] -
    /// parented to the REAL leg bones so the plating walks with the gait.
    mech_leg_armor: [[Entity; 3]; 2],
    /// D.6 detach stages, driven by the sim's `mech_plates_dropped` bits:
    /// 70% = hip skirts + LEFT thigh plate; 40% = LEFT shin plate + rear
    /// drum + whip antenna; 15% = a foot cleat row (the visible limp).
    mech_detach_70: [Entity; 3],
    mech_detach_40: [Entity; 3],
    mech_detach_15: [Entity; 1],
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
/// §owner ATHLETIC MOTION: how far the acceleration term alone may lean
/// the torso, and the hard ceiling on the combined run+accel pitch.
///
/// 0.52 rad is ~30 degrees - the sprinter's drive angle the brief asks
/// for. The old ceiling was 0.185 (10.6 deg), which capped the whole
/// system well below the figure it was supposed to hit.
const ATHLETIC_LEAN_MAX: f32 = 0.40;
const ATHLETIC_LEAN_CAP: f32 = 0.52;
/// §owner ATHLETIC MOTION: peak knee flexion on the recovery leg, in
/// radians. 2.27 rad is ~130 deg - the sprinter's heel-to-seat fold the
/// brief asks for. It reaches that only at full `amp` (sprint speed);
/// a walk still bends about as little as it always did.
const KNEE_DRIVE_MAX: f32 = 2.27;
/// §owner ATHLETIC MOTION: shoulder swing amplitude at a sprint. The
/// old 0.5 gave +-17 deg with a weapon-free arm, and armed fighters
/// never saw it at all because both arms IK to the grip. Unarmed
/// sprinting now drives the arms properly.
const ARM_DRIVE_SWING: f32 = 1.15;

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
            (0.05 + amp * 0.09 + accel_lean).min(ATHLETIC_LEAN_CAP),
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

/// The same critically-damped spring over three axes. Two `damped_spring`
/// calls, not a new solver - the xy pair and the z axis are independent,
/// so this stays exactly the documented math rather than a second
/// implementation free to drift away from it.
fn damped_spring3(x: Vec3, v: Vec3, target: Vec3, k: f32, dt: f32) -> (Vec3, Vec3) {
    let (xy, vxy) = damped_spring(x.truncate(), v.truncate(), target.truncate(), k, dt);
    let (z, vz) = damped_spring(
        Vec2::new(x.z, 0.0),
        Vec2::new(v.z, 0.0),
        Vec2::new(target.z, 0.0),
        k,
        dt,
    );
    (xy.extend(z.x), vxy.extend(vz.x))
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
    /// §C.7: the two hull-mount viewmodels - shown instead of the
    /// (stowed) carried arsenal while piloting
    mech_turret: Entity,
    mech_pod: Entity,
}

/// Extra weapon greebles that only show while aiming - the ADS detail pass.
#[derive(Component)]
struct AdsDetail;

/// How drawn this bow model is, carried on the model ROOT.
///
/// Written by whichever system owns the wielder - the body rig for a
/// fighter, the viewmodel system for the player's own hands - and read by
/// `bow_string_sync`, which is the only place that knows how a string and
/// a nocked arrow are shaped. That split is deliberate: the two wielders
/// disagree about almost everything (whose clock, whose visibility, whose
/// space) and agree about exactly this one number.
#[derive(Component, Clone, Copy)]
struct BowDraw {
    /// `bow_draw_visual`'s 0..1.
    pull: f32,
    /// Whether an arrow is actually on the string. False through the
    /// reload gap after a shot, when the nock is empty.
    nocked: bool,
}

/// One half of a bowstring: tip to nock. `0` is the side, -1.0 or +1.0.
#[derive(Component, Clone, Copy)]
struct BowStringHalf(f32);

/// The arrow sitting on the string, as opposed to one in flight.
#[derive(Component)]
struct NockedArrow;

/// The illuminated dot inside a 1x optic. Carries its own rest position
/// because recoil FLOATS it about that point instead of kicking the
/// whole weapon - so the drift has to be applied as an offset from a
/// remembered origin, not accumulated onto wherever it currently is.
#[derive(Component)]
struct ReticleDot {
    rest: Vec3,
}

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
/// §owner: the missile pod's PRE-FIRE line - amber dots along the
/// dumb-fire path while Y is held, out to the first cover hit. The
/// locked shot steers itself; this is for the tap.
#[derive(Resource)]
struct RocketAimVis(Vec<Entity>);

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
/// §7.2/§8.2: the Forge TURNTABLE renders on its own layer, to a texture
/// the soldier page shows as a UI image. Layer isolation means the main
/// camera never sees the stage and the stage camera never sees the world.
const FORGE_PREVIEW_LAYER: usize = 6;
const FORGE_PREVIEW_PX: u32 = 512;
/// The stage lives far under the map - irrelevant to rendering (layers
/// isolate it) but keeps physics/audio/debug tooling from ever tripping
/// over it.
const FORGE_STAGE_POS: Vec3 = Vec3::new(300.0, -50.0, 300.0);

/// §7.2: the turntable's handles - the UI image, the rotating stand, one
/// pre-spawned weapon model per kind, and the two cosmetic materials the
/// sync system recolours in place.
#[derive(Resource)]
struct ForgePreview {
    image: Handle<Image>,
    stand: Entity,
    weapons: [Entity; N_WEAPONS],
    hat_mat: Handle<StandardMaterial>,
    tunic_mat: Handle<StandardMaterial>,
    /// the preview rig's pose handles - the sync system statically
    /// solves the same carry the live rig runs
    weapon_root: Entity,
    arm_l: [Entity; 3],
    arm_r: [Entity; 3],
    /// one silhouette group per class, indexed by `Class::ALL`
    class_rigs: [Entity; 4],
    /// §8.1: all five helmets mounted at once, indexed by
    /// `HELMET_CHOICES`. Same trick as `class_rigs` and `weapons` - one
    /// `Visibility` write swaps the head, where a rebuild would fight the
    /// render-layer tagging latch.
    helmets: [Entity; N_HELMETS],
}
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
const VM_KICK_SLIDE_M: f32 = 0.015 * VIEW_KICK_TRIM;
const VM_KICK_RETURN_S: f32 = 0.12;
// §1.4a screen-intrusion. There used to be four constants here - a
// declared RECEIVER box and a declared MAST box that every weapon was
// asserted to fit inside, with a comment naming the widest and tallest
// weapons in the arsenal.
//
// They are gone because they were a PROXY for the geometry, and the
// proxy was wrong. The comment claimed the tallest thing in the game was
// the AWM's scope at 0.085 up; the M249's arched carry handle is 0.192,
// and has been since it was raised clear of its own sight line. Nothing
// noticed, because the geometry lived inside a function that needed
// `Commands` and could not be read.
//
// `weapon_bounded_extent` measures the real thing now, per weapon, under
// that weapon's own profile. See `screen_profile` below.

// ---- the three SCREEN PROFILES (Brief VII §3.3, §4.3) --------------------
//
// Brief VII extends Brief VI's intrusion rule "with per-weapon allowance
// profiles (`spear_raised`, `bow_drawn`). The strict gun profile must keep
// passing." There was one profile. This is the other two.
//
// They are not slack granted to awkward weapons. Each says which PART of
// the weapon carries the constraint, because for a polearm and a bow the
// answer is not the receiver:
//
//   STRICT       guns. Receiver and hands stay right of the vertical
//                midline; nothing enters the central 12%-height circle.
//                Muzzle tip exempt, per Brief VIII §3.6.
//   SPEAR_RAISED the SHAFT may cross top-centre - that is the raised
//                javelin's whole silhouette - but the GRIP stays right of
//                the midline and the lower-centre reticle zone stays
//                clear. What is bounded moves from the receiver to the
//                hand.
//   BOW_DRAWN    the bow is SYMMETRIC about the sight line, so a midline
//                test is meaningless on it: it has a limb either side by
//                construction. The constraint that survives is VERTICAL -
//                the whole bow stays below the centre circle, so the
//                crosshair is never covered.
//
// That last profile deliberately supersedes the brief's own wording
// ("limbs left/below, string may approach center, grip never crosses the
// midline inward"). That sentence describes a VERTICAL bow, which is what
// Brief VII was written against; the war bow was turned horizontal since,
// and a horizontal bow held to the RIGHT of the midline is a bow aimed
// off the sight line. `vm_carry` already centres it deliberately. The
// rule is restated here rather than quietly dropped, because a profile
// that no longer matches the geometry it governs is worse than none.

/// Which intrusion profile a weapon is held to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ScreenProfile {
    Strict,
    SpearRaised,
    BowDrawn,
}

fn screen_profile(kind: GunKind) -> ScreenProfile {
    match kind {
        GunKind::Bow => ScreenProfile::BowDrawn,
        GunKind::Spear => ScreenProfile::SpearRaised,
        _ => ScreenProfile::Strict,
    }
}

/// How far either side of the hand a `SpearRaised` profile still counts
/// as "the grip".
///
/// The javelin's cord wraps sit at z -0.06, 0.0 and +0.06. The 1.85 m
/// shaft's CENTRE is out at z 0.35, because a javelin is held about a
/// third of the way along - so this window excludes the shaft, and that
/// exclusion IS the exemption the profile grants.
const GRIP_WINDOW_M: f32 = 0.15;

/// Does this profile bound this part, or exempt it?
fn profile_bounds_part(p: ScreenProfile, w: &WPart) -> bool {
    match p {
        // Everything. Brief VIII exempts the muzzle TIP, but the tip is
        // the narrowest thing on any gun and can never be the widest
        // part - so exempting it explicitly would buy nothing and leave
        // one more rule to get wrong. Bounding it is strictly safer.
        ScreenProfile::Strict => true,
        ScreenProfile::SpearRaised => w.pos.z.abs() <= GRIP_WINDOW_M,
        // Everything, but measured VERTICALLY - see below.
        ScreenProfile::BowDrawn => true,
    }
}

/// The bounded part's reach, MEASURED off the geometry: how far left of
/// the weapon's own root it extends, and how far up.
///
/// These were audited constants until `weapon_parts` existed to be read.
/// The difference is not cosmetic - an audited budget is a claim about
/// geometry that nothing rechecks, so widening a model silently widened
/// the lie, and the number would stay right up until the moment it
/// mattered. Now the budget IS the geometry.
///
/// Conservative in every direction it can be: `WPart::half` bounds a
/// tilted box by its AABB and a cylinder by its box, and no part is
/// exempt that the profile does not name.
fn weapon_bounded_extent(kind: GunKind) -> (f32, f32) {
    weapon_bounded_corners(kind)
        .into_iter()
        .fold((0.0_f32, 0.0_f32), |(l, u), (pl, pu)| (l.max(pl), u.max(pu)))
}

/// Every bounded part's own upper-left corner, one entry per part.
///
/// PER PART, and that distinction is load-bearing. The leftmost thing on
/// a rifle (the magazine, low and wide) and the tallest thing (the scope,
/// high and narrow) are different objects, so a single corner built from
/// the widest x and the tallest y describes a point the weapon does not
/// occupy. Testing the circle against that invented corner fails weapons
/// that are perfectly clear - which is exactly why the retired envelope
/// carried a separate RECEIVER box and MAST box instead of one.
///
/// Taking the real corners is both more accurate AND stricter: it checks
/// eleven or twenty of them instead of two, and every one is somewhere
/// the weapon actually is.
fn weapon_bounded_corners(kind: GunKind) -> Vec<(f32, f32)> {
    let prof = screen_profile(kind);
    weapon_parts(kind)
        .into_iter()
        .filter(|w| profile_bounds_part(prof, w))
        .map(|w| {
            let h = w.half();
            (h.x - w.pos.x, w.pos.y + h.y)
        })
        .collect()
}

// The sustained pose shifts, named. Each was an anonymous `Vec3::new(..)`
// inline in `fp_viewmodel`, which meant the intrusion sweep could not see
// them: it swept the base carry and nothing else, so every pose that
// actually moves the weapon toward the midline was unbounded. The bow's
// is the one that matters - it pulls 7.5 cm LEFT at full draw.
/// Full draw brings the bow up and in toward the aiming eye.
const VM_BOW_DRAW_SHIFT: Vec3 = Vec3::new(-0.075, 0.030, 0.050);
/// The inspect turn.
const VM_INSPECT_SHIFT: Vec3 = Vec3::new(-0.06, -0.02, 0.06);
/// The grenade coil, deepening with the charge.
const VM_GRENADE_SHIFT: Vec3 = Vec3::new(0.03, 0.035, -0.05);
/// Peak amplitude of the suppression shake (§ `Fighter::suppress_t`).
const VM_SUPPRESS_SHAKE: Vec3 = Vec3::new(0.011, 0.008, 0.0);

// ---- §3.4 low-ready / obstruction + ready-up ---------------------------
// BRIEF VIII §3.4, verbatim: "approaching a wall within 0.6m rotates the
// muzzle up-and-in 22° (rotation only) so the barrel never visually
// enters geometry", and "Ready-up on stop: returns over 0.15s with one
// small overshoot (ζ ≈ 0.7)".
//
// ROTATION ONLY, per C5 and the brief's own parenthesis - this must never
// touch `carry_offset`, or the §1.4a screen-intrusion sweep above stops
// bounding what the player actually sees.

/// How close a wall has to be before the weapon comes up.
const LOWREADY_RANGE_M: f32 = 0.6;
/// The dip itself: 22°, muzzle UP (the brief says up-and-in, not down -
/// pointing it at the floor trades a wall clip for a floor clip).
const LOWREADY_PITCH: f32 = 22.0 * PI / 180.0;
/// The "and-in" half. 8° inward, matching the sprint carry's own inward
/// rotation, so the two stances rotate about the same body line.
const LOWREADY_YAW: f32 = 8.0 * PI / 180.0;
/// §3.4: ζ ≈ 0.7. Under-damped ON PURPOSE - ζ<1 is the only thing that
/// produces the "one small overshoot" the brief asks for. No lerp, at
/// any rate, can overshoot at all.
const READY_UP_ZETA: f32 = 0.7;
/// ω_n from the standard 2% settling-time rule t_s ≈ 4/(ζ·ω_n), solved
/// at the brief's 0.15 s. Derived, never hand-typed, so the constant
/// cannot drift away from the spec number it came from.
const READY_UP_OMEGA: f32 = 4.0 / (READY_UP_ZETA * 0.15);

/// §3.4: is the muzzle about to enter geometry? One ray from the eye
/// along the look direction, capped at the obstruction range. Uses the
/// sim's own `raycast_cover` - the same query the camera's wall-hug
/// mirror and the cover system use, never a second approximation.
fn muzzle_blocked(sim: &TdmSim, eye: [f32; 3], fwd: [f32; 3]) -> bool {
    sim.raycast_cover(eye, fwd, LOWREADY_RANGE_M).is_some()
}

/// One step of the §3.4 ready-up spring, in place.
///
/// Sub-stepped because ω is ~38 rad/s: a single explicit step goes
/// unstable around 20 fps, and an unstable spring here does not degrade
/// gracefully - it throws the weapon off screen. Splitting the step
/// keeps a slow frame merely inaccurate instead of catastrophic.
fn ready_up_step(x: &mut f32, v: &mut f32, target: f32, dt: f32) {
    let steps = ((dt * READY_UP_OMEGA / 0.25).ceil() as usize).clamp(1, 8);
    let h = dt / steps as f32;
    for _ in 0..steps {
        let a = -2.0 * READY_UP_ZETA * READY_UP_OMEGA * *v
            - READY_UP_OMEGA * READY_UP_OMEGA * (*x - target);
        *v += a * h;
        *x += *v * h;
    }
}

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

/// §2.1 (Brief IV) CS:GO placement: right +0.11, down -0.13, forward
/// 0.32, yawed ~1.5 deg so the muzzle converges on screen center; long
/// guns exit the frame bottom-right through the stock. PURE, because
/// `fp_viewmodel`'s sight-alignment shift derives from the same numbers -
/// two copies of this table would drift and the sights would land
/// off-eye.
fn vm_carry(wk: GunKind) -> (Vec3, f32) {
    match wk {
        // §owner: the bow went HORIZONTAL, so its bounding shape turned
        // ninety degrees - it is now wide where it used to be tall. At
        // the old 0.36 m carry a 0.7 m span filled the bottom third of
        // the screen. Pushed out and centred, the way a bow is actually
        // held, and it now leaves the sight line clear rather than
        // putting a limb through it.
        // §owner: and it carries LOW. The riser's grip swells top out
        // 0.10 above the root - the tallest thing on any weapon relative
        // to its own carry - so at -0.17 the bow's top edge sat inside
        // the central 12%-of-screen-height circle as soon as any pose
        // raised it. Nothing caught that while the intrusion sweep was
        // checking a declared envelope at a generic carry instead of this
        // weapon's real geometry at its own.
        GunKind::Bow => (Vec3::new(-0.02, -0.22, -0.66), 0.0),
        GunKind::Spear => (Vec3::new(0.15, -0.10, -0.28), -0.12),
        GunKind::Glock | GunKind::Deagle => (Vec3::new(0.10, -0.125, -0.30), 0.0),
        GunKind::M249 => (Vec3::new(0.13, -0.14, -0.42), 0.0),
        _ => (Vec3::new(0.11, -0.13, -0.32), 0.0),
    }
}

/// §owner: the SIGHT LINE - the weapon-local height of the rear notch /
/// front post pair, for guns that have one. Focus brings THIS line to
/// the eye, so the drawn sights and the crosshair agree. `None` = no
/// iron sights to align (fists, the scoped AWM whose viewmodel hides,
/// the draw-pose bow and spear, the hip-fired minigun) - those keep the
/// generic focus shift.
fn sight_line_y(wk: GunKind) -> Option<f32> {
    match wk {
        GunKind::Glock => Some(0.1075),
        GunKind::Deagle => Some(0.1300),
        GunKind::Mp5 => Some(0.1160),
        GunKind::Shotgun => Some(0.0950),
        GunKind::Ak47 => Some(0.1060),
        GunKind::M4 => Some(0.1120),
        GunKind::M249 => Some(0.1265),
        GunKind::Minigun => Some(0.1120),
        _ => None,
    }
}

/// §1.1 Rule 2 (Brief VI): while a scoped-class weapon is zoomed the
/// viewmodel is NOT RENDERED at all - pure predicate, shared by the
/// render path and the scope-hide test.
fn vm_hidden_while_scoped(gun_is_scoped: bool, ads: bool) -> bool {
    gun_is_scoped && ads
}

/// The first-person viewmodel renders only while the HUD does (no menu
/// up) AND in first person, alive, not mid-roll. The vm camera draws
/// AFTER MainCam (order 1, no clear), so a visible gun composites over
/// the Paused plate - the menu gate is load-bearing, not cosmetic.
/// Pure, shared by `fp_viewmodel` and the menu-hide test.
fn vm_rendered(state: &GameState, person_t: f32, alive: bool, roll_t: f32) -> bool {
    hud_visible(state) && person_t < 0.5 && alive && roll_t <= 0.0
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

/// §3.5/§4.5: killfeed modifier glyphs.
///
/// Implemented: headshot (`*`), noscope (`o`), blind (`?`). Each is read
/// from state the sim ALREADY had at the kill site - none of the three
/// needed a new system, only a new question.
///
/// Still deferred, and named rather than quietly dropped: WALLBANG needs
/// the hitscan path to report whether it crossed cover geometry, and
/// THROUGH-SMOKE needs the same ray tested against live smoke volumes.
/// Both are real plumbing through the projectile path, not a flag.
///
/// §0 (Brief VII): ASCII only - the bundled font has no glyph for U+271B.
fn feed_glyphs(
    headshot: bool,
    noscope: bool,
    blind: bool,
    smoke: bool,
    wallbang: bool,
) -> String {
    let mut s = String::new();
    if headshot {
        s.push('*');
    }
    if noscope {
        s.push('o');
    }
    if blind {
        s.push('?');
    }
    if smoke {
        s.push('~');
    }
    if wallbang {
        s.push('#');
    }
    s
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

/// The 2D camera the entire interface renders through. It exists purely
/// to be LAST in the order so no 3D pass can draw over the HUD - see the
/// note at its spawn.
#[derive(Component)]
struct UiCam;

#[derive(Resource, Default)]
struct TracerPool(Vec<Entity>);

/// One pooled projectile: a root that carries BOTH models, so a slot
/// can serve an arrow this life and a spear the next without respawning
/// geometry. `spin` is the arrow's fletching group.
struct MissileSlot {
    root: Entity,
    arrow: Entity,
    spear: Entity,
    spin: Entity,
}

#[derive(Resource, Default)]
struct MissilePool(Vec<MissileSlot>);

#[derive(Resource, Default)]
struct DecalPool(Vec<Entity>);

#[derive(Resource)]
struct FxAssets {
    tracer_mesh: Handle<Mesh>,
    /// Tracers are side-relative like every other team signal: a streak
    /// coming at you must read hostile from the first frame, whichever
    /// team the player drew.
    tracer_ally: Handle<StandardMaterial>,
    tracer_enemy: Handle<StandardMaterial>,
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

// ---- §owner TEXTURE PIPELINE ---------------------------------------------
//
// Every surface in this world was a flat colour. That is the single
// biggest thing holding the look back: shape and lighting were doing all
// the work, so a wall, a crate and the ground differed only in hue.
//
// The textures are GENERATED, not loaded. There is no texture art in
// this project, and inventing a dependency on files that do not exist
// would block the whole feature on an art pass. Procedural noise costs
// nothing at build time, ships inside the binary, and - because every
// generator here is a pure function of a seed - is bit-identical on
// every machine, which matters for a capture harness that compares
// frames.
//
// All of it is COSMETIC. No texture is ever read by `sim`.

/// Side of every generated texture. Small on purpose: these tile, and a
/// 128px tile that repeats convincingly beats a 1024px one that costs
/// memory to say the same thing at this art style's fidelity.
const TEX_SIZE: u32 = 128;

/// A deterministic value-noise hash in 0..1, seeded per texture.
///
/// Deliberately its own tiny hash rather than the sim's `Pcg32`: this
/// runs at startup on the render side and must never touch the
/// simulation's RNG stream, or generating a texture would move where
/// bullets go.
fn tex_hash(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = x
        .wrapping_mul(0x27d4_eb2d)
        ^ y.wrapping_mul(0x1656_67b1)
        ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    (h & 0xffff) as f32 / 65535.0
}

/// Smoothed noise at a given cell size, wrapping at the tile edge so the
/// texture repeats seamlessly - a tile with a visible seam is worse than
/// no texture at all.
fn tex_noise(x: u32, y: u32, cells: u32, seed: u32) -> f32 {
    let cells = cells.max(1);
    let step = (TEX_SIZE / cells).max(1);
    let (gx, gy) = (x / step, y / step);
    let (fx, fy) = (
        (x % step) as f32 / step as f32,
        (y % step) as f32 / step as f32,
    );
    // smoothstep, for a soft interpolation rather than a linear ramp
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let w = |a: u32, b: u32| tex_hash(a % cells, b % cells, seed);
    let (a, b) = (w(gx, gy), w(gx + 1, gy));
    let (c, d) = (w(gx, gy + 1), w(gx + 1, gy + 1));
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Several octaves of the above - the standard trick that turns flat
/// noise into something that reads as a material.
fn tex_fbm(x: u32, y: u32, seed: u32) -> f32 {
    let mut v = 0.0;
    let mut amp = 0.5;
    let mut cells = 4;
    for o in 0..4 {
        v += tex_noise(x, y, cells, seed.wrapping_add(o * 977)) * amp;
        amp *= 0.5;
        cells *= 2;
    }
    v.clamp(0.0, 1.0)
}

/// Build a tiling texture from a per-pixel closure returning a
/// brightness multiplier in 0..1, where 1.0 means "leave this texel
/// alone".
///
/// The base colour stays on the MATERIAL, so one generator serves every
/// tint: the texture supplies structure, the material supplies colour.
/// That is why these are all grey.
///
/// The generators therefore run DARK-SIDE ONLY - they shade downward
/// from white and never above it. The first cut centred them on 1.0 and
/// encoded that as mid-grey, which silently halved the brightness of
/// every surface it touched and washed the variation out to nothing.
fn make_tex(images: &mut Assets<Image>, f: impl Fn(u32, u32) -> f32) -> Handle<Image> {
    use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let mut data = Vec::with_capacity((TEX_SIZE * TEX_SIZE * 4) as usize);
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let v = (f(x, y).clamp(0.0, 1.0) * 255.0) as u8;
            data.extend_from_slice(&[v, v, v, 255]);
        }
    }
    let mut img = Image::new(
        Extent3d {
            width: TEX_SIZE,
            height: TEX_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        // Unorm, NOT Srgb: this is a multiplier against base_color, and
        // running it through an sRGB decode would darken every surface
        // it touches rather than modulating it.
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    // These tile across large surfaces. Without Repeat the sampler
    // clamps and a single edge texel smears across a whole wall.
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    images.add(img)
}

/// Build a NORMAL MAP from the same height field that drives an albedo
/// texture, by central differences (a Sobel-lite).
///
/// This is what turns colour variation into apparent DEPTH: mortar
/// courses catch a shadow on one side, plank seams read as grooves, and
/// the brushed metal picks up a direction. The albedo alone could only
/// ever make a surface look painted.
///
/// Two things are load-bearing and easy to get wrong:
/// - the format MUST stay linear (`Rgba8Unorm`). A normal map decoded as
///   sRGB is a wrong vector at every texel, which reads as blotchy
///   lighting rather than as an obvious error.
/// - the mesh MUST carry tangents, or Bevy silently declines to apply
///   this at all. See `with_tangents` at the mesh sites.
fn make_normal_tex(
    images: &mut Assets<Image>,
    strength: f32,
    height: impl Fn(u32, u32) -> f32,
) -> Handle<Image> {
    use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let w = |x: i64, y: i64| -> f32 {
        // wrap, so the normal map tiles as seamlessly as its albedo
        let xi = x.rem_euclid(TEX_SIZE as i64) as u32;
        let yi = y.rem_euclid(TEX_SIZE as i64) as u32;
        height(xi, yi)
    };
    let mut data = Vec::with_capacity((TEX_SIZE * TEX_SIZE * 4) as usize);
    for y in 0..TEX_SIZE as i64 {
        for x in 0..TEX_SIZE as i64 {
            let dx = (w(x + 1, y) - w(x - 1, y)) * strength;
            let dy = (w(x, y + 1) - w(x, y - 1)) * strength;
            // the surface normal of a heightfield: (-dh/dx, -dh/dy, 1)
            let (nx, ny, nz) = (-dx, -dy, 1.0_f32);
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-6);
            let enc = |v: f32| (((v / len) * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            data.extend_from_slice(&[enc(nx), enc(ny), enc(nz), 255]);
        }
    }
    let mut img = Image::new(
        Extent3d {
            width: TEX_SIZE,
            height: TEX_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    images.add(img)
}

/// The generated surface set. Structure only - every one of these is
/// grey and takes its colour from whatever material mounts it.
#[derive(Resource, Clone)]
struct TextureKit {
    /// wind-rippled dirt with scattered grit
    ground: Handle<Image>,
    /// coarse blockwork with offset courses and mortar lines
    stone: Handle<Image>,
    /// straight grain with occasional knots
    wood: Handle<Image>,
    /// fine brushed grain with a few deeper scratches
    metal: Handle<Image>,
    /// Matching normal maps, generated from the SAME height functions -
    /// so the bumps line up with the colour rather than being a second
    /// pattern laid over the first.
    ground_n: Handle<Image>,
    stone_n: Handle<Image>,
    wood_n: Handle<Image>,
    metal_n: Handle<Image>,
}

/// The four height fields, named once and shared.
///
/// Albedo and normal are generated from the SAME function per surface -
/// that is the whole reason these are hoisted out rather than written
/// inline at each `make_tex` call. Two copies would drift, and a normal
/// map whose bumps do not line up with its own colour looks worse than
/// no normal map at all.
fn height_ground(x: u32, y: u32) -> f32 {
    let broad = tex_fbm(x, y, 11);
    let grit = tex_hash(x, y, 733);
    // fbm clusters hard around its own mean, so raw it produced a band
    // far too narrow to see once lit - a checkerboard diagnostic proved
    // the texture was binding and tiling correctly all along and only
    // the CONTRAST was wrong. Pushed out from the midpoint before use.
    let broad = ((broad - 0.5) * 2.4 + 0.5).clamp(0.0, 1.0);
    // patches of packed dirt against looser sand, at a scale bigger than
    // one tile's noise so the ground does not read as uniform fuzz
    let patch = if tex_noise(x, y, 3, 401) > 0.58 { 0.86 } else { 1.0 };
    ((0.55 + broad * 0.45) * patch + grit * 0.06).min(1.0)
}

/// Mortar courses every quarter tile, offset row to row so it reads as
/// laid blockwork rather than a grid, with noise over the faces so no
/// two blocks match.
fn height_stone(x: u32, y: u32) -> f32 {
    let block = TEX_SIZE / 4;
    let row = y / block;
    let shift = if row % 2 == 0 { 0 } else { block / 2 };
    let bx = (x + shift) % block;
    let by = y % block;
    let mortar = bx < 3 || by < 3;
    let body = 0.74 + tex_fbm(x, y, 29) * 0.26;
    if mortar {
        body * 0.55 // the joint reads as a real recess
    } else {
        body
    }
}

/// Grain along one axis, warped by noise so it is grain and not stripes,
/// with the odd darker knot.
fn height_wood(x: u32, y: u32) -> f32 {
    let n = TEX_SIZE as f32;
    let warp = tex_fbm(x, y, 53) * 12.0;
    let rings = (((y as f32 + warp) / n * 22.0).sin() * 0.5 + 0.5).powf(1.6);
    let knot = if tex_noise(x, y, 6, 91) > 0.86 { 0.62 } else { 1.0 };
    ((0.66 + rings * 0.34) * knot).min(1.0)
}

/// Fine horizontal brushing, plus a few deeper scratches.
fn height_metal(x: u32, y: u32) -> f32 {
    let brush = tex_hash(x / 2, y, 137) * 0.12;
    let scratch = if tex_noise(x, y, 3, 211) > 0.90 { 0.80 } else { 1.0 };
    ((0.84 + brush) * scratch).min(1.0)
}

fn build_texture_kit(images: &mut Assets<Image>) -> TextureKit {
    // Normal strengths are per-surface on purpose: mortar joints are a
    // real recess and want to read hard, brushed metal is microscopic
    // and wants to read as sheen rather than as corrugation.
    TextureKit {
        ground: make_tex(images, height_ground),
        stone: make_tex(images, height_stone),
        wood: make_tex(images, height_wood),
        metal: make_tex(images, height_metal),
        ground_n: make_normal_tex(images, 2.4, height_ground),
        stone_n: make_normal_tex(images, 6.0, height_stone),
        wood_n: make_normal_tex(images, 3.0, height_wood),
        metal_n: make_normal_tex(images, 1.4, height_metal),
    }
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
    /// FP-only translucent shield set - the raised plate must not blind
    /// the player. Third-person shields keep the opaque materials above.
    /// §owner MECH BARRIER: the near-invisible field body, and the bright
    /// cell edges drawn through it. See where they are built.
    barrier_fill: Handle<StandardMaterial>,
    barrier_edge: Handle<StandardMaterial>,
    vm_shield_dark: Handle<StandardMaterial>,
    vm_shield_steel: Handle<StandardMaterial>,
    vm_shield_gold: Handle<StandardMaterial>,
    /// Faded unit-stencil paint for the mech ident plates - decorative
    /// parchment, deliberately dimmer than the ally signal white.
    mech_stencil: Handle<StandardMaterial>,
    /// The 1x optic reticle - unlit so it holds its glow in shadow, the
    /// way a real illuminated dot does.
    optic_red: Handle<StandardMaterial>,
}

/// §2.1 tone slots of the weapon palette.
#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Light,
    Mid,
    Dark,
    Black,
    /// The red-dot RETICLE - unlit emissive red, the only non-grey in
    /// the weapon palette. Deliberately its own slot rather than a
    /// reuse of `mech_red`: that one is a gameplay PROMISE (the visor
    /// weak point carries MECH_VISOR_MULT), and a reticle that shared
    /// its handle would start reading as a hit marker.
    Reticle,
}

impl ModelKit {
    fn tone(&self, t: Tone) -> Handle<StandardMaterial> {
        match t {
            Tone::Light => self.grey_light.clone(),
            Tone::Mid => self.grey_mid.clone(),
            Tone::Dark => self.grey_dark.clone(),
            Tone::Black => self.grey_black.clone(),
            Tone::Reticle => self.optic_red.clone(),
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
    /// true → hangs off the SPINNER child instead of the model root.
    ///
    /// Only the minigun's barrel cluster. A flag on the PART rather than a
    /// hand-written spawn chain, because that chain was the single thing
    /// keeping this whole vocabulary trapped inside a function that needs
    /// `Commands` - see `weapon_parts`.
    spin: bool,
}

impl WPart {
    /// Half-extents in the weapon's own frame, after `tilt`.
    ///
    /// `tilt` rotates about X, so the X half-extent is untouched and Y and
    /// Z mix. This is the AABB of the rotated box - a BOUND, not the exact
    /// silhouette, and conservative on purpose: a budget that
    /// over-estimates a part fails safe, one that under-estimates it lets
    /// geometry through.
    ///
    /// A cylinder is bounded by its own box for the same reason. Bevy's
    /// cylinder is Y-aligned before `tilt`, so that box is already the
    /// right shape to contain it.
    fn half(&self) -> Vec3 {
        let (s, c) = (self.tilt.sin().abs(), self.tilt.cos().abs());
        let h = self.size.abs() * 0.5;
        Vec3::new(h.x, h.y * c + h.z * s, h.y * s + h.z * c)
    }
}

fn wp(cyl: bool, tone: Tone, pos: (f32, f32, f32), tilt: f32, size: (f32, f32, f32)) -> WPart {
    WPart {
        cyl,
        tone,
        pos: Vec3::new(pos.0, pos.1, pos.2),
        tilt,
        size: Vec3::new(size.0, size.1, size.2),
        detail: false,
        spin: false,
    }
}

fn wd(cyl: bool, tone: Tone, pos: (f32, f32, f32), tilt: f32, size: (f32, f32, f32)) -> WPart {
    WPart {
        detail: true,
        ..wp(cyl, tone, pos, tilt, size)
    }
}

/// A part on the SPINNER - the minigun's barrel cluster, and nothing else.
fn ws(cyl: bool, tone: Tone, pos: (f32, f32, f32), tilt: f32, size: (f32, f32, f32)) -> WPart {
    WPart {
        spin: true,
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

/// §owner: where a FIRST-PERSON shot should visually leave from, in
/// camera-local space.
///
/// The sim casts every ray from the EYE - that is the hit test and it
/// must not move. But drawing the streak from there too puts the muzzle
/// flash in the middle of the screen, which reads as the player firing
/// out of their own face. It is most obvious in a mech, where the
/// mounts hang well off to one side of a very tall eye point.
///
/// These are the muzzle tips of the viewmodels as actually placed: the
/// mount carries in `setup`, plus the barrel/tube length from their
/// builders, times the 0.62/0.72 model scales. Kept next to `vm_carry`
/// so the two stay in view of each other.
fn fp_muzzle_local(p: &Fighter) -> Vec3 {
    if p.in_mech() {
        return match p.mech_weapon {
            // the launch tube: carry (0.19,-0.20,-0.46), tube runs to
            // local z 0.90 at scale 0.72
            sim::MechWeapon::Rockets => Vec3::new(0.19, -0.20, -0.46 - 0.90 * 0.72),
            // the gatling cluster: carry (0.20,-0.22,-0.52), barrels
            // reach local z ~0.66 at scale 0.62
            _ => Vec3::new(0.20, -0.22, -0.52 - 0.66 * 0.62),
        };
    }
    // infantry: the carried gun's own offset, run forward to about the
    // end of a typical barrel at the shared 0.9 model scale
    let (tr, _) = vm_carry(p.gun);
    Vec3::new(tr.x, tr.y, tr.z - 0.60 * 0.9)
}

/// Where the arrow's NOCK - the very back of the tail - sits in the arrow
/// model's own local space, per unit of `scale.z`.
///
/// Named because the BOW has to put this exact point on the string, and a
/// nocked arrow whose tail floats off the string, or buries through it, is
/// the first thing the eye catches on a drawn bow. It is the nock block's
/// centre (-0.345) less half its 0.03 length; `spawn_arrow_model` asserts
/// the two still agree.
const ARROW_NOCK_Z: f32 = -0.36;

/// §owner: a real ARROW in flight - forged head, tapered shaft, three
/// fletching vanes. It replaced a featureless 5 cm box, which at the
/// speeds this game looses arrows at read as a grey dash and told the
/// player nothing about which way it was going or where it would bite.
///
/// Built nose-forward along +Z in a UNIT-LENGTH envelope, so the
/// caller's `scale.z` sets the real length and the proportions hold.
/// The `spin` child carries the fletching alone - vanes rotate about
/// the shaft, a shaft that rolled as a whole would look like a drill.
fn spawn_arrow_model(commands: &mut Commands, kit: &ModelKit) -> (Entity, Entity) {
    debug_assert!(
        (ARROW_NOCK_Z - (-0.345 - 0.03 * 0.5)).abs() < 1e-6,
        "ARROW_NOCK_Z must track the nock block below"
    );
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // shaft, then the bodkin head: a tapered point, not a blunt end
    for (mat, z, sc) in [
        (kit.wood.clone(), 0.02, Vec3::new(0.020, 0.020, 0.72)),
        (kit.steel.clone(), 0.40, Vec3::new(0.030, 0.030, 0.10)),
        (kit.grey_black.clone(), 0.465, Vec3::new(0.012, 0.012, 0.05)),
        // the nock, so the tail reads as an arrow and not a cut stick
        (kit.grey_black.clone(), -0.345, Vec3::new(0.024, 0.024, 0.03)),
    ] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(mat),
                Transform::from_xyz(0.0, 0.0, z).with_scale(sc),
            ))
            .set_parent(root);
    }
    // three fletching vanes on their own spinner
    let spin = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .set_parent(root)
        .id();
    for i in 0..3 {
        let a = i as f32 * std::f32::consts::TAU / 3.0;
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.white.clone()),
                Transform {
                    translation: Vec3::new(a.cos() * 0.026, a.sin() * 0.026, -0.28),
                    rotation: Quat::from_rotation_z(a),
                    scale: Vec3::new(0.006, 0.048, 0.13),
                },
            ))
            .set_parent(spin);
    }
    (root, spin)
}

/// §owner: a real SPEAR - leaf blade, collar, shaft, butt spike. Same
/// unit-length envelope as the arrow.
fn spawn_spear_model(commands: &mut Commands, kit: &ModelKit) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    for (cyl, mat, z, sc) in [
        (true, kit.wood.clone(), -0.02, Vec3::new(0.030, 0.86, 0.030)),
        // leaf blade: wide, flat, and long enough to read at range
        (false, kit.steel.clone(), 0.435, Vec3::new(0.055, 0.016, 0.20)),
        (false, kit.grey_light.clone(), 0.520, Vec3::new(0.022, 0.014, 0.06)),
        // collar where blade meets shaft, and the butt spike
        (true, kit.grey_black.clone(), 0.330, Vec3::new(0.044, 0.05, 0.044)),
        (true, kit.grey_black.clone(), -0.455, Vec3::new(0.034, 0.07, 0.034)),
    ] {
        let mesh = if cyl { kit.cyl.clone() } else { kit.cube.clone() };
        let rot = if cyl {
            Quat::from_rotation_x(FRAC_PI_2)
        } else {
            Quat::IDENTITY
        };
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform {
                    translation: Vec3::new(0.0, 0.0, z),
                    rotation: rot,
                    scale: sc,
                },
            ))
            .set_parent(root);
    }
    root
}

/// How long a freshly launched rocket is drawn easing off the muzzle
/// and onto its true position. Short enough that the correction is over
/// long before the bird is anywhere near a target.
const ROCKET_LAUNCH_BLEND_S: f32 = 0.09;

/// Fletching roll rate. Fast enough to read as spin-stabilised at the
/// speeds arrows fly here, slow enough not to strobe against the frame
/// rate. Cosmetic: the sim knows nothing about it.
const ARROW_SPIN_RAD_S: f32 = 24.0;

/// Inner half-width of a red-dot window. At the ADS distance every
/// firearm settles to, this subtends roughly a ninth of screen height -
/// a real optic's proportion, big enough to aim through and small
/// enough to leave peripheral vision intact.
const OPTIC_HALF: f32 = 0.023;
/// Frame bar thickness, and the housing's fore-aft depth.
const OPTIC_FRAME: f32 = 0.005;
const OPTIC_DEPTH: f32 = 0.032;
/// Side of the emissive dot. Small on purpose: the aiming mark must not
/// cover the thing being aimed at, which is exactly how the old cross
/// failed.
const OPTIC_DOT_M: f32 = 0.0062;
/// §owner: how far the dot floats inside its window at full recoil.
///
/// This is the whole trick behind "the gun stays still and the DOT
/// moves". Bounded well inside `OPTIC_HALF` so the dot can never leave
/// the glass - a reticle that slid behind the housing would read as a
/// rendering bug, not as recoil.
const RETICLE_DRIFT_M: f32 = 0.0075;

/// §owner: the 1x RED DOT every firearm carries.
///
/// An open square housing you look THROUGH - four bars around a window,
/// never a solid block - with a thin emissive red CROSS floating at the
/// exact centre. This replaced the goalpost irons, which asked the
/// player to line up two pieces of dark grey against a dark grey world.
///
/// `y` is the reticle centre and MUST equal the gun's `sight_line_y`:
/// focus aligns that height to the eye, so any disagreement puts the
/// cross off the crosshair and the optic becomes decoration. The pairing
/// is asserted for every gun in `every_firearm_carries_an_aligned_optic`.
///
/// The cross is 1x - it moves the AIM POINT nowhere. It is a sight
/// picture, not a zoom: magnification stays whatever `zoom_deg` says.
fn push_red_dot(parts: &mut Vec<WPart>, y: f32, z: f32, mount_top: f32) {
    let outer = OPTIC_HALF + OPTIC_FRAME * 0.5;
    let span = OPTIC_HALF * 2.0 + OPTIC_FRAME * 2.0;
    debug_assert!(
        y - outer >= mount_top - 1e-4,
        "the optic's lower frame is inside the gun: window bottom {} vs          receiver top {mount_top}",
        y - outer
    );
    // housing: left / right posts, then the top and bottom bars
    parts.push(wp(false, Tone::Black, (-outer, y, z), 0.0, (OPTIC_FRAME, span, OPTIC_DEPTH)));
    parts.push(wp(false, Tone::Black, (outer, y, z), 0.0, (OPTIC_FRAME, span, OPTIC_DEPTH)));
    parts.push(wp(false, Tone::Black, (0.0, y + outer, z), 0.0, (span, OPTIC_FRAME, OPTIC_DEPTH)));
    parts.push(wp(false, Tone::Black, (0.0, y - outer, z), 0.0, (span, OPTIC_FRAME, OPTIC_DEPTH)));
    // the mount BRIDGES housing to receiver. Sized from the real gap so
    // the optic never floats and never sinks into the slide.
    let gap = (y - outer - mount_top).max(0.0);
    if gap > 1e-4 {
        parts.push(wp(
            false,
            Tone::Dark,
            (0.0, mount_top + gap * 0.5, z),
            0.0,
            (0.016, gap, OPTIC_DEPTH * 0.7),
        ));
    }
    // THE DOT - one small emissive square, dead centre.
    //
    // §owner: this was a CROSS of two long bars and it read badly - the
    // arms reached most of the way across the window, so the thing you
    // were supposed to aim WITH was also the thing covering what you
    // were aiming AT. A dot occludes almost nothing and is what a 1x
    // optic actually projects.
    //
    // It is spawned tagged rather than baked in with the housing,
    // because the dot MOVES: recoil floats it inside the window while
    // the gun body stays still (see `RETICLE_DRIFT_M` / `fp_viewmodel`).
    parts.push(wp(
        false,
        Tone::Reticle,
        (0.0, y, z),
        0.0,
        (OPTIC_DOT_M, OPTIC_DOT_M, 0.004),
    ));
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
    /// which HULL MOUNT was selected last frame - a mount swap is the
    /// in-chassis equivalent of a weapon swap for shot detection
    prev_mech_weapon: sim::MechWeapon,
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
            prev_mech_weapon: sim::MechWeapon::Gatling,
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

/// §4.1: one segment of the health bar, by index. Ten of these.
#[derive(Component)]
struct VitalsSeg(usize);

/// §4.1: one armour pip, by index.
#[derive(Component)]
struct ArmorPip(usize);

/// §4.1: the rows holding the above. Tagged so `hud_vitals_style = 1`
/// (numbers only) can hide the whole visual language in one query
/// instead of walking every segment.
#[derive(Component)]
struct VitalsBarRow;

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
/// §4.3: how many award toasts can stack.
const AWARD_SLOTS: usize = 4;
/// §4.3: "each fading after 2.5s".
const AWARD_TTL_S: f32 = 2.5;
/// Two kills inside this window read as a streak.
const AWARD_STREAK_S: f32 = 4.0;

#[derive(Component)]
struct AwardToast(usize);

/// The award detector's memory between frames.
#[derive(Default)]
struct AwardState {
    items: Vec<(String, f32)>,
    prev_kills: u32,
    prev_assists: u32,
    prev_parries: u32,
    prev_owner: [Option<Team>; 2],
    last_kill_t: f32,
    streak: u32,
    inited: bool,
}

/// §4.3: watch the sim's own stat deltas and push toasts. Client-side
/// and read-only - the sim is never consulted about presentation.
fn award_toasts(
    game: Res<Game>,
    time: Res<Time>,
    mut st: Local<AwardState>,
    mut q: Query<(&AwardToast, &mut Text, &mut TextColor)>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];
    let my_team = p.team;
    // first frame (and every rebuild): sync silently, no toast backlog
    if !st.inited {
        st.inited = true;
        st.prev_kills = p.kills;
        st.prev_assists = p.assists;
        st.prev_parries = p.parries;
        for (i, cp) in simr.checkpoints.iter().take(2).enumerate() {
            st.prev_owner[i] = cp.owner;
        }
    }
    // kills - with the streak names layered on top
    if p.kills > st.prev_kills {
        let now = simr.t;
        if now - st.last_kill_t <= AWARD_STREAK_S {
            st.streak += 1;
        } else {
            st.streak = 1;
        }
        st.last_kill_t = now;
        let label = match st.streak {
            1 => "KILL",
            2 => "DOUBLE KILL",
            3 => "TRIPLE KILL",
            _ => "RAMPAGE",
        };
        st.items.push((label.to_string(), AWARD_TTL_S));
        // the freshest kill-feed row says whether it was a headshot or
        // a wallbang - both worth their own line
        if let Some((ev, _)) = simr.kill_feed.last() {
            if ev.killer == simr.player {
                if ev.headshot {
                    st.items.push(("HEADSHOT".to_string(), AWARD_TTL_S));
                }
                if ev.wallbang {
                    st.items.push(("WALLBANG".to_string(), AWARD_TTL_S));
                }
            }
        }
    }
    st.prev_kills = p.kills;
    if p.assists > st.prev_assists {
        st.items.push(("ASSIST".to_string(), AWARD_TTL_S));
    }
    st.prev_assists = p.assists;
    if p.parries > st.prev_parries {
        st.items.push(("PARRY".to_string(), AWARD_TTL_S));
    }
    st.prev_parries = p.parries;
    // captures: a ring flipping TO my team
    for (i, cp) in simr.checkpoints.iter().take(2).enumerate() {
        if cp.owner != st.prev_owner[i] {
            if cp.owner == Some(my_team) {
                st.items.push(("POINT CAPTURED".to_string(), AWARD_TTL_S));
            }
            st.prev_owner[i] = cp.owner;
        }
    }
    // tick + trim to the visible stack
    let dt = time.delta_secs();
    for it in &mut st.items {
        it.1 -= dt;
    }
    st.items.retain(|it| it.1 > 0.0);
    while st.items.len() > AWARD_SLOTS {
        st.items.remove(0);
    }
    for (slot, mut text, mut color) in &mut q {
        match st.items.get(slot.0) {
            Some((label, ttl)) => {
                **text = label.clone();
                let g = branding::palette::GOLD.to_srgba();
                *color = TextColor(Color::srgba(
                    g.red,
                    g.green,
                    g.blue,
                    (ttl / 0.6).clamp(0.0, 1.0),
                ));
            }
            None => **text = String::new(),
        }
    }
}

/// Top inset for the minimap. Clears the K/D line that shares this
/// corner rather than overlapping it - 52 still collided with it at the
/// default window (the capture showed the text ON the map).
const MINIMAP_TOP_PX: f32 = 96.0;

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

/// §4.5: "Max 5 rows".
const KILLFEED_ROWS: usize = 5;
/// §4.5: "Local-player rows get a 2px #B50000 border".
const KILLFEED_BORDER_PX: f32 = 2.0;
/// §4.5, verbatim: #B50000 on rgba(0,0,0,0.5).
const KILLFEED_MINE_BORDER: Color = Color::srgb(0.710, 0.0, 0.0);
const KILLFEED_MINE_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

/// §4.5: one killfeed row. Carries the border and background.
#[derive(Component)]
struct KillfeedRow(usize);

/// §4.5: one span within a row - `(row, part)` where part is
/// 0 = killer (+assist), 1 = weapon/modifier glyphs, 2 = victim.
#[derive(Component)]
struct KillfeedCell(usize, usize);

#[derive(Component)]
struct HitFeedText;

#[derive(Component)]
struct BannerText;

/// §4.6: the zero-size node pinned to the exact screen centre that every
/// crosshair piece hangs off. It carries the kill-confirm rotation, so a
/// single `Transform` write spins the whole cross into an X.
#[derive(Component)]
struct CrosshairRoot;

/// §4.6: one drawn piece of the crosshair. `idx` 0..3 are the arms
/// (top/right/bottom/left) and 4 is the centre dot; `outline` marks the
/// dark backing rect that sits behind the fill.
#[derive(Component, Clone, Copy)]
struct CrosshairPiece {
    idx: u8,
    outline: bool,
}

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
    // §C.7 (Brief VIII): in a chassis the strip IS the two hull mounts -
    // the carried inventory is sealed away with the rest of the infantry
    // kit, exactly as the sim's slot keys already treat it.
    let in_mech = p.in_mech();
    let cur = if in_mech {
        match p.mech_weapon {
            sim::MechWeapon::Rockets => 1,
            _ => 0,
        }
    } else if p.shield_up {
        3 // raising the plate un-fades the strip and moves the accent
    } else {
        p.active
    };
    if cur != *last_active {
        *last_active = cur;
        *idle_t = 0.0;
    } else {
        *idle_t += time.delta_secs();
    }
    let strip_fade = if *idle_t > 4.0 { 0.45 } else { 1.0 };
    for (cell, mut t, mut tc) in &mut q {
        let (name, active) = if in_mech {
            match cell.0 {
                0 => (format!("TURRET {}", p.mech_rounds), cur == 0),
                1 => (format!("ROCKETS {}", p.pod_ammo), cur == 1),
                _ => {
                    **t = String::new();
                    continue;
                }
            }
        } else if cell.0 == 3 {
            // the shield is an ESSENTIAL slot: always listed, lit while
            // raised. MUST branch before the inventory index - the
            // carried array is only 3 wide.
            ("SHIELD".to_string(), p.shield_up)
        } else {
            let g = p.inventory[cell.0];
            let n = if g == GunKind::Fists {
                "-".to_string()
            } else {
                gun(g).name.to_string()
            };
            (n, cell.0 == p.active && !p.shield_up)
        };
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

/// §1.2: the grouping the registry's own doc comment has promised since
/// it was written ("ONE table owns every action's bind, display name,
/// and grouping") - the field never existed until now.
#[derive(Clone, Copy, PartialEq)]
enum BindGroup {
    Move,
    Fight,
    Gear,
    View,
}

impl BindGroup {
    const ALL: [BindGroup; 4] = [Self::Move, Self::Fight, Self::Gear, Self::View];
    fn title(self) -> &'static str {
        match self {
            Self::Move => "MOVEMENT",
            Self::Fight => "COMBAT",
            Self::Gear => "GEAR",
            Self::View => "VIEW AND INFO",
        }
    }
}

struct Bind {
    key: &'static str,
    action: &'static str,
    /// Shown on the one-time first-run card (the non-obvious binds).
    essential: bool,
    group: BindGroup,
}

const BIND_REGISTRY: &[Bind] = &[
    Bind { key: "W A S D", action: "Move", essential: false, group: BindGroup::Move },
    Bind { key: "SHIFT", action: "Sprint", essential: false, group: BindGroup::Move },
    Bind { key: "RMB + SHIFT", action: "Steady walk - slow, SILENT, steadier. Stays on when you release RMB", essential: true, group: BindGroup::Move },
    Bind { key: "SPACE", action: "Jump (crouch first to jump higher)", essential: true, group: BindGroup::Move },
    Bind { key: "CTRL", action: "Crouch", essential: false, group: BindGroup::Move },
    Bind { key: "Q", action: "Ground: dodge roll - Air + direction: FLIP (no firing)", essential: true, group: BindGroup::Move },
    Bind { key: "LMB", action: "Fire", essential: false, group: BindGroup::Fight },
    Bind { key: "RMB", action: "HOLD: focus/aim - scope zoom cycle (heavy rifle), draw (bow, spear)", essential: false, group: BindGroup::Fight },
    Bind { key: "F", action: "Knife - tap: slash, hold: lunge (backstab kills)", essential: true, group: BindGroup::Fight },
    Bind { key: "C (hold)", action: "Armor ability (brace / flame / repulsor)", essential: true, group: BindGroup::Fight },
    // §5 (owner): the grenade grammar changed this session and the
    // registry must say what the hands actually do now.
    Bind { key: "G", action: "Grenade to hand (again: cycle type) - RMB aims the arc, LMB throws", essential: true, group: BindGroup::Fight },
    Bind { key: "H / Mouse4", action: "Legacy: hold to cook, release to throw", essential: false, group: BindGroup::Fight },
    Bind { key: "B", action: "Stow the grenade / cancel an aimed throw (keeps it)", essential: false, group: BindGroup::Fight },
    Bind { key: "4", action: "Shield stance - essential slot (throwables only while up)", essential: true, group: BindGroup::Fight },
    Bind { key: "1 2 3", action: "Weapon slots", essential: false, group: BindGroup::Fight },
    Bind { key: "R", action: "Reload", essential: false, group: BindGroup::Fight },
    Bind { key: "U", action: "Dismount the mech (chassis is scrapped; the pad respawns)", essential: false, group: BindGroup::Gear },
    Bind { key: "Y (hold)", action: "Mech missile pod: hold to LOCK a mech (1.3s), release to fire - tap: dumb-fire. Never locks infantry", essential: false, group: BindGroup::Gear },
    Bind { key: "T", action: "Inspect weapon", essential: false, group: BindGroup::Gear },
    Bind { key: "MOUSE", action: "Look", essential: false, group: BindGroup::View },
    Bind { key: "V or O", action: "Camera: first <-> third person", essential: true, group: BindGroup::View },
    Bind { key: "Z / X", action: "Lean left / right", essential: false, group: BindGroup::View },
    Bind { key: "TAB", action: "Scoreboard", essential: false, group: BindGroup::View },
    Bind { key: "M", action: "Minimap on/off", essential: false, group: BindGroup::View },
    Bind { key: "F3", action: "Hit-zone debug rings", essential: false, group: BindGroup::View },
    Bind { key: "F4", action: "Rig joint markers (gap view)", essential: false, group: BindGroup::View },
    Bind { key: "ESC", action: "Menu", essential: false, group: BindGroup::View },
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
        ArmorSet::RobotSuit => "MECH BOARDED - 1/2: MOUNTS - C: REPULSOR - U: DISMOUNT - protect your REAR",
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

/// The same draw, seen from inside the archer's head.
///
/// The third-person script above cannot check the first-person bow: the
/// capture starts in third person (`CamCtl::first_person` defaults false)
/// and never presses V, so every bow frame ever captured has been of the
/// world model. The viewmodel bow went years without a nocked arrow partly
/// because nothing ever looked at it.
///
/// V first, then equip - toggling person AFTER the draw starts would
/// re-pose mid-pull and confuse what the frames mean.
const BOW_DRAW_FP_BEATS: &[CapBeat] = &[
    CapBeat { press: &[CapKey::K(KeyCode::KeyV)], ..beat(0.5) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyV)], ..beat(0.6) },
    CapBeat { press: &[CapKey::K(KeyCode::Digit3)], ..beat(0.8) },
    CapBeat { release: &[CapKey::K(KeyCode::Digit3)], ..beat(0.9) },
    CapBeat { snap: Some("01-fp-bow-idle"), ..beat(1.3) },
    CapBeat { press: &[CapKey::M(MouseButton::Left)], ..beat(1.4) },
    // ~0.2s of draw: nocked, string moving, nowhere near anchor
    CapBeat { snap: Some("02-fp-bow-quarter-draw"), ..beat(1.6) },
    // past BOW_DRAW_FULL_S (0.7s): at anchor, full power
    CapBeat { snap: Some("03-fp-bow-full-draw"), ..beat(2.2) },
    CapBeat {
        release: &[CapKey::M(MouseButton::Left)],
        snap: Some("04-fp-bow-release"),
        ..beat(2.3)
    },
    // the arrow is gone and the nock is empty until the auto-nock lands
    CapBeat { snap: Some("05-fp-bow-after-shot"), ..beat(2.7) },
    CapBeat { end: true, ..beat(3.1) },
];

/// The four surfaces that shipped with NO capture coverage, which is
/// exactly why each one broke in a way only a player could see: the
/// first-person hull mounts (the pilot used to hold his stowed rifle),
/// the translucent guard plate, and the viewmodel drawing OVER the pause
/// menu. The existing "menus" script cannot prove that last one - it
/// walks Intro straight to Paused, so the viewmodel was hidden by the
/// third-person default and the bug hid with it. This script reproduces
/// the real conditions: in a match, in first person, then ESC.
const MECH_FP_BEATS: &[CapBeat] = &[
    CapBeat { press: &[CapKey::K(KeyCode::KeyV)], ..beat(0.5) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyV)], ..beat(0.6) },
    // TURRET is the default mount - the barrel cluster, hull housing,
    // and hazard strip should sit in the lower-right frame.
    // (Snap no earlier than ~1.5 s: the first render frames land before
    // the swapchain settles and save a 0-byte PNG.)
    CapBeat { snap: Some("01-fp-mech-turret"), ..beat(1.6) },
    // FIRING: the streak has to leave the barrel cluster in the lower
    // right, not the middle of the screen. Held through two snaps so the
    // belt is definitely running by the second.
    CapBeat { press: &[CapKey::M(MouseButton::Left)], ..beat(1.66) },
    CapBeat { snap: Some("01b-fp-turret-firing"), ..beat(1.76) },
    CapBeat { release: &[CapKey::M(MouseButton::Left)], ..beat(1.78) },
    CapBeat { press: &[CapKey::K(KeyCode::Digit2)], ..beat(1.8) },
    CapBeat { release: &[CapKey::K(KeyCode::Digit2)], ..beat(1.9) },
    CapBeat { snap: Some("02-fp-mech-rockets"), ..beat(2.4) },
    // RMB on the pod is PRE-AIM ONLY: the amber arc appears and the FOV
    // must not move a degree. Compare 03 against 02 - identical framing.
    CapBeat { press: &[CapKey::M(MouseButton::Right)], ..beat(2.6) },
    CapBeat { snap: Some("03-fp-rockets-preaim-no-zoom"), ..beat(3.2) },
    CapBeat { release: &[CapKey::M(MouseButton::Right)], ..beat(3.3) },
    // ESC from a live first-person frame - the case the old capture
    // could not reach. No gun may appear over the plate.
    CapBeat { press: &[CapKey::K(KeyCode::Escape)], ..beat(3.6) },
    CapBeat { release: &[CapKey::K(KeyCode::Escape)], ..beat(3.8) },
    CapBeat { snap: Some("04-pause-no-viewmodel"), ..beat(4.6) },
    CapBeat { end: true, ..beat(5.0) },
];

/// Every iron-sighted gun's ADS sight picture, one frame each. Focus
/// aligns `sight_line_y` to the eye, so a gun whose declared sight line
/// does not match its actual geometry lays whatever IS at that height
/// across the view - which is exactly how the M249 shipped with its flat
/// feed cover on the eye line and no rear aperture at all. Nothing could
/// photograph that, so nothing caught it.
///
/// Slot 1 is the primary, 2 the secondary, 3 the special; the harness
/// sets loadout slots per-run (see `capture_quick_deploy`).
/// The four class silhouettes, third person, one run each. The system
/// is only worth having if you can name the class at the range you
/// decide to shoot from, so this photographs the thing that has to
/// carry that: the shape at shoulder height.
const CLASS_LOOK_BEATS: &[CapBeat] = &[
    CapBeat { look: Some((0.0, 0.02)), ..beat(0.4) },
    CapBeat { snap: Some("01-behind"), ..beat(1.2) },
    // walk a step so the shape reads in motion too, then turn to profile
    CapBeat { press: &[CapKey::K(KeyCode::KeyW)], ..beat(1.4) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyW)], ..beat(2.0) },
    CapBeat { look: Some((1.55, 0.02)), ..beat(2.2) },
    CapBeat { snap: Some("02-profile"), ..beat(3.0) },
    CapBeat { end: true, ..beat(3.4) },
];

/// §owner MELEE v2: the three swing lines, third person, caught at the
/// COCK - the frame a defender has to read to answer the attack. If
/// these three silhouettes are not distinguishable the feature does not
/// work, however correct the sim is.
///
/// Strafe is held to pick the line, and the knife tapped, so each snap
/// lands mid-wind rather than mid-recovery.
const MELEE_DIRS_BEATS: &[CapBeat] = &[
    // LEFT: strafe left, tap the blade
    CapBeat { press: &[CapKey::K(KeyCode::KeyA), CapKey::K(KeyCode::KeyF)], ..beat(0.8) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyF)], ..beat(0.84) },
    CapBeat { snap: Some("01-left-cock"), ..beat(0.94) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyA)], ..beat(1.6) },
    // RIGHT
    CapBeat { press: &[CapKey::K(KeyCode::KeyD), CapKey::K(KeyCode::KeyF)], ..beat(2.0) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyF)], ..beat(2.04) },
    CapBeat { snap: Some("02-right-cock"), ..beat(2.14) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyD)], ..beat(2.8) },
    // OVERHEAD: no strafe at all
    CapBeat { press: &[CapKey::K(KeyCode::KeyF)], ..beat(3.2) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyF)], ..beat(3.24) },
    CapBeat { snap: Some("03-overhead-cock"), ..beat(3.34) },
    CapBeat { end: true, ..beat(3.9) },
];

const IRON_SIGHTS_BEATS: &[CapBeat] = &[
    CapBeat { press: &[CapKey::K(KeyCode::KeyV)], ..beat(0.5) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyV)], ..beat(0.6) },
    // primary, hip then aimed
    CapBeat { snap: Some("01-primary-hip"), ..beat(1.4) },
    CapBeat { press: &[CapKey::M(MouseButton::Right)], ..beat(1.6) },
    CapBeat { snap: Some("02-primary-ads"), ..beat(2.4) },
    CapBeat { release: &[CapKey::M(MouseButton::Right)], ..beat(2.5) },
    // secondary
    CapBeat { press: &[CapKey::K(KeyCode::Digit2)], ..beat(2.7) },
    CapBeat { release: &[CapKey::K(KeyCode::Digit2)], ..beat(2.8) },
    CapBeat { press: &[CapKey::M(MouseButton::Right)], ..beat(3.4) },
    CapBeat { snap: Some("03-secondary-ads"), ..beat(4.2) },
    CapBeat { release: &[CapKey::M(MouseButton::Right)], ..beat(4.3) },
    // special
    CapBeat { press: &[CapKey::K(KeyCode::Digit3)], ..beat(4.5) },
    CapBeat { release: &[CapKey::K(KeyCode::Digit3)], ..beat(4.6) },
    CapBeat { press: &[CapKey::M(MouseButton::Right)], ..beat(5.2) },
    CapBeat { snap: Some("04-special-ads"), ..beat(6.0) },
    CapBeat { release: &[CapKey::M(MouseButton::Right)], ..beat(6.1) },
    CapBeat { end: true, ..beat(6.4) },
];

/// A shaft actually IN FLIGHT. Every existing projectile script snaps
/// on the draw and the release, by which point the arrow is already off
/// the frame at 82 m/s - so the thing that flies had never been
/// photographed, only the bow that launched it. These beats fire and
/// then snap on a tight cadence to catch it downrange, from third
/// person so the shaft is side-on rather than receding to a dot.
///
/// Aimed slightly UP so the arc, and the nose following it down, both
/// read across the frames.
const PROJECTILE_FLIGHT_BEATS: &[CapBeat] = &[
    CapBeat { press: &[CapKey::K(KeyCode::Digit3)], ..beat(0.4) },
    CapBeat { release: &[CapKey::K(KeyCode::Digit3)], ..beat(0.5) },
    // Aimed DOWN at the dirt a few metres out, not up. A shaft loosed
    // level recedes end-on at 82 m/s and is a two-pixel dot by the time
    // any beat can fire; angled into the ground it stays broadside,
    // close, and then STAYS there stuck for the last frame - which also
    // proves the impact angle survives (the sim keeps `vel` on stick so
    // the render can read it).
    CapBeat { look: Some((0.0, 0.42)), ..beat(0.8) },
    // draw / wind to full, then loose. The mid-hold snap catches the
    // javelin's charge bar, which nothing else photographs.
    CapBeat { press: &[CapKey::M(MouseButton::Left)], ..beat(1.0) },
    CapBeat { snap: Some("00-charging"), ..beat(1.55) },
    CapBeat { release: &[CapKey::M(MouseButton::Left)], ..beat(2.0) },
    CapBeat { snap: Some("01-launch"), ..beat(2.03) },
    CapBeat { snap: Some("02-early-flight"), ..beat(2.06) },
    CapBeat { snap: Some("03-mid-flight"), ..beat(2.10) },
    CapBeat { snap: Some("04-descending"), ..beat(2.16) },
    CapBeat { snap: Some("05-landed"), ..beat(2.9) },
    CapBeat { end: true, ..beat(3.3) },
];

/// The guard plate in first person. It used to be an opaque black slab
/// that blinded the player it was protecting; it must now read as
/// smoked glass with the world legible through it. On foot, because the
/// plate is sealed away inside a chassis.
const SHIELD_FP_BEATS: &[CapBeat] = &[
    CapBeat { press: &[CapKey::K(KeyCode::KeyV)], ..beat(0.5) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyV)], ..beat(0.6) },
    CapBeat { snap: Some("01-fp-guard-down"), ..beat(1.2) },
    // slot 4 raises it now - E is dead
    CapBeat { press: &[CapKey::K(KeyCode::Digit4)], ..beat(1.4) },
    CapBeat { release: &[CapKey::K(KeyCode::Digit4)], ..beat(1.5) },
    CapBeat { snap: Some("02-fp-guard-up-see-through"), ..beat(2.2) },
    CapBeat { end: true, ..beat(2.6) },
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
        "bow_draw_fp" => BOW_DRAW_FP_BEATS,
        "mech_scale" => MECH_CAPTURE_BEATS,
        "mech_fp" => MECH_FP_BEATS,
        "shield_fp" => SHIELD_FP_BEATS,
        "sights_a" | "sights_b" | "sights_c" => IRON_SIGHTS_BEATS,
        "arrow_flight" | "spear_flight" => PROJECTILE_FLIGHT_BEATS,
        "melee_dirs" => MELEE_DIRS_BEATS,
        "class_line" | "class_skirmisher" | "class_warden" | "class_marksman" => {
            CLASS_LOOK_BEATS
        }
        "minigun_check" => MINIGUN_CHECK_BEATS,
        "traversal" => TRAVERSAL_BEATS,
        "map_lap" => MAP_LAP_BEATS,
        _ => &[],
    }
}

/// Where a capture script writes its frames.
///
/// Anchored to the CRATE, not the working directory. These paths used to
/// be bare relative strings, so a capture landed wherever the binary
/// happened to be launched from - and since a Bevy release binary is
/// normally launched from `engine/` while the handback tree lives under
/// `engine/crates/jk_tdm/`, a perfectly successful run would write four
/// PNGs into a directory nobody looks in and report exit 0. That is how
/// `02-soldier-page` and `03-match-page` went missing from the menus
/// capture: the run worked, the files were simply somewhere else.
///
/// `CARGO_MANIFEST_DIR` is baked in at compile time and always points at
/// this crate's root, so the frames land in the tracked tree no matter
/// where the process was started.
fn capture_dir(script: &str) -> String {
    format!(
        "{}/handback/brief-vii/{script}",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    )
}

/// Populated once at Startup from `JK_CAPTURE`; if unset, every capture
/// system below is a no-op and the game behaves exactly as launched by a
/// human.
const CAPTURE_SCRIPTS: [&str; 22] = [
    "baseline",
    "idle_life",
    "bow_draw",
    "bow_draw_fp",
    "mech_scale",
    "mech_fp",
    "shield_fp",
    "sights_a",
    "sights_b",
    "sights_c",
    "arrow_flight",
    "spear_flight",
    "class_line",
    "class_skirmisher",
    "class_warden",
    "class_marksman",
    "melee_dirs",
    "minigun_check",
    "menus",
    "traversal",
    "map_lap",
    branding::CAPTURE_SPLASH_SCRIPT,
];

/// Photograph the splash: mid-hold (full art + wordmark + rule), then
/// mid-fade-out, then exit. Wall-clock driven like `capture_menus`.
fn capture_splash(
    mut commands: Commands,
    cap: Res<CaptureMode>,
    time: Res<Time>,
    mut t: Local<f32>,
    mut stage: Local<usize>,
    window: Query<Entity, With<PrimaryWindow>>,
) {
    if cap.script.as_deref() != Some(branding::CAPTURE_SPLASH_SCRIPT) {
        return;
    }
    *t += time.delta_secs();
    let snap = |commands: &mut Commands, label: &str| {
        let dir = capture_dir(branding::CAPTURE_SPLASH_SCRIPT);
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
        // mid-hold: fade-in complete, everything at full strength
        0 if *t > branding::SPLASH_SKIP_TO_S - 0.3 => {
            snap(&mut commands, "01-hold");
            *stage = 1;
        }
        // mid fade-out: the backdrop leading the art out
        1 if *t > branding::SPLASH_SKIP_TO_S + 0.2 => {
            snap(&mut commands, "02-fade-out");
            *stage = 2;
        }
        2 if *t > branding::SPLASH_SKIP_TO_S + 1.2 => std::process::exit(0),
        _ => {}
    }
}

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
    // UI scripts stay in Intro on purpose - they capture SCREENS, which
    // never enter Playing at all, so the Playing-gated drivers below
    // could never see them. A new UI script added without this guard is
    // silently dropped into a fight instead.
    if matches!(
        cap.script.as_deref(),
        Some("menus") | Some(branding::CAPTURE_SPLASH_SCRIPT)
    ) {
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
        // both bow scripts press Digit3, so slot 3 has to actually hold a
        // bow or they capture whatever the default special happens to be
        Some("bow_draw") | Some("bow_draw_fp") => sel.loadout[2] = GunKind::Bow,
        // the seven iron-sighted guns across three runs; the AWM is
        // deliberately absent (scoped-class hides its viewmodel by
        // design) as are the bow/spear/minigun, which have no irons
        Some("sights_a") => {
            sel.loadout = [GunKind::M249, GunKind::Glock, GunKind::Ak47];
        }
        Some("sights_b") => {
            sel.loadout = [GunKind::M4, GunKind::Deagle, GunKind::Mp5];
        }
        Some("sights_c") => {
            sel.loadout = [GunKind::Shotgun, GunKind::Glock, GunKind::Ak47];
        }
        Some("class_line") => sel.class = sim::Class::Line,
        Some("class_skirmisher") => sel.class = sim::Class::Skirmisher,
        Some("class_warden") => sel.class = sim::Class::Warden,
        Some("class_marksman") => sel.class = sim::Class::Marksman,
        Some("arrow_flight") => sel.loadout[2] = GunKind::Bow,
        Some("spear_flight") => sel.loadout[2] = GunKind::Spear,
        _ => {}
    }
    start_match(&sel, Mode::Tdm, &mut game, &mut next);
    if matches!(cap.script.as_deref(), Some("mech_scale") | Some("mech_fp")) {
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
        f.mech_rounds = MECH_ROUNDS;
        f.pod_ammo = POD_TUBES;
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
        let dir = capture_dir(&name);
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
    mut page: ResMut<IntroPage>,
    window: Query<Entity, With<PrimaryWindow>>,
) {
    if cap.script.as_deref() != Some("menus") {
        return;
    }
    *t += time.delta_secs();
    let snap = |commands: &mut Commands, label: &str| {
        let dir = capture_dir("menus");
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(win) = window.get_single() {
            commands
                .spawn(Screenshot::window(win))
                .observe(bevy::render::view::screenshot::save_to_disk(format!(
                    "{dir}/{label}.png"
                )));
        }
    };
    // walks all three intro pages, then Settings - the paged flow means
    // one snap no longer shows the menu, it shows a THIRD of it
    match *stage {
        0 if *t > 1.2 => {
            snap(&mut commands, "01-title-page");
            *stage = 1;
        }
        1 if *t > 1.6 => {
            page.0 = IntroPage::SOLDIER;
            *stage = 2;
        }
        2 if *t > 2.4 => {
            snap(&mut commands, "02-soldier-page");
            *stage = 3;
        }
        // §C tier 2: the ARMOURY. A page with no capture is a page that
        // can break unseen - which is exactly what happened to the two
        // frames named in `capture_dir`'s comment above.
        3 if *t > 2.8 => {
            page.0 = IntroPage::ARMOURY;
            *stage = 4;
        }
        4 if *t > 3.6 => {
            snap(&mut commands, "03-armoury-page");
            *stage = 5;
        }
        5 if *t > 4.0 => {
            page.0 = IntroPage::MATCH;
            *stage = 6;
        }
        6 if *t > 4.8 => {
            snap(&mut commands, "04-match-page");
            *stage = 7;
        }
        7 if *t > 5.2 => {
            next.set(GameState::Settings);
            *stage = 8;
        }
        8 if *t > 6.2 => {
            snap(&mut commands, "05-settings");
            *stage = 9;
        }
        // The pause menu had NO capture coverage at all - it was the one
        // surface nothing could prove, which is exactly why it was chosen
        // to pilot the new design vocabulary.
        9 if *t > 6.6 => {
            next.set(GameState::Paused);
            *stage = 10;
        }
        10 if *t > 7.6 => {
            snap(&mut commands, "06-pause");
            *stage = 11;
        }
        // Manual and Controls had no capture coverage at all - the same
        // hole the pause menu sat in until it became the pilot surface.
        11 if *t > 8.2 => {
            next.set(GameState::Manual);
            *stage = 12;
        }
        12 if *t > 9.2 => {
            snap(&mut commands, "07-manual");
            *stage = 13;
        }
        13 if *t > 9.8 => {
            next.set(GameState::Controls);
            *stage = 14;
        }
        14 if *t > 10.8 => {
            snap(&mut commands, "08-controls");
            *stage = 15;
        }
        15 if *t > 11.6 => std::process::exit(0),
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

// ---- the gameplay HUD's on/off switch ------------------------------------
//
// THE BUG THIS EXISTS FOR. Nothing hid the HUD when a menu opened. Every
// menu root and every HUD root sat at implicit z 0, and the tie-break is
// Bevy's root query order, which this crate never pinned - so the HUD
// won. Captures of the Intro and Settings screens show the match timer,
// the score line, the K/D counter, the weapon strip, the loadout panel,
// the vitals cluster and the ammo block drawn over the menu, with the
// Settings panel physically colliding with "30 / 120 FRAG x2", plus
// world-space health bars floating over NPCs behind it all.
//
// Fixed by VISIBILITY, not by z-order. A z bump would put the menu on
// top while leaving the HUD alive, still updating, and still leaking at
// the frame edges. Not `Camera.is_active` either: MainCam carries
// `IsDefaultUiCamera` and the menus render through it, so disabling it
// blanks the menus too. Not `RenderLayers`: Bevy 0.15 does not propagate
// those to children and the HUD has ~90 of them.

/// Every top-level gameplay-HUD entity carries this.
///
/// Only the ROOTS need it. Bevy propagates `Visibility::Hidden` down to
/// descendants, which this crate already relies on for `ScoreboardRoot`,
/// `ScopeRoot`, `ContextBarRoot` and the killfeed rows - so the ~90
/// children need nothing.
#[derive(Component)]
struct HudRoot;

/// THE predicate for whether the gameplay HUD is on screen.
///
/// Pure, so it is testable without standing up Bevy, and so no system can
/// keep a second drifting copy of the answer - `minimap_system` had
/// exactly that, and it disagreed (it counted Paused as in-match).
fn hud_visible(state: &GameState) -> bool {
    matches!(state, GameState::Playing)
}

/// Drive HUD visibility off the state. Registered on BOTH
/// `OnEnter(Playing)` and `OnExit(Playing)`, and it READS the state
/// rather than assuming a direction, so the two registrations cannot
/// disagree with each other.
///
/// It writes `Inherited`, never `Visible`. `ScoreboardRoot`, `ScopeRoot`
/// and `ContextBarRoot` own their own visibility; forcing `Visible` here
/// would pop the scoreboard for a frame on every resume. `Inherited` on
/// a root resolves to visible, and their own systems re-assert within the
/// same frame because state transitions run before `Update`.
fn hud_visibility(
    state: Res<State<GameState>>,
    mut q: Query<&mut Visibility, Or<(With<HudRoot>, With<branding::EmblemWatermark>)>>,
) {
    let v = if hud_visible(state.get()) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut q {
        *vis = v;
    }
}

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

// ---- the pre-game flow ----------------------------------------------
// The intro used to be ONE screen carrying the title, ten pick-rows and
// three deploy buttons. At the game's own 720p default that is a wall
// of controls with no order to it - a capture of it is genuinely hard
// to read, and nothing tells a new player what to look at first.
//
// Split into three pages, each answering one question:
//   1. TITLE      - what is this? (key art, wordmark, emblem)
//   2. THE MATCH  - where and how hard? (mode, map, difficulty, size)
//   3. THE SOLDIER- what am I carrying? (weapons, grenades, colours)
//
// Deliberately a RESOURCE rather than new `GameState` variants: every
// row already exists and is already wired: paging only decides what is
// VISIBLE. New states would mean re-spawning the whole tree per page
// and re-solving teardown, which is how the lingering-entity bug that
// `close_intro` documents got in the first time.

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct IntroPage(pub u8);

impl IntroPage {
    pub const TITLE: u8 = 0;
    pub const SOLDIER: u8 = 1;
    /// §C tier 2: the armoury - 24 plates, four rows, and the weight.
    ///
    /// Its OWN page rather than five more rows on SOLDIER, because
    /// SOLDIER already runs past the bottom of a 1080p plate: the class,
    /// three weapon slots, melee, grenades, three cosmetic rows, the
    /// Forge save/load block and the loadout spec fill it. Five more
    /// would have pushed the Forge buttons off the screen, and a save
    /// button you cannot reach is worse than an armour grid you have to
    /// click Next to see.
    pub const ARMOURY: u8 = 2;
    pub const MATCH: u8 = 3;
    pub const LAST: u8 = 3;

    /// Heading for each page - the question the page answers.
    pub fn heading(self) -> &'static str {
        match self.0 {
            Self::MATCH => "CHOOSE YOUR BATTLE",
            Self::SOLDIER => "EQUIP YOUR SOLDIER",
            Self::ARMOURY => "PLATE YOUR SOLDIER",
            _ => "",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self.0 {
            // "ENTER to begin" was a LIE. `intro_keyboard_paging` clamps
            // Enter at IntroPage::LAST, so on the match page Enter does
            // nothing and there has never been a keyboard path to start
            // a match - deploying requires clicking a mode. Removing the
            // false promise costs one string; adding a real keyboard
            // deploy would need a fourth selection concept.
            Self::TITLE => "ENTER - continue    -    ESC menu > RULES & MANUAL",
            Self::MATCH => "the battlefield, the mode, and how hard it pushes back    -    CLICK A MODE TO DEPLOY",
            Self::SOLDIER => "the shield always rides in its own slot (4 raises it)",
            Self::ARMOURY => {
                "every plate is optional - what you leave off is lighter, and softer where it was"
            }
            _ => "",
        }
    }
}

/// Tags a row/element with the page it belongs to.
#[derive(Component, Clone, Copy)]
struct OnIntroPage(u8);

/// Page navigation buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum IntroNav {
    Back,
    Next,
}

fn tech_readout(sel: Res<Selected>, mut q: Query<&mut Text, With<TechReadout>>) {
    let Ok(mut t) = q.get_single_mut() else {
        return;
    };
    let mut s = String::from("- LOADOUT SPEC -\n");
    // §owner: the CLASS line first - it is the only pick on this page
    // that changes how the soldier PLAYS rather than what he carries, so
    // the trade has to be readable before you commit to it. Percentages
    // are deltas from LINE (1.0 across the board); anything at parity is
    // omitted, so the line reads as "what is different about me".
    {
        let cs = sim::class_spec(sel.class);
        let mut deltas: Vec<String> = Vec::new();
        for (label, mult, higher_is_better) in [
            ("hp", cs.health_mult, true),
            ("speed", cs.move_mult, true),
            ("aim", cs.spread_mult, false),
            ("swap", cs.switch_mult, false),
        ] {
            if (mult - 1.0).abs() < 1e-3 {
                continue;
            }
            // a spread/switch multiplier BELOW 1.0 is an IMPROVEMENT, so
            // the sign the player should read is not the sign of the
            // number - mark the ones that cost you.
            let pct = (mult - 1.0) * 100.0;
            let good = if higher_is_better { pct > 0.0 } else { pct < 0.0 };
            deltas.push(format!(
                "{label} {}{:.0}%{}",
                if pct > 0.0 { "+" } else { "" },
                pct,
                if good { "" } else { " cost" }
            ));
        }
        let line = if deltas.is_empty() {
            "baseline across the board".to_string()
        } else {
            deltas.join("  ")
        };
        s += &format!("{:<14} {line}\n", cs.name);
        s += &format!("{:<14} {}\n", "", cs.blurb);
    }
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
    /// §owner: the practice range.
    Training,
}

#[derive(Component, Clone, Copy)]
struct MapButton(MapKind);

#[derive(Component, Clone, Copy)]
struct DiffButton(Difficulty);

#[derive(Component, Clone, Copy)]
struct SizeButton(usize);

/// §owner: the TDM score target pick - a quick match or a long one.
#[derive(Component, Clone, Copy)]
struct ScoreButton(u32);

/// Loadout pick: (slot index, weapon).
#[derive(Component, Clone, Copy)]
struct LoadoutButton(usize, GunKind);

/// §7.2/§8.2: the Forge's first VISIBLE controls. The three profile
/// slots and the dice existed only as hotkeys (Ctrl+1/2/3, 1/2/3) with a
/// toast as their sole feedback - a save system nothing on screen
/// admitted to having. These are the same `forge_save`/`forge_load`
/// calls, clickable. The full specced editor (category grid, turntable,
/// per-piece armour) remains open work; this is its front door.
#[derive(Component, Clone, Copy)]
enum ForgeUiButton {
    Save(usize),
    Load(usize),
    Randomize,
}

/// Which appearance pick a cosmetic pill drives.
///
/// A named slot rather than the `0`/`1` this used to be: §8.1 made it
/// three, and a chain of `if slot == 0 ... else` over bare integers is one
/// where adding the third silently routes it to the `else`. The compiler
/// checks a match over this.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum CosmeticSlot {
    /// §8.1: the helmet SHAPE - `HELMET_CHOICES`.
    Helmet,
    /// The helmet TINT - `HAT_CHOICES`. Orthogonal to the shape.
    HatTint,
    Tunic,
}

/// One appearance pill: which slot, and which index within it.
#[derive(Component, Clone, Copy)]
struct CosmeticButton(CosmeticSlot, usize);

/// §C tier 2: one plate in the Forge's armour grid.
///
/// A TOGGLE, not a pick - which is why it cannot ride `pill_row`'s
/// selection convention unchanged. Every other row in the Forge is "one
/// of these", so exactly one pill is lit; here every pill is independent
/// and any number can be lit at once. The visual language survives
/// because "lit = you have this" is what a lit pill already meant.
#[derive(Component, Clone, Copy)]
struct ArmorButton(sim::ArmorPiece);

/// The live weight readout under the armour grid.
#[derive(Component)]
struct ArmorWeightText;

/// §C tier 2: one of the five standard harnesses.
#[derive(Component, Clone, Copy)]
struct ArmorPresetButton(sim::ArmorPreset);

/// How the armour grid is LAID OUT, which is deliberately finer than
/// how it is scored.
///
/// Damage resolves against four zones, and the first version of this
/// page used those four as its rows - which put ten pills in LEGS and
/// eight in ARMS against four in TORSO. Every label in the long rows
/// wrapped to two lines and the page read as a wall.
///
/// The eye wants even rows and the hand wants to find a plate, so the
/// grouping here is ANATOMICAL rather than mechanical: six rows of two
/// to six. It changes nothing about what the pieces do - `ArmorPiece`
/// still answers to `zone()` for damage - and a test pins that every
/// plate appears in exactly one row, so a piece cannot go missing from
/// the UI while still counting toward weight.
/// How much of the armoury plate the pill rows may use.
///
/// The turntable card is absolutely positioned against the SCREEN, not
/// the plate, and it overhangs the plate's right edge. Every other intro
/// page gets away with full-width rows because none of them is long
/// enough to reach under it; the armoury's six-pill rows are.
const ARMOURY_ROW_W_PCT: f32 = 74.0;

const ARMOUR_ROWS: [(&str, &[sim::ArmorPiece]); 6] = {
    use sim::ArmorPiece::*;
    [
        ("HEAD", &[Helmet, Gorget]),
        ("TORSO", &[CuirassFront, CuirassBack, Fauld, PelvisPlate]),
        ("SHOULDERS", &[PauldronL, PauldronR, RerebraceL, RerebraceR]),
        ("FOREARMS", &[VambraceL, VambraceR, GauntletL, GauntletR]),
        ("THIGHS", &[TassetL, TassetR, CuisseL, CuisseR]),
        ("SHINS", &[PoleynL, PoleynR, GreaveL, GreaveR, SabatonL, SabatonR]),
    ]
};

/// §6 (Brief IV): melee slot pick - false = knife, true = axe.
#[derive(Component, Clone, Copy)]
struct MeleeButton(bool);

/// §owner: the CLASS pick - the standing choice of who you fight as.
#[derive(Component, Clone, Copy)]
struct ClassButton(sim::Class);

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

/// The pause menu's rows, in SCREEN order, with the bind that also
/// performs them and whether they throw work away.
///
/// ONE table. The `MenuButton` declaration order above already disagrees
/// with the display order, so a future data-driven loop over the enum
/// would silently render them wrong. `menu_buttons` dispatches on the
/// variant and never on an index, so this order is free to change.
const PAUSE_ROWS: [(MenuButton, &str, Option<&str>, menu_ui::RowKind); 7] = [
    (MenuButton::Resume, "RESUME", Some("ESC"), menu_ui::RowKind::Normal),
    (MenuButton::Restart, "RESTART MATCH", None, menu_ui::RowKind::Destructive),
    (MenuButton::BackToLoadout, "CHANGE MODE / LOADOUT", None, menu_ui::RowKind::Normal),
    (MenuButton::Settings, "SETTINGS", None, menu_ui::RowKind::Normal),
    (MenuButton::Controls, "CONTROLS", None, menu_ui::RowKind::Normal),
    (MenuButton::Manual, "RULES & MANUAL", None, menu_ui::RowKind::Normal),
    (MenuButton::Quit, "QUIT", None, menu_ui::RowKind::Destructive),
];

/// §4.x: drive `UiScale` from the window height so the whole interface
/// keeps its authored proportions from 720p to 4K.
///
/// Writes ONLY on change. An unconditional write marks the resource
/// `Changed` and forces a full layout pass over every node, every frame.
fn sync_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
) {
    let Ok(w) = windows.get_single() else { return };
    let want = menu_ui::menu_ui_scale(w.resolution.height());
    if (ui_scale.0 - want).abs() > f32::EPSILON {
        ui_scale.0 = want;
    }
}

#[derive(Component, Clone, Copy)]
enum SettingsButton {
    SwapMouse,
    Minimap,
    Sens,
    Fov,
    InvertY,
    // §4.6 (Brief VIII): the crosshair family. A persisted setting with
    // no control is a dead control - these are the rows that make the
    // eleven new `GameSettings` fields reachable without a text editor.
    CrossSize,
    CrossGap,
    CrossThickness,
    CrossDot,
    CrossOutline,
    CrossColor,
    CrossAlpha,
    CrossTShape,
    CrossDynamic,
    // §4.1/§4.3: same rule as the crosshair family above - a persisted
    // setting with no control is a dead control.
    VitalsStyle,
    MinimapRotate,
    MinimapScale,
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
    CrossSize,
    CrossGap,
    CrossThickness,
    CrossDot,
    CrossOutline,
    CrossColor,
    CrossAlpha,
    CrossTShape,
    CrossDynamic,
    VitalsStyle,
    MinimapRotate,
    MinimapScale,
}

/// Every value row on the settings page, in screen order, paired with
/// the label kind that renders it. ONE table, so the page, the click
/// handler and the label text cannot disagree about what exists - the
/// list used to be hand-written inline in `open_settings` only.
/// (`Back` is not here: it has no value to show.)
/// Settings sections.
///
/// The old table's own comment claimed the two minimap tunables "sit
/// directly under the on/off that governs them, so the group reads as one
/// idea". They did not: the layout wrapped ROW-MAJOR into two columns, so
/// Minimap landed in the left cell and MinimapRotate in the RIGHT cell of
/// the same visual line. Sections are explicit now, and a column holds
/// WHOLE sections, so the grouping the comment promised is finally the
/// grouping on screen.
#[derive(Clone, Copy, PartialEq)]
enum SettingsGroup {
    Aim,
    Minimap,
    Hud,
    Crosshair,
}

impl SettingsGroup {
    fn title(self) -> &'static str {
        match self {
            Self::Aim => "AIM AND VIEW",
            Self::Minimap => "MINIMAP",
            Self::Hud => "HUD",
            Self::Crosshair => "CROSSHAIR",
        }
    }
}

/// Every value row, grouped. REORDERING IS SAFE: `settings_buttons`
/// matches on the VARIANT, never on an index or a position - which is the
/// single most important fact about this surface.
const SETTINGS_ROWS: [(SettingsButton, SettingsButtonKind, SettingsGroup); 17] = [
    (SettingsButton::Sens, SettingsButtonKind::Sens, SettingsGroup::Aim),
    (SettingsButton::Fov, SettingsButtonKind::Fov, SettingsGroup::Aim),
    (SettingsButton::InvertY, SettingsButtonKind::InvertY, SettingsGroup::Aim),
    (SettingsButton::SwapMouse, SettingsButtonKind::SwapMouse, SettingsGroup::Aim),
    (SettingsButton::Minimap, SettingsButtonKind::Minimap, SettingsGroup::Minimap),
    (SettingsButton::MinimapRotate, SettingsButtonKind::MinimapRotate, SettingsGroup::Minimap),
    (SettingsButton::MinimapScale, SettingsButtonKind::MinimapScale, SettingsGroup::Minimap),
    (SettingsButton::VitalsStyle, SettingsButtonKind::VitalsStyle, SettingsGroup::Hud),
    (SettingsButton::CrossSize, SettingsButtonKind::CrossSize, SettingsGroup::Crosshair),
    (SettingsButton::CrossGap, SettingsButtonKind::CrossGap, SettingsGroup::Crosshair),
    (SettingsButton::CrossThickness, SettingsButtonKind::CrossThickness, SettingsGroup::Crosshair),
    (SettingsButton::CrossDot, SettingsButtonKind::CrossDot, SettingsGroup::Crosshair),
    (SettingsButton::CrossOutline, SettingsButtonKind::CrossOutline, SettingsGroup::Crosshair),
    (SettingsButton::CrossColor, SettingsButtonKind::CrossColor, SettingsGroup::Crosshair),
    (SettingsButton::CrossAlpha, SettingsButtonKind::CrossAlpha, SettingsGroup::Crosshair),
    (SettingsButton::CrossTShape, SettingsButtonKind::CrossTShape, SettingsGroup::Crosshair),
    (SettingsButton::CrossDynamic, SettingsButtonKind::CrossDynamic, SettingsGroup::Crosshair),
];

/// The tests assert on `settings_label_text`'s FULL "Name: value" string -
/// that every one is non-empty, globally distinct, and changes when its
/// field changes. So the function keeps returning the whole thing, and the
/// two-node split is derived HERE, at the call site, where it cannot
/// disturb six tests.
fn split_label(full: &str) -> (&str, &str) {
    full.split_once(": ").unwrap_or((full, ""))
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
        .init_resource::<IntroPage>()
        // Sampled from the key art. Was a cool blue-grey, which fought
        // the warm gold-and-sepia art on every menu screen.
        .insert_resource(ClearColor(branding::palette::DUST))
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
        .add_systems(Update, (capture_menus, capture_splash))
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
                // Paused too: a script that drives ESC has to keep
                // ticking on the far side of the transition or it can
                // never snap the menu and never reach its `end` beat -
                // the process just hangs until the tool times out.
                .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
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
        // HUD WRITERS - Playing only. Hiding the roots is not enough on
        // its own: several of these force their own entity back to
        // `Visible` every frame and would undo `hud_visibility` on the
        // very next tick. `stability_bracket` re-shows both brackets
        // whenever the player is alive and armed, in ANY state;
        // `scoreboard_system` re-shows the scoreboard on Tab or a
        // finished round, in ANY state, so a match that ended before you
        // paused popped the scoreboard over the menu; `scope_overlay`
        // re-shows the scope whenever `cam_ctl.ads` is still set.
        .add_systems(
            Update,
            (
                hud_system,
                hud_fade,
                scoreboard_system,
                damage_indicator,
                scope_overlay,
                compass_system,
                stability_bracket,
                health_vignette,
                weapon_strip,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // NOT HUD - audio, world-space visuals and debug. These are
        // correct to run in every state and are deliberately left alone.
        // `minimap_system` stays here too: it owns the M hotkey and does
        // its own state check, now routed through `hud_visible` so the
        // crate has exactly one answer to "is the HUD up".
        .add_systems(
            Update,
            (
                sfx_system,
                distant_gunfire,
                ads_detail,
                checkpoint_rings,
                minimap_system,
                zone_overlay,
                tag_viewmodel_layer,
                tag_forge_preview_layer,
            ),
        )
        .init_resource::<DebugZones>()
        .init_resource::<DistantShots>()
        .init_resource::<Toast>()
        .add_systems(Update, esc_toggle)
        // §4.x: the interface keeps its authored proportions from 720p to
        // 4K, and the key art re-picks its cover axis when the window
        // changes aspect.
        .add_systems(Update, sync_ui_scale)
        .add_systems(
            Update,
            menu_ui::key_art_refit.run_if(on_event::<bevy::window::WindowResized>),
        )
        .add_systems(OnEnter(GameState::Playing), grab_cursor)
        // The HUD's on/off switch. Both edges, one system - it reads the
        // state rather than assuming a direction, so the two
        // registrations cannot drift apart.
        //
        // PostStartup is NOT redundant with those two. The app boots
        // straight into `GameState::Intro`, so neither Playing edge has
        // fired yet and nothing has ever hidden the HUD - the very first
        // screen a player sees was showing it. That was invisible for the
        // text elements (their writers are Playing-gated, so they simply
        // render empty) and glaring for anything that carries its own
        // colour: the vitals bar, the armour pips and the emblem
        // watermark all sat on top of the title page. PostStartup rather
        // than Startup so `setup` has already spawned them.
        .add_systems(PostStartup, hud_visibility)
        .add_systems(OnEnter(GameState::Playing), hud_visibility)
        .add_systems(OnExit(GameState::Playing), hud_visibility)
        .add_systems(OnEnter(GameState::Intro), open_intro)
        .add_systems(
            Update,
            (intro_paging, intro_nav_buttons, intro_keyboard_paging)
                .run_if(in_state(GameState::Intro)),
        )
        .add_systems(OnExit(GameState::Intro), close_intro)
        .add_systems(
            Update,
            (
                intro_buttons,
                intro_forge_buttons,
                intro_map_buttons,
                intro_loadout_buttons,
                intro_cosmetic_buttons,
                intro_armor_buttons,
                intro_armor_preset_buttons,
                intro_class_buttons,
                intro_score_buttons,
                intro_melee_buttons,
                intro_nade_buttons,
                intro_diff_buttons,
                intro_size_buttons,
                tech_readout, // §14: live weapon numbers on the loadout
                forge_preview_spin,
                forge_preview_sync,
                attach_turntable_card,
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
                spin_mech_turret_barrels,
                mech_barrier_sync,
                bow_string_sync,
                grenade_arc,
                rocket_aim_preview,
                crosshair_render,
                ammo_bar_sync,
                hud_colors,
                vitals_bars,
                killfeed_rows,
                context_bar,
                award_toasts,
                sync_rockets,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

/// Draw every bow in the world - the string halves and the arrow on them.
///
/// One system for every bow that exists: the player's viewmodel, the
/// player's own body, and every bot's. They differ only in who wrote
/// `BowDraw`, which is exactly the point - the SHAPE of a drawn bow is one
/// piece of knowledge and it lives here alone. The old code had the hand
/// in the body rig, the string frozen in the model table, and the arrow
/// stuck to a hand, and no two of them agreed.
///
/// Cosmetic only, and downstream of the sim in every sense: it reads a
/// number the sim produced and writes Transforms nothing reads back.
fn bow_string_sync(
    bows: Query<(&BowDraw, &Children)>,
    mut halves: Query<(&BowStringHalf, &mut Transform), Without<NockedArrow>>,
    mut arrows: Query<(&mut Transform, &mut Visibility), With<NockedArrow>>,
) {
    for (draw, children) in &bows {
        for child in children.iter() {
            if let Ok((side, mut t)) = halves.get_mut(*child) {
                *t = bow_string_half(side.0, draw.pull);
            } else if let Ok((mut t, mut v)) = arrows.get_mut(*child) {
                *t = bow_nocked_arrow(draw.pull);
                // An empty nock is INFORMATION - it is how an opponent
                // reads that this archer cannot loose yet.
                *v = if draw.nocked {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// §1 (Brief V): the grenade pre-aim arc. While the throw is held this
/// calls the sim's OWN `throw_release_velocity` + `predict_grenade` -
/// never a reimplementation - so the dots trace exactly the flight the
/// grenade will take, live, as the camera moves and the power charges.
/// Dots after the first bounce render faint: less certain, by design.
/// §owner: rockets get a pre-fire read too. While the pod is aimed (Y,
/// in a chassis, tubes loaded) a dotted line runs from the mount along
/// the crosshair to the first cover hit - the exact path a TAP would
/// dumb-fire down. Purely client-side; the sim is never consulted about
/// intent, only about geometry.
fn rocket_aim_preview(
    game: Res<Game>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    vis: Res<RocketAimVis>,
    cam_q: Query<&Transform, With<MainCam>>,
    mut q: Query<(&mut Transform, &mut Visibility), Without<MainCam>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    // §C.7: the arc shows whenever the pod is being PRE-AIMED - Y, or
    // RMB with the ROCKETS mount selected (mirrors the cmd mapping)
    let show = p.alive()
        && p.in_mech()
        && p.pod_ammo > 0
        && (keys.pressed(KeyCode::KeyY)
            || (p.mech_weapon == sim::MechWeapon::Rockets
                && buttons.pressed(MouseButton::Right)));
    if !show {
        for e in &vis.0 {
            if let Ok((_, mut v)) = q.get_mut(*e) {
                *v = Visibility::Hidden;
            }
        }
        return;
    }
    let Ok(cam_tf) = cam_q.get_single() else { return };
    let (d, _) = crosshair_aim_dir(&game.sim, cam_tf);
    let o = game.sim.muzzle_origin(game.sim.player);
    let dir = [d.x, d.y, d.z];
    let t_end = game
        .sim
        .raycast_cover(o, dir, 90.0)
        .map(|(t, _)| t)
        .unwrap_or(90.0);
    let n = vis.0.len() as f32;
    for (k, e) in vis.0.iter().enumerate() {
        let t = t_end * (k as f32 + 1.0) / n;
        if let Ok((mut tf, mut v)) = q.get_mut(*e) {
            tf.translation =
                Vec3::new(o[0] + dir[0] * t, o[1] + dir[1] * t, o[2] + dir[2] * t);
            *v = Visibility::Visible;
        }
    }
}

fn grenade_arc(
    game: Res<Game>,
    arc: Res<GrenadeArcVis>,
    cam_ctl: Res<CamCtl>,
    cam_q: Query<&Transform, With<MainCam>>,
    mut q: Query<(&mut Transform, &mut Visibility), Without<MainCam>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    // §5 (owner): the arc is what RIGHT CLICK buys. Holding a grenade
    // shows nothing; AIMING it draws the flight. You can still throw
    // blind - it just does not tell you where it lands.
    let show = p.alive() && p.cook_t > 0.0 && cam_ctl.ads;
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

/// The cycle clock for whatever weapon this fighter is ACTUALLY firing.
///
/// Client FX — casing ejection, muzzle flash, shot audio, scope flinch —
/// all detect a fresh shot by watching a cooldown JUMP UP. That worked
/// while every weapon in the game shared `fire_cd`.
///
/// §C gave the hull mounts their own clocks (`gatling_cd`,
/// `autocannon_cd`) — correctly, because a mount sharing `fire_cd` was
/// throttling the pilot's carried gun on dismount. But it also meant
/// the mounts stopped feeding every FX site, so a firing hull gatling
/// went silent and flashless. This is the one function that keeps them
/// wired, so the FX follow the weapon that actually fired instead of a
/// field that happens to be named `fire_cd`.
///
/// One shared helper rather than the same `if in_mech` at five call
/// sites: five copies of a rule drift, one cannot.
// ---- §4.7: death -> killer-cam -> spectate --------------------------

/// How long the camera lingers on the corpse-to-killer look before
/// handing off to the spectate framing.
const KILLER_CAM_S: f32 = 1.4;

/// What the dead player's camera should be doing, as a pure function of
/// sim state - shared by the camera and the HUD so the two can never
/// disagree about which phase the death is in.
///
/// Returns `None` when there is nothing to spectate: the fighter is
/// alive, killed themself (a cooked frag has no killer worth watching),
/// or the killer index is out of range. `Some((killer, spectating))`
/// otherwise, where `spectating=false` is the brief killer-cam and
/// `true` is the follow framing.
///
/// The phase clock derives from `respawn_t` counting DOWN from its known
/// initial value (RESPAWN_S, or the Extraction no-respawn sentinel), so
/// no new state exists anywhere - a replay reproduces the exact same
/// camera phases for free.
fn death_phase(sim: &TdmSim, me: usize) -> Option<(usize, bool)> {
    let p = &sim.fighters[me];
    if p.alive() {
        return None;
    }
    let (killer, _) = p.last_hit_by?;
    if killer == me || killer >= sim.fighters.len() {
        return None;
    }
    let total = if sim.mode == Mode::Extraction { 9999.0 } else { RESPAWN_S };
    let elapsed_dead = (total - p.respawn_t).max(0.0);
    Some((killer, elapsed_dead >= KILLER_CAM_S))
}

fn shot_clock(f: &Fighter) -> f32 {
    if f.in_mech() {
        match f.mech_weapon {
            sim::MechWeapon::Gatling => f.gatling_cd,
            sim::MechWeapon::Autocannon => f.autocannon_cd,
            // §C.7: the pod's cycle is the relaunch cooldown
            sim::MechWeapon::Rockets => f.pod_cd,
        }
    } else {
        f.fire_cd
    }
}

/// Detect fresh shots by each fighter's shot clock jumping UP, and eject
/// a casing from beside the action. Bows, spears, and fists leave none.
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
        let clock = shot_clock(f);
        let fresh = clock > prev_cd[i] + 1e-6;
        prev_cd[i] = clock;
        if !fresh || budget == 0 {
            continue;
        }
        if matches!(f.gun, GunKind::Bow | GunKind::Spear | GunKind::Fists) {
            continue;
        }
        // §C.7: a rocket tube ejects no brass
        if f.in_mech() && f.mech_weapon == sim::MechWeapon::Rockets {
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
fn sync_rockets(
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    cam_q: Query<&GlobalTransform, With<MainCam>>,
    mut q: Query<(&RocketVis, &mut Transform, &mut Visibility)>,
) {
    // Our own launch, seen in first person, should leave the TUBE. The
    // sim spawns the bird ahead of the eye because that is where the
    // collision and the homing have to start from, and moving it would
    // be a gameplay change - so only the drawn position is corrected,
    // and only for the first instants of flight, easing onto the true
    // position before anything can be hit. After that the visual IS the
    // truth again.
    let fp_muzzle: Option<Vec3> = if cam_ctl.first_person {
        cam_q.get_single().ok().map(|g| {
            let p = &game.sim.fighters[game.sim.player];
            g.transform_point(fp_muzzle_local(p))
        })
    } else {
        None
    };
    for (rv, mut tf, mut vis) in &mut q {
        if let Some(r) = game.sim.rockets.get(rv.0) {
            tf.translation = Vec3::from_array(r.pos);
            if let Some(m) = fp_muzzle {
                if r.shooter == game.sim.player && r.t < ROCKET_LAUNCH_BLEND_S {
                    let k = 1.0 - (r.t / ROCKET_LAUNCH_BLEND_S).clamp(0.0, 1.0);
                    tf.translation = tf.translation.lerp(m, k);
                }
            }
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

/// Every weapon's geometry, as DATA.
///
/// Root sits at the grip; +Z is the muzzle. (The bow runs along +-X
/// with its string toward -Z.) Per-gun differentiation is PROPORTION,
/// not new part types; tone changes - not geometry - suggest
/// complexity.
///
/// This table used to live INSIDE `spawn_weapon_model`, welded to
/// `Commands`, and that had a cost nobody was charging for: the
/// geometry could not be READ. The screen-intrusion budgets that
/// govern these very models had to be transcribed by eye into
/// constants, and nothing could check that a model still fitted the
/// budget it was given - so widening a gun silently widened the
/// budget's lie. `weapon_bounded_extent` measures them now.
///
/// The minigun's barrel cluster was the one thing blocking the move:
/// it spawned onto a `MinigunSpinner` child. It is a `spin` FLAG on
/// the part now, and the caller does the parenting.
fn weapon_parts(kind: GunKind) -> Vec<WPart> {
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
            parts.push(wp(false, Tone::Black, (0.0, 0.085, 0.15), 0.0, (0.008, 0.012, 0.01)));
            // 1x red dot on the slide - replaces the goalpost irons
            push_red_dot(&mut parts, 0.1075, -0.015, 0.079); // slide top 0.079
        }
        GunKind::Deagle => {
            // the hand cannon: long light slide, heavy dark frame
            parts.push(wp(false, Tone::Light, (0.0, 0.055, 0.10), 0.0, (0.052, 0.075, 0.30)));
            parts.push(wp(false, Tone::Mid, (0.0, 0.096, 0.10), 0.0, (0.030, 0.012, 0.28)));
            parts.push(wp(false, Tone::Dark, (0.0, 0.0, 0.07), 0.0, (0.048, 0.05, 0.24)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.055, -0.01), 0.20, (0.046, 0.14, 0.065)));
            push_muzzle(&mut parts, 0.055, 0.27, 0.055);
            parts.push(wp(false, Tone::Black, (0.0, 0.10, 0.22), 0.0, (0.008, 0.014, 0.01)));
            push_red_dot(&mut parts, 0.1300, -0.02, 0.102); // slide rib top 0.102
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
            // §5 (owner): the MP5 was the one gun with no sights modelled
            // at all. Rear notch on the receiver, front post at the muzzle
            // end, both at the same height so they line up.
            parts.push(wp(false, Tone::Black, (0.0, 0.088, 0.30), 0.0, (0.008, 0.016, 0.01)));
            push_red_dot(&mut parts, 0.1160, 0.005, 0.089); // rail top 0.089
        }
        GunKind::Shotgun => {
            // pump gun: barrel + tube pair over a light pump
            parts.push(wp(false, Tone::Dark, (0.0, 0.02, 0.02), 0.0, (0.05, 0.085, 0.30)));
            parts.push(wp(true, Tone::Mid, (0.0, 0.045, 0.38), FRAC_PI_2, (0.028, 0.48, 0.028)));
            parts.push(wp(true, Tone::Dark, (0.0, -0.005, 0.36), FRAC_PI_2, (0.024, 0.42, 0.024)));
            parts.push(wp(false, Tone::Light, (0.0, -0.015, 0.30), 0.0, (0.054, 0.05, 0.16)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.035, -0.20), 0.12, (0.045, 0.10, 0.26)));
            parts.push(wp(false, Tone::Mid, (0.0, -0.035, -0.325), 0.12, (0.05, 0.11, 0.02)));
            parts.push(wp(false, Tone::Black, (0.0, 0.09, 0.55), 0.0, (0.008, 0.016, 0.01)));
            push_red_dot(&mut parts, 0.0950, -0.03, 0.0625); // receiver top 0.0625
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
            push_red_dot(&mut parts, 0.1060, 0.08, 0.078); // top cover 0.078
            parts.push(wp(false, Tone::Black, (0.0, 0.09, 0.58), 0.0, (0.008, 0.018, 0.01)));
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
            parts.push(wp(false, Tone::Black, (0.0, 0.105, 0.24), 0.0, (0.008, 0.018, 0.01)));
            push_red_dot(&mut parts, 0.1120, 0.0, 0.084); // rail top 0.084
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
            // §owner: the parts that make a rifle read as a SNIPER rather
            // than a long tube - elevation and windage turrets, the bolt
            // you cycle between shots, a box magazine, a cheek riser, and
            // a slotted brake on the muzzle.
            // elevation turret on top of the scope, windage on the side
            parts.push(wp(true, Tone::Dark, (0.0, 0.148, 0.10), 0.0, (0.036, 0.030, 0.036)));
            parts.push(wp(true, Tone::Black, (0.0, 0.166, 0.10), 0.0, (0.026, 0.012, 0.026)));
            parts.push(wp(true, Tone::Dark, (0.052, 0.10, 0.10), FRAC_PI_2, (0.032, 0.028, 0.032)));
            // bolt handle: a stub off the right of the receiver, angled
            // down and back the way a turned-down bolt sits
            parts.push(wp(true, Tone::Mid, (0.046, 0.045, -0.05), FRAC_PI_2, (0.018, 0.075, 0.018)));
            parts.push(wp(true, Tone::Black, (0.082, 0.032, -0.05), FRAC_PI_2, (0.024, 0.030, 0.024)));
            // detachable box magazine under the action
            parts.push(wp(false, Tone::Dark, (0.0, -0.075, 0.03), 0.10, (0.038, 0.10, 0.10)));
            parts.push(wp(false, Tone::Black, (0.0, -0.128, 0.03), 0.10, (0.042, 0.012, 0.11)));
            // raised cheek piece on the comb
            parts.push(wp(false, Tone::Mid, (0.0, 0.055, -0.20), 0.0, (0.040, 0.030, 0.20)));
            // slotted muzzle brake - three cuts, the sniper's own tell
            for bz in [0.885_f32, 0.905, 0.925] {
                parts.push(wp(false, Tone::Black, (0.0, 0.052, bz), 0.0, (0.050, 0.008, 0.006)));
            }
        }
        GunKind::M249 => {
            // belt-fed support gun: deep receiver, box mag, thick barrel
            parts.push(wp(false, Tone::Dark, (0.0, 0.02, 0.05), 0.0, (0.075, 0.12, 0.50)));
            parts.push(wp(false, Tone::Light, (0.0, 0.088, 0.05), 0.0, (0.07, 0.02, 0.30)));
            parts.push(wp(true, Tone::Dark, (0.0, 0.04, 0.50), FRAC_PI_2, (0.045, 0.40, 0.045)));
            push_muzzle(&mut parts, 0.04, 0.73, 0.06);
            parts.push(wp(false, Tone::Mid, (0.0, -0.13, 0.02), 0.0, (0.09, 0.16, 0.13)));
            // carry handle - ARCHED ABOVE the sight line, not across it.
            // At y 0.12 x 0.06 tall it spanned 0.090-0.150 and contained
            // the sight ray, so the aperture looked into solid plastic.
            parts.push(wp(false, Tone::Light, (0.0, 0.171, 0.08), 0.0, (0.02, 0.042, 0.16)));
            parts.push(wp(false, Tone::Light, (0.0, 0.150, 0.015), 0.0, (0.018, 0.036, 0.018)));
            parts.push(wp(false, Tone::Light, (0.0, 0.150, 0.145), 0.0, (0.018, 0.036, 0.018)));
            push_stock(&mut parts, -0.30, 0.05);
            parts.push(wp(false, Tone::Dark, (0.03, -0.10, 0.44), 0.0, (0.014, 0.16, 0.014)));
            parts.push(wp(false, Tone::Dark, (-0.03, -0.10, 0.44), 0.0, (0.014, 0.16, 0.014)));
            // sights ride a raised block CLEAR of the feed cover. The
            // cover's top is 0.098; the old sight line was 0.10, so
            // focus laid a 30 cm plate exactly across the eye and the
            // gun read as a grey wall. Front post is a tall blade off
            // the barrel so it shows THROUGH the rear aperture.
            parts.push(wp(false, Tone::Black, (0.0, 0.095, 0.62), 0.0, (0.008, 0.075, 0.01)));
            push_red_dot(&mut parts, 0.1265, 0.0, 0.098); // feed cover 0.098
        }
        GunKind::Bow => {
            // §owner: the war bow is held HORIZONTAL, and it CURVES.
            //
            // It used to be two straight blocks stacked vertically, which
            // read as a pole rather than a bow and put the upper limb
            // through the shooter's sight line. Limbs now run left and
            // right, and each is built from three shortening segments
            // that step backward in z - a real recurve profile rather
            // than one tilted slab, which is what makes the shape read as
            // SPRUNG rather than rigid.
            //
            // The riser stays at the origin so `weapon_hand_specs`' grip
            // socket (0, 0, 0.03) and the nock at the string's centre are
            // both untouched by the reorientation.
            parts.push(wp(false, Tone::Mid, (0.0, 0.0, 0.012), 0.0, (0.052, 0.115, 0.058)));
            // grip swell above and below the shelf, so the riser is not a
            // plain brick
            parts.push(wp(false, Tone::Dark, (0.0, 0.075, 0.012), 0.0, (0.040, 0.05, 0.050)));
            parts.push(wp(false, Tone::Dark, (0.0, -0.075, 0.012), 0.0, (0.040, 0.05, 0.050)));
            for side in [-1.0_f32, 1.0] {
                // three segments per limb: each shorter, thinner, and
                // further back than the last - the curve
                for (dx, dz, w, h, d) in [
                    (0.115, 0.000, 0.150, 0.030, 0.044),
                    (0.235, -0.030, 0.120, 0.026, 0.038),
                    (0.330, -0.072, 0.090, 0.022, 0.032),
                ] {
                    parts.push(wp(
                        false,
                        Tone::Dark,
                        (side * dx, 0.0, 0.012 + dz),
                        0.0,
                        (w, h, d),
                    ));
                }
                // the light tip / string nock at the end of the recurve
                parts.push(wp(
                    false,
                    Tone::Light,
                    (side * BOW_TIP_X, 0.0, BOW_TIP_Z),
                    0.0,
                    (0.040, 0.026, 0.028),
                ));
            }
            // The string is NOT in this list - it is two live halves hung
            // below, because a drawn string is a V and this table can only
            // describe a fixed pose. See `bow_string_sync`.
            //
            // The arrow rest moved with it. It used to be a shelf ON TOP of
            // the riser at y +0.030, left over from the vertical bow where
            // the arrow lay over the hand. Held horizontal there is no "on
            // top" - the shaft passes BESIDE the riser, so the rest is a
            // side bracket under the shaft's own line.
            parts.push(wd(
                false,
                Tone::Light,
                (BOW_ARROW_X, -0.016, 0.030),
                0.0,
                (0.030, 0.018, 0.062),
            ));
            parts.push(wd(false, Tone::Reticle, (0.0, 0.052, 0.044), 0.0, (0.006, 0.006, 0.006)));
        }
        GunKind::Spear => {
            // §owner: a JAVELIN, not a broomstick with a wedge on it. The
            // head is a leaf blade with a raised midrib and a socket that
            // swallows the shaft; the haft carries a bound grip where the
            // hand goes and a weighted butt that balances the throw.
            parts.push(wp(true, Tone::Dark, (0.0, 0.0, 0.35), FRAC_PI_2, (0.032, 1.85, 0.032)));
            // leaf blade: a wide belly tapering to a point, with the
            // midrib proud along its spine
            parts.push(wp(false, Tone::Light, (0.0, 0.0, 1.30), 0.0, (0.058, 0.016, 0.20)));
            parts.push(wp(false, Tone::Light, (0.0, 0.0, 1.42), 0.0, (0.030, 0.013, 0.10)));
            parts.push(wp(false, Tone::Mid, (0.0, 0.0, 1.30), 0.0, (0.014, 0.026, 0.19)));
            parts.push(wp(false, Tone::Light, (0.0, 0.0, 1.485), 0.0, (0.012, 0.010, 0.05)));
            // socket: the collar the blade seats into, with two rivets
            parts.push(wp(true, Tone::Black, (0.0, 0.0, 1.17), FRAC_PI_2, (0.046, 0.11, 0.046)));
            for rz in [1.135_f32, 1.205] {
                parts.push(wp(true, Tone::Mid, (0.026, 0.0, rz), FRAC_PI_2, (0.012, 0.012, 0.012)));
            }
            // bound grip where the hand sits - three cord wraps
            for gz in [-0.06_f32, 0.0, 0.06] {
                parts.push(wp(true, Tone::Mid, (0.0, 0.0, gz), FRAC_PI_2, (0.040, 0.030, 0.040)));
            }
            // weighted butt + a spike, the counterweight that makes a
            // javelin fly nose-first
            parts.push(wp(true, Tone::Black, (0.0, 0.0, -0.54), FRAC_PI_2, (0.044, 0.10, 0.044)));
            parts.push(wp(true, Tone::Mid, (0.0, 0.0, -0.62), FRAC_PI_2, (0.020, 0.07, 0.020)));
            parts.push(wd(false, Tone::Light, (0.0, 0.0, 1.10), 0.0, (0.045, 0.045, 0.02)));
        }
        GunKind::Minigun => {
            // §7: six-barrel cluster round a spine, deep motor housing at
            // the rear, spade grip below - no stock, the housing IS the
            // brace. Reads as MASS from every angle. The whole barrel
            // cluster lives on its own SPINNER child so the viewmodel can
            // rotate it with the sim's spin_t.
            // The cluster: spine, two end caps, six barrels. `ws` marks
            // them as SPINNER parts and the caller hangs them off the
            // rotating child - they were a hand-written spawn chain, which
            // is exactly what kept this table welded to `Commands`.
            parts.push(ws(true, Tone::Mid, (0.0, 0.0, 0.24), FRAC_PI_2, (0.024, 0.52, 0.024)));
            parts.push(ws(true, Tone::Black, (0.0, 0.0, 0.50), FRAC_PI_2, (0.082, 0.05, 0.082)));
            parts.push(ws(true, Tone::Black, (0.0, 0.0, 0.16), FRAC_PI_2, (0.086, 0.05, 0.086)));
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.0;
                let (bx, by) = (a.cos() * 0.052, a.sin() * 0.052);
                parts.push(ws(
                    true,
                    Tone::Dark,
                    (bx, by, 0.26),
                    FRAC_PI_2,
                    (0.017, 0.56, 0.017),
                ));
            }
            // motor housing + rear cap (the torso-brace face)
            parts.push(wp(false, Tone::Dark, (0.0, 0.0, -0.05), 0.0, (0.13, 0.15, 0.18)));
            parts.push(wp(false, Tone::Mid, (0.0, 0.0, -0.135), 0.0, (0.11, 0.13, 0.02)));
            // spade grip under the rear + side support handle
            parts.push(wp(false, Tone::Black, (0.0, -0.115, -0.07), 0.15, (0.03, 0.09, 0.04)));
            parts.push(wp(false, Tone::Black, (-0.02, -0.10, 0.18), 0.0, (0.026, 0.10, 0.035)));
            // feed chute hint on the right flank
            parts.push(wd(false, Tone::Light, (0.075, -0.02, -0.02), 0.0, (0.02, 0.06, 0.10)));
            // §owner: the minigun had NO sights of any kind - it fell to
            // the generic ADS shift and the player aimed by tracer alone.
            // The optic sits above the motor housing (top y 0.075).
            push_red_dot(&mut parts, 0.1120, -0.05, 0.075); // motor housing 0.075
        }
    }
    parts
}

/// Build one weapon model out of `weapon_parts`. Every gun
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
    let parts = weapon_parts(kind);
    // The spinner is created LAZILY, only if some part asked for one, so
    // no weapon ends up carrying an empty child node it never rotates.
    let spinner = parts.iter().any(|p| p.spin).then(|| {
        let e = commands
            .spawn((Transform::IDENTITY, Visibility::default()))
            .id();
        if with_hands {
            commands.entity(e).insert(MinigunSpinner);
        }
        commands.entity(e).set_parent(root);
        e
    });
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
        if p.tone == Tone::Reticle {
            // the dot is driven per-frame, so it needs to be findable
            // and it needs its rest pose remembered
            e.insert(ReticleDot { rest: p.pos });
        }
        // a spinner part hangs off the rotating child; everything else
        // off the model root
        e.set_parent(if p.spin { spinner.unwrap_or(root) } else { root });
    }
    // The bow's LIVE parts: two string halves and the arrow on them.
    //
    // Spawned here rather than pushed as `WPart`s because both move every
    // frame, and spawned for BOTH views deliberately. The viewmodel bow
    // never had a nocked arrow at all - a first-person archer drew an empty
    // string - and the reason is visible in the capture scripts: the
    // third-person script cannot see the viewmodel, and until `bow_draw_fp`
    // existed nothing had ever looked at it.
    if kind == GunKind::Bow {
        commands.entity(root).insert(BowDraw { pull: 0.0, nocked: true });
        for side in [-1.0_f32, 1.0] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.tone(Tone::Light)),
                    bow_string_half(side, 0.0),
                    BowStringHalf(side),
                ))
                .set_parent(root);
        }
        let (arrow, _spin) = spawn_arrow_model(commands, kit);
        commands
            .entity(arrow)
            .insert((bow_nocked_arrow(0.0), NockedArrow))
            .set_parent(root);
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
/// gold trim - held on the left arm when raised (slot 4).
/// `see_through` = the FIRST-PERSON copy: translucent materials so the
/// raised plate guards without blinding - the world reads through it.
/// Third-person shields (yours and every enemy's) stay opaque.
fn spawn_shield_model(commands: &mut Commands, kit: &ModelKit, see_through: bool) -> Entity {
    let plate = if see_through {
        kit.vm_shield_dark.clone()
    } else {
        kit.armor_dark.clone()
    };
    let metal_m = if see_through {
        kit.vm_shield_steel.clone()
    } else {
        kit.steel.clone()
    };
    let trim = if see_through {
        kit.vm_shield_gold.clone()
    } else {
        kit.gold.clone()
    };
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // three gently angled slats fake a curved plate
    for (x, ry) in [(-0.16_f32, 0.28_f32), (0.0, 0.0), (0.16, -0.28)] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(plate.clone()),
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
            MeshMaterial3d(metal_m.clone()),
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
                MeshMaterial3d(trim.clone()),
                Transform::from_xyz(0.0, y, 0.0).with_scale(Vec3::new(0.46, 0.035, 0.05)),
            ))
            .set_parent(root);
    }
    // handle bar + a gripping fist behind the plate
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(metal_m),
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

/// §owner MECH BARRIER: the arm module that projects the field, and the
/// field itself.
#[derive(Component)]
struct MechBarrier {
    /// The folding emitter arms - three panels that lie along the
    /// forearm stowed and swing out to frame the field.
    petals: [Entity; 3],
    /// The projected field: fill sheet plus the hex cell edges.
    field: Entity,
}

/// How far the emitter petals swing out when the barrier deploys.
const BARRIER_PETAL_DEG: f32 = 62.0;
/// Deploy/stow time. Fast - the brief asks for a shield that "deploys
/// quickly during combat", and a barrier you have to plan half a second
/// ahead of is one you die behind.
const BARRIER_DEPLOY_S: f32 = 0.18;

/// Build the arm-mounted barrier projector.
///
/// It is a MODULE on the forearm, not a plate held in front of it: the
/// brief asks for something that "looks integrated into the mech rather
/// than attached afterward", and the difference is whether the thing has
/// a housing that belongs to the arm it rides on. So: a boxed emitter
/// bolted along the forearm cradle, three petals that lie flat against
/// it when stowed, and the field projected from between them.
///
/// The FIELD is built as a fill sheet plus a hexagonal lattice of edge
/// bars. That is the whole trick behind "transparent to the pilot,
/// visible to the enemy": the fill is at 8% alpha and unlit, so it is
/// very nearly a window, while the lattice is bright and emissive and
/// reads from outside as a wall of light. One translucent sheet cannot
/// be both things at once.
fn spawn_mech_barrier(commands: &mut Commands, kit: &ModelKit) -> (Entity, MechBarrier) {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // EMITTER HOUSING - reads as part of the forearm
    for (mat, pos, sc) in [
        (kit.mech_khaki.clone(), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.20, 0.14, 0.42)),
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.085, 0.0), Vec3::new(0.17, 0.035, 0.38)),
        (kit.mech_shadow.clone(), Vec3::new(0.0, -0.075, 0.0), Vec3::new(0.16, 0.030, 0.36)),
    ] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(pos).with_scale(sc),
            ))
            .set_parent(root);
    }
    // the projector lens the field grows out of, and its capacitor ring
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.mech_metal.clone()),
            Transform {
                translation: Vec3::new(0.0, 0.0, 0.215),
                rotation: Quat::from_rotation_x(FRAC_PI_2),
                scale: Vec3::new(0.155, 0.040, 0.155),
            },
        ))
        .set_parent(root);
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.barrier_edge.clone()),
            Transform {
                translation: Vec3::new(0.0, 0.0, 0.238),
                rotation: Quat::from_rotation_x(FRAC_PI_2),
                scale: Vec3::new(0.105, 0.016, 0.105),
            },
        ))
        .set_parent(root);

    // THE PETALS - three folding frames. Stowed they lie along the
    // housing; deployed they splay to carry the field's corners.
    let mut petals = [Entity::PLACEHOLDER; 3];
    for (i, roll) in [0.0_f32, 2.094, -2.094].into_iter().enumerate() {
        let petal = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.22)
                    .with_rotation(Quat::from_rotation_z(roll)),
                Visibility::default(),
            ))
            .set_parent(root)
            .id();
        // the arm, plus a lit strip running its length
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_khaki_dk.clone()),
                Transform::from_xyz(0.0, 0.20, 0.0)
                    .with_scale(Vec3::new(0.055, 0.42, 0.045)),
            ))
            .set_parent(petal);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.barrier_edge.clone()),
                Transform::from_xyz(0.0, 0.20, 0.026)
                    .with_scale(Vec3::new(0.016, 0.36, 0.012)),
            ))
            .set_parent(petal);
        // the tip node that anchors a corner of the field
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform::from_xyz(0.0, 0.41, 0.0)
                    .with_scale(Vec3::splat(0.055)),
            ))
            .set_parent(petal);
        petals[i] = petal;
    }

    // THE FIELD - a group so one Visibility and one scale animate it all
    let field = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.30),
            Visibility::Hidden,
        ))
        .set_parent(root)
        .id();
    // the fill sheet: a thin disc, near-invisible, double-sided
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.barrier_fill.clone()),
            Transform {
                translation: Vec3::ZERO,
                rotation: Quat::from_rotation_x(FRAC_PI_2),
                scale: Vec3::new(1.70, 0.012, 1.70),
            },
        ))
        .set_parent(field);
    // the HEX LATTICE. Cells on an axial grid, each drawn as six short
    // bars - so the pattern is real geometry that catches the light and
    // occludes correctly, not a texture that would go flat edge-on.
    //
    // Only cells whose centre falls inside the field radius are drawn,
    // which is what gives the barrier a hexagonal rim rather than a
    // circle with a grid stamped on it.
    const CELL: f32 = 0.235;
    for q in -4i32..=4 {
        for r in -4i32..=4 {
            let cx = CELL * 1.5 * q as f32;
            let cy = CELL * (3.0_f32).sqrt() * (r as f32 + q as f32 * 0.5);
            if (cx * cx + cy * cy).sqrt() > 0.80 {
                continue;
            }
            for k in 0..6 {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                let na = (k + 1) as f32 * std::f32::consts::TAU / 6.0;
                let (x0, y0) = (cx + a.cos() * CELL * 0.58, cy + a.sin() * CELL * 0.58);
                let (x1, y1) = (cx + na.cos() * CELL * 0.58, cy + na.sin() * CELL * 0.58);
                let (mx, my) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
                let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
                commands
                    .spawn((
                        Mesh3d(kit.cube.clone()),
                        MeshMaterial3d(kit.barrier_edge.clone()),
                        Transform {
                            translation: Vec3::new(mx, my, 0.0),
                            rotation: Quat::from_rotation_z((y1 - y0).atan2(x1 - x0)),
                            scale: Vec3::new(len, 0.010, 0.008),
                        },
                    ))
                    .set_parent(field);
            }
        }
    }
    (root, MechBarrier { petals, field })
}

/// §owner MECH BARRIER: fold, deploy, and ripple.
///
/// The deploy is a single 0..1 clock driving three things at once - the
/// petals swinging out, the field scaling up from nothing, and its
/// visibility - so the barrier cannot get into a state where the frame
/// is open and the field is missing.
///
/// The RIPPLE is a scale wobble on the field group rather than a shader:
/// this build has no custom material pipeline, and a breathing lattice
/// reads as energy from every angle a shader would have. Two frequencies
/// that do not share a period, the same trick the mech's idle tremor
/// uses, so it never resolves into a single pulse.
fn mech_barrier_sync(
    time: Res<Time>,
    game: Res<Game>,
    rigs: Query<(&FighterVis, &MechBarrier)>,
    mut parts: Query<(&mut Transform, &mut Visibility)>,
) {
    let dt = time.delta_secs();
    let tnow = time.elapsed_secs();
    for (vis, bar) in &rigs {
        let Some(f) = game.sim.fighters.get(vis.index) else { continue };
        // up only in a chassis, with the pool still standing
        let want = f.in_mech() && f.shield_up && f.mech_shield_hp > 0.0;
        // one clock, read back off the field's own scale so no extra
        // per-fighter state is needed for a purely cosmetic animation
        let cur = parts
            .get(bar.field)
            .map(|(t, _)| t.scale.x.clamp(0.0, 1.0))
            .unwrap_or(0.0);
        let step = dt / BARRIER_DEPLOY_S;
        let e = if want { (cur + step).min(1.0) } else { (cur - step).max(0.0) };
        for p in bar.petals {
            if let Ok((mut t, _)) = parts.get_mut(p) {
                let roll = t.rotation.to_euler(EulerRot::ZYX).0;
                t.rotation = Quat::from_rotation_z(roll)
                    * Quat::from_rotation_x(-BARRIER_PETAL_DEG.to_radians() * e);
            }
        }
        if let Ok((mut t, mut v)) = parts.get_mut(bar.field) {
            // the ripple - only once the field is actually up, so a
            // stowed barrier is perfectly still
            let ripple = if e > 0.99 {
                1.0 + (tnow * 5.3).sin() * 0.012 + (tnow * 8.7).sin() * 0.007
            } else {
                1.0
            };
            t.scale = Vec3::splat(e * ripple);
            *v = if e > 0.01 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// §C: the TURRET mount's viewmodel barrel cluster - spun by the mount's
/// own trigger/vent state (`spin_mech_turret_barrels`), never the
/// carried minigun's `spin_t`.
#[derive(Component)]
struct MechTurretSpinner;

/// §C.7: the TURRET hull-mount viewmodel - a gatling barrel cluster in
/// the mech palette. No hands, no forearms, no ammo bar: a hull mount is
/// STRUCTURAL, never held (sim.rs MechWeapon doctrine).
fn spawn_mech_turret_vm(commands: &mut Commands, kit: &ModelKit) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    let spinner = commands
        .spawn((Transform::IDENTITY, Visibility::default(), MechTurretSpinner))
        .set_parent(root)
        .id();
    let mut cluster: Vec<(Handle<StandardMaterial>, Vec3, Vec3)> = vec![
        // central spine + two collar discs
        (kit.mech_metal.clone(), Vec3::new(0.0, 0.0, 0.30), Vec3::new(0.030, 0.62, 0.030)),
        (kit.grey_black.clone(), Vec3::new(0.0, 0.0, 0.60), Vec3::new(0.100, 0.06, 0.100)),
        (kit.grey_black.clone(), Vec3::new(0.0, 0.0, 0.20), Vec3::new(0.105, 0.06, 0.105)),
    ];
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.0;
        cluster.push((
            kit.grey_dark.clone(),
            Vec3::new(a.cos() * 0.064, a.sin() * 0.064, 0.32),
            Vec3::new(0.022, 0.68, 0.022),
        ));
    }
    for (mat, pos, size) in cluster {
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(mat),
                Transform {
                    translation: pos,
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: size,
                },
            ))
            .set_parent(spinner);
    }
    // hull housing: khaki plate + dark cradle + one hazard strip
    for (mat, pos, size) in [
        (kit.mech_khaki.clone(), Vec3::new(0.0, -0.02, -0.10), Vec3::new(0.20, 0.16, 0.26)),
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, -0.11, 0.06), Vec3::new(0.14, 0.05, 0.30)),
        // a narrow front stripe, NOT a deck: mech_hazard is capped at
        // an accent (<=10% of surface), and a full-footprint yellow top
        // plate was the single loudest thing on the screen
        (kit.mech_hazard.clone(), Vec3::new(0.0, 0.081, -0.02), Vec3::new(0.20, 0.012, 0.05)),
    ] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(mat),
                Transform {
                    translation: pos,
                    scale: size,
                    ..default()
                },
            ))
            .set_parent(root);
    }
    // ---- §owner FLAGSHIP PASS: the turret ------------------------------
    //
    // This is the weapon the player looks at for the whole time they are
    // in a chassis, and it was a spine, two discs and six tubes. A
    // rotary cannon is a SYSTEM: barrels, the clamps that hold them true,
    // the drive that turns them, the belt that feeds them, and the heat
    // it all makes. Each of those is a group below, and each of them is
    // the reason the next one exists.
    {
        // BARREL CLAMPS - two rings around the cluster, and the reason
        // the barrels look like an assembly instead of six loose rods.
        // On the SPINNER, so they turn with what they are clamping.
        for (rz, r) in [(0.16_f32, 0.095_f32), (0.50, 0.088)] {
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(kit.mech_khaki_dk.clone()),
                    Transform {
                        translation: Vec3::new(0.0, 0.0, rz),
                        rotation: Quat::from_rotation_x(FRAC_PI_2),
                        scale: Vec3::new(r * 2.0, 0.045, r * 2.0),
                    },
                ))
                .set_parent(spinner);
        }
        // muzzle collars, one per barrel - a bored gun has a THICKER
        // mouth, and six of them catch the light as the cluster turns
        for i in 0..6 {
            let a = i as f32 * std::f32::consts::TAU / 6.0;
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform {
                        translation: Vec3::new(a.cos() * 0.064, a.sin() * 0.064, 0.645),
                        rotation: Quat::from_rotation_x(FRAC_PI_2),
                        scale: Vec3::new(0.034, 0.035, 0.034),
                    },
                ))
                .set_parent(spinner);
        }
        // DRIVE HOUSING - the motor that turns the cluster, static, with
        // a cooling jacket and a lit power feed. Static on purpose: the
        // drive does not spin, the thing it drives does, and that
        // difference is most of what makes a gatling read correctly.
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_khaki_dk.clone()),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.03),
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: Vec3::new(0.25, 0.20, 0.25),
                },
            ))
            .set_parent(root);
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.0;
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform {
                        translation: Vec3::new(a.cos() * 0.132, a.sin() * 0.132, 0.03),
                        rotation: Quat::from_rotation_z(a),
                        scale: Vec3::new(0.030, 0.055, 0.19),
                    },
                ))
                .set_parent(root);
        }
        // HEAT SINK stack over the drive - fins, and a glowing seam
        // between them. The mount already HAS a heat model the HUD
        // shows; this is where that number lives on the gun.
        for k in 0..5 {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform::from_xyz(0.0, 0.115, -0.02 + k as f32 * 0.042)
                        .with_scale(Vec3::new(0.19, 0.075, 0.016)),
                ))
                .set_parent(root);
        }
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.med_glow.clone()),
                Transform::from_xyz(0.0, 0.075, 0.065)
                    .with_scale(Vec3::new(0.17, 0.012, 0.20)),
            ))
            .set_parent(root);
        // AMMUNITION - a belt box under the breech and the links running
        // up into it. Gold, matching the ammo-pickup vocabulary, so a
        // player reads "rounds" without being told.
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_khaki_dk.clone()),
                Transform::from_xyz(0.0, -0.20, -0.16)
                    .with_scale(Vec3::new(0.26, 0.20, 0.30)),
            ))
            .set_parent(root);
        for (lx, ly, lz) in [
            (0.0_f32, -0.115_f32, -0.11_f32),
            (0.0, -0.075, -0.055),
            (0.0, -0.055, 0.00),
        ] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.gold.clone()),
                    Transform::from_xyz(lx, ly, lz)
                        .with_scale(Vec3::new(0.10, 0.045, 0.075)),
                ))
                .set_parent(root);
        }
        // CABLING from the drive back into the hull, and the two small
        // status lamps a crewed weapon always has
        for sd in [-1.0_f32, 1.0] {
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(kit.mech_shadow.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.13, -0.09, -0.20),
                        rotation: Quat::from_rotation_x(0.55),
                        scale: Vec3::new(0.032, 0.28, 0.032),
                    },
                ))
                .set_parent(root);
            commands
                .spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(kit.core_glow.clone()),
                    Transform::from_xyz(sd * 0.115, 0.055, -0.14)
                        .with_scale(Vec3::splat(0.024)),
                ))
                .set_parent(root);
        }
        // reinforced barrel shroud - a partial cowl over the top of the
        // cluster, open below so the barrels stay readable
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_khaki.clone()),
                Transform::from_xyz(0.0, 0.105, 0.34)
                    .with_scale(Vec3::new(0.20, 0.030, 0.42)),
            ))
            .set_parent(root);
        for sd in [-1.0_f32, 1.0] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.mech_khaki.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.105, 0.055, 0.34),
                        rotation: Quat::from_rotation_z(sd * 0.55),
                        scale: Vec3::new(0.028, 0.11, 0.42),
                    },
                ))
                .set_parent(root);
        }
    }
    root
}

/// §C.7: the ROCKETS hull-mount viewmodel - a LAUNCH TUBE carried
/// forward over the mount, not a box.
///
/// The first draft was a boxy pod with a 3x2 bored face, which is what
/// the hull's own pod looks like from outside. In first person you view
/// it from BEHIND, so the tube face - the entire read - pointed away
/// and the player got a featureless slab with a yellow deck. A tube
/// receding to a visible muzzle ring reads as "launcher" from the one
/// angle a pilot actually has.
fn spawn_mech_pod_vm(commands: &mut Commands, kit: &ModelKit) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // the tube itself, running forward (local +Z; the root's PI yaw puts
    // that down the view axis), plus its muzzle collar and dark bore
    for (mat, pos, sc) in [
        (kit.mech_khaki.clone(), Vec3::new(0.0, 0.0, 0.45), Vec3::new(0.150, 0.90, 0.150)),
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.0, 0.88), Vec3::new(0.178, 0.05, 0.178)),
        (kit.grey_black.clone(), Vec3::new(0.0, 0.0, 0.90), Vec3::new(0.120, 0.03, 0.120)),
        // a hazard BAND around the tube - a ring, not a painted deck
        (kit.mech_hazard.clone(), Vec3::new(0.0, 0.0, 0.24), Vec3::new(0.163, 0.035, 0.163)),
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.0, 0.10), Vec3::new(0.172, 0.06, 0.172)),
    ] {
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(mat),
                Transform {
                    translation: pos,
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: sc,
                },
            ))
            .set_parent(root);
    }
    // the reload magazine slung under the rear, and a grip block
    for (mat, pos, sc) in [
        (kit.mech_khaki_dk.clone(), Vec3::new(0.0, -0.13, 0.16), Vec3::new(0.17, 0.14, 0.30)),
        (kit.mech_metal.clone(), Vec3::new(0.0, -0.115, 0.44), Vec3::new(0.06, 0.12, 0.09)),
    ] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(mat),
                Transform {
                    translation: pos,
                    scale: sc,
                    ..default()
                },
            ))
            .set_parent(root);
    }
    // two spare rounds visible in the magazine - the tube alone reads a
    // little empty, and these sell "10 in the pod"
    for x in [-0.045_f32, 0.045] {
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform {
                    translation: Vec3::new(x, -0.13, 0.30),
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: Vec3::new(0.05, 0.10, 0.05),
                },
            ))
            .set_parent(root);
    }
    // ---- §owner FLAGSHIP PASS: the launcher -----------------------------
    //
    // A launcher is a tube plus everything that makes firing one safe:
    // the casing that contains a misfire, the lock that holds the round
    // until it is told not to, the vent that puts the backblast
    // somewhere, and the sensor that decided to fire in the first place.
    // The tube had none of them.
    {
        // ARMOUR CASING - a half-shell over the tube, ribbed, open below
        // so the tube stays the thing you read
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_khaki.clone()),
                Transform::from_xyz(0.0, 0.115, 0.42)
                    .with_scale(Vec3::new(0.20, 0.045, 0.78)),
            ))
            .set_parent(root);
        for k in 0..5 {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.mech_khaki_dk.clone()),
                    Transform::from_xyz(0.0, 0.115, 0.13 + k as f32 * 0.145)
                        .with_scale(Vec3::new(0.215, 0.055, 0.030)),
                ))
                .set_parent(root);
        }
        for sd in [-1.0_f32, 1.0] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.mech_khaki.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.115, 0.055, 0.42),
                        rotation: Quat::from_rotation_z(sd * 0.60),
                        scale: Vec3::new(0.030, 0.10, 0.78),
                    },
                ))
                .set_parent(root);
        }
        // LOCKING MECHANISM at the mouth - two jaws and their actuator.
        // This is the part that says the round is HELD rather than
        // resting in a pipe.
        for sd in [-1.0_f32, 1.0] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.155, 0.0, 0.815),
                        rotation: Quat::from_rotation_z(sd * -0.25),
                        scale: Vec3::new(0.045, 0.13, 0.075),
                    },
                ))
                .set_parent(root);
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(kit.mech_shadow.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.145, 0.0, 0.735),
                        rotation: Quat::from_rotation_z(FRAC_PI_2),
                        scale: Vec3::new(0.040, 0.055, 0.040),
                    },
                ))
                .set_parent(root);
        }
        // EXHAUST / BACKBLAST vents at the rear, angled out and down so
        // the blast is visibly going somewhere that is not the pilot
        for sd in [-1.0_f32, 1.0] {
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(kit.mech_khaki_dk.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.115, -0.035, -0.06),
                        rotation: Quat::from_rotation_z(sd * -0.55),
                        scale: Vec3::new(0.085, 0.16, 0.085),
                    },
                ))
                .set_parent(root);
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(kit.grey_black.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.155, -0.105, -0.06),
                        rotation: Quat::from_rotation_z(sd * -0.55),
                        scale: Vec3::new(0.065, 0.03, 0.065),
                    },
                ))
                .set_parent(root);
        }
        // RELOAD components - the feed arm that lifts a round from the
        // magazine into the breech, caught mid-travel
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform {
                    translation: Vec3::new(0.0, -0.075, 0.22),
                    rotation: Quat::from_rotation_x(-0.45),
                    scale: Vec3::new(0.075, 0.14, 0.045),
                },
            ))
            .set_parent(root);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_shadow.clone()),
                Transform {
                    translation: Vec3::new(0.0, -0.135, 0.10),
                    rotation: Quat::from_rotation_z(FRAC_PI_2),
                    scale: Vec3::new(0.055, 0.18, 0.055),
                },
            ))
            .set_parent(root);
        // TARGETING SENSOR over the muzzle - a boxed seeker head with a
        // lit aperture. The pod already LOCKS (`pod_lock_t`); this is
        // where that lives on the model.
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_khaki_dk.clone()),
                Transform::from_xyz(0.0, 0.175, 0.70)
                    .with_scale(Vec3::new(0.13, 0.085, 0.17)),
            ))
            .set_parent(root);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.core_glow.clone()),
                Transform {
                    translation: Vec3::new(0.0, 0.175, 0.79),
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: Vec3::new(0.055, 0.014, 0.055),
                },
            ))
            .set_parent(root);
        // and the cable run from the seeker back into the mount
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_shadow.clone()),
                Transform {
                    translation: Vec3::new(0.075, 0.15, 0.40),
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: Vec3::new(0.026, 0.60, 0.026),
                },
            ))
            .set_parent(root);
    }
    root
}

/// §C: the turret viewmodel's barrels spin with the MOUNT's own state -
/// crawl at idle, spin-up while the trigger is held, dead while venting.
fn spin_mech_turret_barrels(
    game: Res<Game>,
    time: Res<Time>,
    mut q: Query<&mut Transform, With<MechTurretSpinner>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let rate = if !p.in_mech() || p.gatling_vent_t > 0.0 {
        0.0
    } else if p.gatling_trigger_t > 0.0 {
        MINIGUN_SPIN_FULL_RAD_S * 0.75
    } else {
        MINIGUN_IDLE_CRAWL_RAD_S
    };
    if rate <= 0.0 {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    for mut tf in &mut q {
        tf.rotation = Quat::from_rotation_z(rate * dt) * tf.rotation;
    }
}

/// §6.3 / D.6: hull-side parts that visually shear off at HP thresholds.
struct MechHullDetach {
    skirt_l: Entity,
    skirt_r: Entity,
    drum_r: Entity,
    antenna: Entity,
}

/// Leg-bone-parented mech armour for one side.
struct MechLegArmor {
    roots: [Entity; 3],
    thigh_plate: Entity,
    shin_plate: Entity,
    cleat_front: Entity,
}

/// Spawn the mech hull rig - a WALKING WEAPONS PLATFORM: a slab hull
/// cantilevered over an exposed hip/waist mechanism, a sensor-visor deck
/// instead of a head, a 10-tube rocket pod on the left hardpoint and a
/// gatling barrel cluster hung low on the right flank (Brief VIII-B
/// D.1-D.5). Torso-local coordinates; the head hit band (>0.82 of
/// height, torso-local y > 0.846) is filled by the sensor deck, and the
/// emissive visor slit sits inside the visor-mult band - so the sim's
/// damage model is untouched by construction.
fn spawn_armor_rig(commands: &mut Commands, kit: &ModelKit) -> (Entity, MechHullDetach) {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    let cube = || kit.cube.clone();
    let cyl = || kit.cyl.clone();
    let ball = || kit.ball.clone();
    // (mesh, material, translation, rotation, scale) - torso-local
    let plates: [(Handle<Mesh>, Handle<StandardMaterial>, Vec3, Quat, Vec3); 53] = [
        // ---- HULL: a slab wider than tall, over the legs (D.1/D.4) ----
        (cube(), kit.mech_khaki.clone(), Vec3::new(0.0, 0.50, 0.08), Quat::IDENTITY, Vec3::new(1.06, 0.44, 0.92)),
        (cube(), kit.mech_khaki.clone(), Vec3::new(0.0, 0.665, 0.44), Quat::from_rotation_x(0.55), Vec3::new(0.94, 0.04, 0.30)),
        (cube(), kit.mech_shadow.clone(), Vec3::new(0.0, 0.52, 0.545), Quat::IDENTITY, Vec3::new(0.56, 0.18, 0.03)),
        (cube(), kit.mech_khaki_lt.clone(), Vec3::new(0.35, 0.55, 0.548), Quat::IDENTITY, Vec3::new(0.11, 0.15, 0.012)),
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(0.42, 0.745, 0.14), Quat::IDENTITY, Vec3::new(0.16, 0.03, 0.22)),
        (cube(), kit.mech_shadow.clone(), Vec3::new(0.0, 0.272, 0.08), Quat::IDENTITY, Vec3::new(1.00, 0.02, 0.86)),
        // two-tone break-up: a lighter deck plate + a dark chin line -
        // paint, not geometry, is what stops the slab reading flat
        (cube(), kit.mech_khaki_lt.clone(), Vec3::new(0.0, 0.723, -0.06), Quat::IDENTITY, Vec3::new(0.58, 0.010, 0.50)),
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.315, 0.548), Quat::IDENTITY, Vec3::new(1.00, 0.07, 0.010)),
        // ---- SENSOR DECK: the "no head" head - fills the >0.82 band ----
        (cube(), kit.mech_khaki.clone(), Vec3::new(0.0, 0.88, 0.02), Quat::IDENTITY, Vec3::new(0.62, 0.32, 0.54)),
        (cube(), kit.mech_shadow.clone(), Vec3::new(0.0, 0.89, 0.297), Quat::IDENTITY, Vec3::new(0.48, 0.17, 0.012)),
        // the SENSOR VISOR strip - a thin lens line, not a lightbar
        (cube(), kit.mech_red.clone(), Vec3::new(0.0, 0.945, 0.308), Quat::IDENTITY, Vec3::new(0.40, 0.032, 0.02)),
        // brow hood over the slit + cheek blocks framing the recess -
        // NO extra mech_red anywhere: one slit is the x2 weak-point read
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.99, 0.30), Quat::from_rotation_x(-0.20), Vec3::new(0.50, 0.025, 0.12)),
        (cube(), kit.mech_khaki_lt.clone(), Vec3::new(-0.27, 0.89, 0.295), Quat::IDENTITY, Vec3::new(0.06, 0.17, 0.015)),
        (cube(), kit.mech_khaki_lt.clone(), Vec3::new(0.27, 0.89, 0.295), Quat::IDENTITY, Vec3::new(0.06, 0.17, 0.015)),
        // ---- REAR: comms/cooling drum, LEFT (right one is 40%-tagged) --
        (cyl(), kit.mech_khaki_dk.clone(), Vec3::new(-0.28, 0.86, -0.38), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.22, 0.34, 0.22)),
        (cyl(), kit.mech_metal.clone(), Vec3::new(-0.28, 0.86, -0.30), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.235, 0.02, 0.235)),
        (cyl(), kit.mech_metal.clone(), Vec3::new(-0.28, 0.86, -0.46), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.235, 0.02, 0.235)),
        // ---- ANTENNAS: whip base; sensor stalk + ball tip, LEFT --------
        (cube(), kit.mech_metal.clone(), Vec3::new(0.42, 0.76, -0.30), Quat::IDENTITY, Vec3::new(0.05, 0.08, 0.05)),
        (cyl(), kit.mech_metal.clone(), Vec3::new(-0.24, 1.10, -0.18), Quat::IDENTITY, Vec3::new(0.016, 0.14, 0.016)),
        (kit.ball.clone(), kit.mech_metal.clone(), Vec3::new(-0.24, 1.19, -0.18), Quat::IDENTITY, Vec3::new(0.05, 0.05, 0.05)),
        // ---- HIP & WAIST: the mechanism stays EXPOSED (Task 5.4) -------
        (cyl(), kit.mech_metal.clone(), Vec3::new(0.0, 0.005, 0.02), Quat::IDENTITY, Vec3::new(0.44, 0.05, 0.40)),
        // kept from the old rig, verbatim:
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(0.0, 0.10, 0.05), Quat::IDENTITY, Vec3::new(0.40, 0.14, 0.22)),
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(0.0, -0.02, 0.10), Quat::from_rotation_x(-0.25), Vec3::new(0.34, 0.12, 0.10)),
        (cube(), kit.mech_metal.clone(), Vec3::new(0.0, 0.02, 0.18), Quat::from_rotation_x(0.15), Vec3::new(0.14, 0.06, 0.06)),
        (cube(), kit.mech_shadow.clone(), Vec3::new(0.09, -0.01, 0.19), Quat::from_rotation_z(0.4), Vec3::new(0.03, 0.09, 0.03)),
        (cube(), kit.mech_metal.clone(), Vec3::new(0.09, 0.05, 0.16), Quat::from_rotation_z(0.3), Vec3::new(0.08, 0.05, 0.01)),
        // ---- SHOULDER HOUSINGS: low on the flanks, never shoulder-top --
        (cube(), kit.mech_khaki.clone(), Vec3::new(-0.60, 0.42, 0.06), Quat::IDENTITY, Vec3::new(0.20, 0.28, 0.42)),
        (cube(), kit.mech_khaki.clone(), Vec3::new(0.60, 0.42, 0.06), Quat::IDENTITY, Vec3::new(0.20, 0.28, 0.42)),
        (cube(), kit.mech_shadow.clone(), Vec3::new(-0.60, 0.25, 0.06), Quat::IDENTITY, Vec3::new(0.16, 0.08, 0.36)),
        (cube(), kit.mech_shadow.clone(), Vec3::new(0.60, 0.25, 0.06), Quat::IDENTITY, Vec3::new(0.16, 0.08, 0.36)),
        // ---- ROCKET POD, left hardpoint: rail + box + 10-tube face -----
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(-0.44, 0.735, 0.02), Quat::IDENTITY, Vec3::new(0.28, 0.04, 0.34)),
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(-0.44, 0.855, 0.02), Quat::IDENTITY, Vec3::new(0.34, 0.23, 0.42)),
        // recessed dark face so the tubes read as BORED openings
        (cube(), kit.mech_shadow.clone(), Vec3::new(-0.44, 0.855, 0.232), Quat::IDENTITY, Vec3::new(0.32, 0.20, 0.008)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.575, 0.895, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.508, 0.895, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.440, 0.895, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.372, 0.895, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.305, 0.895, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.575, 0.815, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.508, 0.815, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.440, 0.815, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.372, 0.815, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cyl(), kit.mech_shadow.clone(), Vec3::new(-0.305, 0.815, 0.235), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.055, 0.014, 0.055)),
        (cube(), kit.mech_red.clone(), Vec3::new(-0.44, 0.755, 0.235), Quat::IDENTITY, Vec3::new(0.30, 0.012, 0.01)),
        (cube(), kit.mech_hazard.clone(), Vec3::new(-0.44, 0.965, 0.225), Quat::IDENTITY, Vec3::new(0.30, 0.016, 0.014)),
        // ---- GATLING ARM, right flank: hangs LOW and FORWARD -----------
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(0.60, 0.24, 0.14), Quat::IDENTITY, Vec3::new(0.20, 0.20, 0.30)),
        (cube(), kit.mech_metal.clone(), Vec3::new(0.60, 0.35, 0.10), Quat::IDENTITY, Vec3::new(0.09, 0.12, 0.09)),
        (cyl(), kit.mech_metal.clone(), Vec3::new(0.60, 0.24, 0.40), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.19, 0.24, 0.19)),
        (cyl(), kit.mech_metal.clone(), Vec3::new(0.60, 0.24, 0.83), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.14, 0.03, 0.14)),
        (cube(), kit.mech_khaki_dk.clone(), Vec3::new(0.60, 0.06, 0.12), Quat::IDENTITY, Vec3::new(0.15, 0.14, 0.24)),
        (cube(), kit.mech_shadow.clone(), Vec3::new(0.51, 0.15, 0.05), Quat::from_rotation_z(0.55), Vec3::new(0.14, 0.06, 0.10)),
        // ---- UNIT STENCILS: faded parchment ident plates ---------------
        (cube(), kit.mech_stencil.clone(), Vec3::new(-0.40, 0.62, 0.548), Quat::IDENTITY, Vec3::new(0.10, 0.045, 0.008)),
        (cube(), kit.mech_stencil.clone(), Vec3::new(-0.615, 0.80, 0.10), Quat::IDENTITY, Vec3::new(0.008, 0.05, 0.12)),
    ];
    for (mesh, mat, tr, rot, sc) in plates {
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform { translation: tr, rotation: rot, scale: sc },
            ))
            .set_parent(root);
    }
    // 6 gatling barrels in a hex ring about (0.60, 0.24), plus a hazard
    // band - generated, not typed six times
    for k in 0..6 {
        let ang = k as f32 * std::f32::consts::TAU / 6.0;
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform {
                    translation: Vec3::new(
                        0.60 + 0.052 * ang.cos(),
                        0.24 + 0.045 * ang.sin(),
                        0.64,
                    ),
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: Vec3::new(0.03, 0.44, 0.03),
                },
            ))
            .set_parent(root);
    }
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.mech_hazard.clone()),
            Transform {
                translation: Vec3::new(0.60, 0.135, 0.245),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(0.13, 0.014, 0.012),
            },
        ))
        .set_parent(root);
    // spine heat-sink fins between the drums - the rear deck gets a
    // machine read of its own instead of a bare khaki roof
    for k in -2i32..=2 {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform {
                    translation: Vec3::new(k as f32 * 0.055, 0.78, -0.36),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(0.018, 0.14, 0.14),
                },
            ))
            .set_parent(root);
    }
    // per-side dressing: exhaust stack sunk into each shoulder top,
    // pauldron edge trim + corner bolts, and the waist support pistons
    // that sell the hull's cantilever over the hip ring
    for sd in [-1.0_f32, 1.0] {
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform::from_xyz(sd * 0.60, 0.60, -0.14)
                    .with_scale(Vec3::new(0.075, 0.18, 0.075)),
            ))
            .set_parent(root);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_shadow.clone()),
                Transform::from_xyz(sd * 0.60, 0.695, -0.14)
                    .with_scale(Vec3::new(0.055, 0.012, 0.055)),
            ))
            .set_parent(root);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.mech_khaki_lt.clone()),
                Transform::from_xyz(sd * 0.705, 0.535, 0.06)
                    .with_scale(Vec3::new(0.012, 0.035, 0.38)),
            ))
            .set_parent(root);
        for by in [0.30_f32, 0.53] {
            for bz in [0.24_f32, -0.12] {
                commands
                    .spawn((
                        Mesh3d(kit.cube.clone()),
                        MeshMaterial3d(kit.mech_metal.clone()),
                        Transform::from_xyz(sd * 0.705, by, bz)
                            .with_scale(Vec3::new(0.015, 0.03, 0.03)),
                    ))
                    .set_parent(root);
            }
        }
        for (px, py, sc) in [
            (0.30, 0.15, Vec3::new(0.035, 0.30, 0.035)),
            (0.24, 0.22, Vec3::new(0.055, 0.16, 0.055)),
        ] {
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform {
                        translation: Vec3::new(sd * px, py, -0.08),
                        rotation: Quat::from_rotation_z(sd * -0.45),
                        scale: sc,
                    },
                ))
                .set_parent(root);
        }
    }
    // ---- §owner MECH REFIT: the DENSITY pass ---------------------------
    //
    // The hull reads 15% smaller (see `MECH_HULL_SCALE`), and everything
    // below is what buys that size back as MASS rather than volume. A
    // slab is heavy because it is big; a machine is heavy because every
    // surface on it is doing a job. The brief for this pass was armour
    // layering, pistons, hydraulics, vents, cooling, energy conduits and
    // joints, and each of those is a group here rather than a scatter of
    // greebles - a detail you cannot name is a detail that reads as
    // noise.
    //
    // Everything is torso-local and cosmetic. None of it touches the
    // angle-armour model, the visor weak point, or the plate-detach
    // stages: `mech_red` still appears exactly once on the whole machine,
    // and that is the visor slit.
    {
        // ARMOUR LAYERING - a second skin standing proud of the slab,
        // with a shadow gap behind each panel. Layering is what stops a
        // box being a box: the eye reads the STEP, not the plate.
        for (px, py, pz, sx, sy, sz) in [
            // front glacis, upper and lower, with a rolled lip between
            (0.0_f32, 0.60_f32, 0.556_f32, 0.72_f32, 0.20_f32, 0.030_f32),
            (0.0, 0.40, 0.556, 0.80, 0.16, 0.030),
            // flank panels, inboard of the shoulder housings
            (-0.40, 0.50, 0.28, 0.14, 0.34, 0.34),
            (0.40, 0.50, 0.28, 0.14, 0.34, 0.34),
            // rear deck plate
            (0.0, 0.60, -0.44, 0.62, 0.22, 0.030),
        ] {
            commands
                .spawn((
                    Mesh3d(cube()),
                    MeshMaterial3d(kit.mech_khaki_lt.clone()),
                    Transform::from_xyz(px, py, pz).with_scale(Vec3::new(sx, sy, sz)),
                ))
                .set_parent(root);
            // the shadow gap that makes it read as a separate plate
            commands
                .spawn((
                    Mesh3d(cube()),
                    MeshMaterial3d(kit.mech_shadow.clone()),
                    Transform::from_xyz(px, py, pz - sz.signum() * 0.012)
                        .with_scale(Vec3::new(sx * 1.04, sy * 1.06, sz * 0.5)),
                ))
                .set_parent(root);
        }
        // the ROLLED LIP along the top edge - a chamfer, so the hull has
        // a horizon line instead of a corner
        for (lz, ang) in [(0.50_f32, -0.62_f32), (-0.40, 0.62)] {
            commands
                .spawn((
                    Mesh3d(cube()),
                    MeshMaterial3d(kit.mech_khaki_dk.clone()),
                    Transform {
                        translation: Vec3::new(0.0, 0.715, lz),
                        rotation: Quat::from_rotation_x(ang),
                        scale: Vec3::new(0.96, 0.09, 0.05),
                    },
                ))
                .set_parent(root);
        }

        // ENERGY CONDUITS - glowing lines from the core out to the
        // hardpoints. They are the reason the core reads as a POWER
        // source rather than a light: a reactor with nothing leaving it
        // is a lamp.
        for sd in [-1.0_f32, 1.0] {
            // core -> shoulder, in two dog-legged runs
            for (cx, cy, cz, sx, sy, sz) in [
                (sd * 0.20, 0.545, 0.545, 0.20, 0.016, 0.014),
                (sd * 0.315, 0.50, 0.545, 0.016, 0.11, 0.014),
                (sd * 0.315, 0.445, 0.42, 0.016, 0.014, 0.27),
            ] {
                commands
                    .spawn((
                        Mesh3d(cube()),
                        MeshMaterial3d(kit.med_glow.clone()),
                        Transform::from_xyz(cx, cy, cz)
                            .with_scale(Vec3::new(sx, sy, sz)),
                    ))
                    .set_parent(root);
            }
            // armoured cable trunk running the flank, half-sunk
            commands
                .spawn((
                    Mesh3d(cyl()),
                    MeshMaterial3d(kit.mech_shadow.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.50, 0.33, -0.10),
                        rotation: Quat::from_rotation_z(FRAC_PI_2),
                        scale: Vec3::new(0.045, 0.22, 0.045),
                    },
                ))
                .set_parent(root);
        }

        // COOLING - louvre stacks on both flanks and a radiator block
        // over the reactor. Real vents are a RUN of thin slats with dark
        // behind them; one grille texture would read as a sticker.
        for sd in [-1.0_f32, 1.0] {
            commands
                .spawn((
                    Mesh3d(cube()),
                    MeshMaterial3d(kit.mech_shadow.clone()),
                    Transform::from_xyz(sd * 0.47, 0.60, -0.16)
                        .with_scale(Vec3::new(0.055, 0.20, 0.30)),
                ))
                .set_parent(root);
            for k in 0..5 {
                commands
                    .spawn((
                        Mesh3d(cube()),
                        MeshMaterial3d(kit.mech_metal.clone()),
                        Transform {
                            translation: Vec3::new(
                                sd * 0.485,
                                0.535 + k as f32 * 0.033,
                                -0.16,
                            ),
                            rotation: Quat::from_rotation_x(0.35),
                            scale: Vec3::new(0.040, 0.012, 0.30),
                        },
                    ))
                    .set_parent(root);
            }
        }

        // HYDRAULICS - the waist rams that carry the hull's cantilever,
        // and the shoulder-root actuators. A piston is a POLISHED ROD in
        // a dark cylinder: two parts, or it is just a peg.
        for sd in [-1.0_f32, 1.0] {
            for (bx, by, bz, ang, len) in [
                (0.30_f32, 0.30_f32, -0.30_f32, 0.30_f32, 0.26_f32),
                (0.46, 0.42, 0.30, -0.22, 0.22),
            ] {
                // barrel
                commands
                    .spawn((
                        Mesh3d(cyl()),
                        MeshMaterial3d(kit.mech_khaki_dk.clone()),
                        Transform {
                            translation: Vec3::new(sd * bx, by, bz),
                            rotation: Quat::from_rotation_x(ang),
                            scale: Vec3::new(0.055, len, 0.055),
                        },
                    ))
                    .set_parent(root);
                // the bright rod emerging from it
                commands
                    .spawn((
                        Mesh3d(cyl()),
                        MeshMaterial3d(kit.mech_metal.clone()),
                        Transform {
                            translation: Vec3::new(
                                sd * bx,
                                by + len * 0.52,
                                bz - ang.sin() * len * 0.52,
                            ),
                            rotation: Quat::from_rotation_x(ang),
                            scale: Vec3::new(0.030, len * 0.55, 0.030),
                        },
                    ))
                    .set_parent(root);
            }
        }

        // MECHANICAL JOINTS - collar rings where the arms and hips
        // pivot, so the limbs look SOCKETED rather than glued on.
        for (jx, jy, jz, r) in [
            (-0.60_f32, 0.42_f32, 0.06_f32, 0.145_f32),
            (0.60, 0.42, 0.06, 0.145),
            (0.0, 0.02, 0.02, 0.235),
        ] {
            commands
                .spawn((
                    Mesh3d(cyl()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform {
                        translation: Vec3::new(jx, jy, jz),
                        rotation: if jy > 0.2 {
                            Quat::from_rotation_z(FRAC_PI_2)
                        } else {
                            Quat::IDENTITY
                        },
                        scale: Vec3::new(r, 0.045, r),
                    },
                ))
                .set_parent(root);
            // bolt heads around the collar
            for k in 0..6 {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                let (ox, oy) = (a.cos() * r * 0.72, a.sin() * r * 0.72);
                let pos = if jy > 0.2 {
                    Vec3::new(jx, jy + oy, jz + ox)
                } else {
                    Vec3::new(jx + ox, jy + 0.03, jz + oy)
                };
                commands
                    .spawn((
                        Mesh3d(ball()),
                        MeshMaterial3d(kit.mech_shadow.clone()),
                        Transform::from_translation(pos)
                            .with_scale(Vec3::splat(0.026)),
                    ))
                    .set_parent(root);
            }
        }

        // THE SENSOR HEAD - a combat command unit, not a helmet.
        //
        // It was a khaki box with a red slit. The slit STAYS exactly as
        // it was, because it is the x2 weak point and a gameplay promise;
        // everything here is built around it rather than competing with
        // it. Nothing added is `mech_red`.
        {
            // armour segmentation: a browplate, a jaw, and a split down
            // the crown, so the head reads as assembled from pieces
            for (px, py, pz, sx, sy, sz, mat) in [
                (0.0_f32, 1.045_f32, 0.02_f32, 0.52_f32, 0.035_f32, 0.50_f32, kit.mech_khaki_lt.clone()),
                (0.0, 0.775, 0.06, 0.46, 0.045, 0.42, kit.mech_khaki_dk.clone()),
                (0.0, 0.90, 0.02, 0.045, 0.30, 0.52, kit.mech_khaki_dk.clone()),
            ] {
                commands
                    .spawn((
                        Mesh3d(cube()),
                        MeshMaterial3d(mat),
                        Transform::from_xyz(px, py, pz)
                            .with_scale(Vec3::new(sx, sy, sz)),
                    ))
                    .set_parent(root);
            }
            for sd in [-1.0_f32, 1.0] {
                // main OPTIC pod: housing, barrel, and a lit aperture -
                // three parts, because a camera is a lens in a tube and
                // a single glowing dot is a light
                commands
                    .spawn((
                        Mesh3d(cube()),
                        MeshMaterial3d(kit.mech_khaki_dk.clone()),
                        Transform::from_xyz(sd * 0.235, 0.985, 0.20)
                            .with_scale(Vec3::new(0.11, 0.09, 0.16)),
                    ))
                    .set_parent(root);
                commands
                    .spawn((
                        Mesh3d(cyl()),
                        MeshMaterial3d(kit.mech_metal.clone()),
                        Transform {
                            translation: Vec3::new(sd * 0.235, 0.985, 0.29),
                            rotation: Quat::from_rotation_x(FRAC_PI_2),
                            scale: Vec3::new(0.075, 0.06, 0.075),
                        },
                    ))
                    .set_parent(root);
                commands
                    .spawn((
                        Mesh3d(cyl()),
                        MeshMaterial3d(kit.core_glow.clone()),
                        Transform {
                            translation: Vec3::new(sd * 0.235, 0.985, 0.322),
                            rotation: Quat::from_rotation_x(FRAC_PI_2),
                            scale: Vec3::new(0.045, 0.012, 0.045),
                        },
                    ))
                    .set_parent(root);
                // secondary sensor cluster - three small lenses in a
                // stepped bracket, the "many eyes" read
                for (k, ly) in [(0usize, 0.845_f32), (1, 0.885), (2, 0.925)] {
                    commands
                        .spawn((
                            Mesh3d(ball()),
                            MeshMaterial3d(if k == 1 {
                                kit.core_glow.clone()
                            } else {
                                kit.mech_shadow.clone()
                            }),
                            Transform::from_xyz(sd * 0.30, ly, 0.255)
                                .with_scale(Vec3::splat(0.030)),
                        ))
                        .set_parent(root);
                }
                // cooling louvres in the cheek
                for k in 0..3 {
                    commands
                        .spawn((
                            Mesh3d(cube()),
                            MeshMaterial3d(kit.mech_metal.clone()),
                            Transform {
                                translation: Vec3::new(
                                    sd * 0.315,
                                    0.95 + k as f32 * 0.038,
                                    0.05,
                                ),
                                rotation: Quat::from_rotation_x(0.35),
                                scale: Vec3::new(0.022, 0.010, 0.24),
                            },
                        ))
                        .set_parent(root);
                }
                // COMMS: a swept blade antenna off each crown corner
                commands
                    .spawn((
                        Mesh3d(cube()),
                        MeshMaterial3d(kit.mech_metal.clone()),
                        Transform {
                            translation: Vec3::new(sd * 0.22, 1.135, -0.10),
                            rotation: Quat::from_rotation_z(sd * 0.30),
                            scale: Vec3::new(0.016, 0.20, 0.055),
                        },
                    ))
                    .set_parent(root);
            }
        }

        // THE ARMS - a real shoulder-to-weapon chain.
        //
        // The hull carried its guns on bare hardpoints: a housing, then
        // the weapon, with nothing between them. That is what made the
        // machine read as a turret on legs. Each side gets a socketed
        // pauldron, an upper arm, an elbow block and a forearm cradle, so
        // the weapon is HELD rather than bolted to a wall.
        //
        // Cosmetic and STATIC - these are not animated bones. The mech's
        // mounts fire on their own clocks and the arm exists to make that
        // legible, which is why it is built along the line the weapon
        // already sits on rather than being given joints nothing drives.
        for sd in [-1.0_f32, 1.0] {
            // pauldron: a curved cap over the shoulder collar, with a
            // dark rim so it reads as a separate shell
            commands
                .spawn((
                    Mesh3d(ball()),
                    MeshMaterial3d(kit.mech_khaki.clone()),
                    Transform::from_xyz(sd * 0.635, 0.50, 0.06)
                        .with_scale(Vec3::new(0.28, 0.30, 0.44)),
                ))
                .set_parent(root);
            commands
                .spawn((
                    Mesh3d(cube()),
                    MeshMaterial3d(kit.mech_khaki_dk.clone()),
                    Transform::from_xyz(sd * 0.70, 0.375, 0.06)
                        .with_scale(Vec3::new(0.10, 0.055, 0.42)),
                ))
                .set_parent(root);
            // upper arm - a boxed member angled down and forward
            commands
                .spawn((
                    Mesh3d(cube()),
                    MeshMaterial3d(kit.mech_khaki.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.66, 0.30, 0.10),
                        rotation: Quat::from_rotation_z(sd * 0.10),
                        scale: Vec3::new(0.17, 0.24, 0.20),
                    },
                ))
                .set_parent(root);
            // its actuator, riding the outside of the member
            commands
                .spawn((
                    Mesh3d(cyl()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.755, 0.30, 0.10),
                        rotation: Quat::from_rotation_x(0.18),
                        scale: Vec3::new(0.042, 0.24, 0.042),
                    },
                ))
                .set_parent(root);
            // ELBOW - a metal knuckle with a visible pivot pin, the one
            // place the arm is allowed to look like it bends
            commands
                .spawn((
                    Mesh3d(ball()),
                    MeshMaterial3d(kit.mech_metal.clone()),
                    Transform::from_xyz(sd * 0.655, 0.175, 0.13)
                        .with_scale(Vec3::splat(0.115)),
                ))
                .set_parent(root);
            commands
                .spawn((
                    Mesh3d(cyl()),
                    MeshMaterial3d(kit.mech_shadow.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.655, 0.175, 0.13),
                        rotation: Quat::from_rotation_z(FRAC_PI_2),
                        scale: Vec3::new(0.075, 0.135, 0.075),
                    },
                ))
                .set_parent(root);
            // forearm cradle - the piece the weapon actually sits in
            commands
                .spawn((
                    Mesh3d(cube()),
                    MeshMaterial3d(kit.mech_khaki_dk.clone()),
                    Transform {
                        translation: Vec3::new(sd * 0.645, 0.145, 0.30),
                        rotation: Quat::from_rotation_x(-0.10),
                        scale: Vec3::new(0.155, 0.14, 0.34),
                    },
                ))
                .set_parent(root);
            // and the strap plates that clamp it
            for cz in [0.20_f32, 0.38] {
                commands
                    .spawn((
                        Mesh3d(cube()),
                        MeshMaterial3d(kit.mech_metal.clone()),
                        Transform::from_xyz(sd * 0.645, 0.145, cz)
                            .with_scale(Vec3::new(0.175, 0.16, 0.022)),
                    ))
                    .set_parent(root);
            }
        }

        // THE CORE - a layered reactor, not a light on a wall.
        //
        // Five concentric elements so it reads as DEPTH: a sunk housing,
        // a dark well, a metal iris, the lens itself, and a containment
        // ring standing proud of the glacis. The glow is `core_glow`,
        // the same emissive the antenna tip uses, so the machine has one
        // energy colour and not two.
        {
            let cz = 0.556;
            for (mat, r, z, h) in [
                (kit.mech_khaki_dk.clone(), 0.20_f32, cz - 0.014, 0.030_f32),
                (kit.mech_shadow.clone(), 0.155, cz + 0.002, 0.022),
                (kit.mech_metal.clone(), 0.125, cz + 0.012, 0.018),
                (kit.core_glow.clone(), 0.088, cz + 0.020, 0.016),
            ] {
                commands
                    .spawn((
                        Mesh3d(cyl()),
                        MeshMaterial3d(mat),
                        Transform {
                            translation: Vec3::new(0.0, 0.545, z),
                            rotation: Quat::from_rotation_x(FRAC_PI_2),
                            scale: Vec3::new(r * 2.0, h, r * 2.0),
                        },
                    ))
                    .set_parent(root);
            }
            // containment segments around the lens - eight blocks with
            // gaps, which is what makes it a machine iris and not a dial
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::TAU / 8.0 + 0.39;
                commands
                    .spawn((
                        Mesh3d(cube()),
                        MeshMaterial3d(kit.mech_khaki_lt.clone()),
                        Transform {
                            translation: Vec3::new(
                                a.cos() * 0.165,
                                0.545 + a.sin() * 0.165,
                                cz + 0.014,
                            ),
                            rotation: Quat::from_rotation_z(a),
                            scale: Vec3::new(0.075, 0.036, 0.028),
                        },
                    ))
                    .set_parent(root);
            }
        }
    }

    // gatling dressing: two clamp rings around the barrel cluster and a
    // gold feed-link chute arcing from the ammo box into the housing
    // (gold matches the ammo-pickup vocabulary)
    for (rz, sc) in [
        (0.55, Vec3::new(0.135, 0.025, 0.135)),
        (0.74, Vec3::new(0.125, 0.02, 0.125)),
    ] {
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_khaki_dk.clone()),
                Transform {
                    translation: Vec3::new(0.60, 0.24, rz),
                    rotation: Quat::from_rotation_x(FRAC_PI_2),
                    scale: sc,
                },
            ))
            .set_parent(root);
    }
    for (lx, ly, lz) in [(0.545, 0.155, 0.03), (0.525, 0.205, 0.00), (0.545, 0.255, -0.02)] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.gold.clone()),
                Transform::from_xyz(lx, ly, lz).with_scale(Vec3::new(0.035, 0.035, 0.05)),
            ))
            .set_parent(root);
    }
    // D.6 stage-tagged hull parts - GROUP NODES (unit scale, so children
    // never inherit a squash); `sync_fighters` flips the GROUP's
    // Visibility and the cluster inherits the stage hide.
    // hip skirts: the original plate plus fore/aft lamellae - stage-70
    // then strips the whole hip line, matching the Skirts climb zone
    let skirt = |commands: &mut Commands, sd: f32| -> Entity {
        let g = commands
            .spawn((
                Transform::from_xyz(0.205 * sd, -0.02, 0.02)
                    .with_rotation(Quat::from_rotation_z(-0.12 * sd)),
                Visibility::Inherited,
            ))
            .set_parent(root)
            .id();
        for (tr, rx, sc) in [
            (Vec3::ZERO, 0.0, Vec3::new(0.05, 0.15, 0.22)),
            (Vec3::new(0.0, -0.01, 0.205), -0.10, Vec3::new(0.045, 0.12, 0.13)),
            (Vec3::new(0.0, -0.01, -0.185), 0.10, Vec3::new(0.045, 0.12, 0.13)),
        ] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.mech_khaki_dk.clone()),
                    Transform {
                        translation: tr,
                        rotation: Quat::from_rotation_x(rx),
                        scale: sc,
                    },
                ))
                .set_parent(g);
        }
        g
    };
    let skirt_l = skirt(commands, -1.0);
    let skirt_r = skirt(commands, 1.0);
    // right drum: the drum plus two rib rings, one stage-40 cluster
    let drum_r = {
        let g = commands
            .spawn((
                Transform::from_xyz(0.28, 0.86, -0.38)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                Visibility::Inherited,
            ))
            .set_parent(root)
            .id();
        for (ty, mat, sc) in [
            (0.0, kit.mech_khaki_dk.clone(), Vec3::new(0.22, 0.34, 0.22)),
            (0.08, kit.mech_metal.clone(), Vec3::new(0.235, 0.02, 0.235)),
            (-0.08, kit.mech_metal.clone(), Vec3::new(0.235, 0.02, 0.235)),
        ] {
            commands
                .spawn((
                    Mesh3d(kit.cyl.clone()),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(0.0, ty, 0.0).with_scale(sc),
                ))
                .set_parent(g);
        }
        g
    };
    // antenna: whip + ball tip + a raked second whip - stage-40 sheds a
    // believable comms cluster, matching the Drum zone
    let antenna = {
        let g = commands
            .spawn((Transform::from_xyz(0.42, 0.76, -0.30), Visibility::Inherited))
            .set_parent(root)
            .id();
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform::from_xyz(0.0, 0.36, 0.0)
                    .with_scale(Vec3::new(0.014, 0.72, 0.014)),
            ))
            .set_parent(g);
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform::from_xyz(0.0, 0.735, 0.0).with_scale(Vec3::splat(0.035)),
            ))
            .set_parent(g);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.mech_metal.clone()),
                Transform::from_xyz(-0.05, 0.26, -0.04)
                    .with_rotation(Quat::from_rotation_z(0.18))
                    .with_scale(Vec3::new(0.010, 0.50, 0.010)),
            ))
            .set_parent(g);
        g
    };
    (root, MechHullDetach { skirt_l, skirt_r, drum_r, antenna })
}

/// D.1: reverse-raked leg plating, parented to the REAL leg bones so it
/// animates with the walk. The thigh plate leans back, the shin plate
/// counter-rakes forward - together they fake the reverse-joint zigzag
/// at silhouette range without touching the rig's joints. The knee dome
/// and pistons stay exposed in the gap (Task 5.4: covering the
/// mechanism kills the read). `side` = -1.0 left, +1.0 right.
fn spawn_mech_leg_armor(
    commands: &mut Commands,
    kit: &ModelKit,
    thigh: Entity,
    shin: Entity,
    foot: Entity,
    side: f32,
) -> MechLegArmor {
    let mut root_of = |commands: &mut Commands, bone: Entity| {
        commands
            .spawn((Transform::IDENTITY, Visibility::Hidden))
            .set_parent(bone)
            .id()
    };
    let t_root = root_of(commands, thigh);
    let s_root = root_of(commands, shin);
    let f_root = root_of(commands, foot);
    let part = |commands: &mut Commands,
                parent: Entity,
                mesh: Handle<Mesh>,
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
            .set_parent(parent)
            .id()
    };
    // THIGH: the single largest flat surface, outer face, reverse-raked
    let thigh_plate = part(commands, t_root, kit.cube.clone(), kit.mech_khaki.clone(),
        Vec3::new(side * 0.10, -0.13, 0.03), Quat::from_rotation_x(-0.22), Vec3::new(0.045, 0.34, 0.24));
    part(commands, t_root, kit.cube.clone(), kit.mech_khaki_dk.clone(),
        Vec3::new(side * 0.126, -0.13, 0.03), Quat::from_rotation_x(-0.22), Vec3::new(0.012, 0.30, 0.03));
    part(commands, t_root, kit.cube.clone(), kit.mech_khaki_dk.clone(),
        Vec3::new(0.0, -0.12, 0.095), Quat::from_rotation_x(-0.22), Vec3::new(0.15, 0.20, 0.03));
    // SHIN: exposed knee pistons + counter-raked plate + ankle actuator
    part(commands, s_root, kit.cyl.clone(), kit.mech_metal.clone(),
        Vec3::new(0.045, -0.03, 0.075), Quat::from_rotation_x(0.35), Vec3::new(0.03, 0.16, 0.03));
    part(commands, s_root, kit.cyl.clone(), kit.mech_metal.clone(),
        Vec3::new(-0.045, -0.03, 0.075), Quat::from_rotation_x(0.35), Vec3::new(0.03, 0.16, 0.03));
    let shin_plate = part(commands, s_root, kit.cube.clone(), kit.mech_khaki_dk.clone(),
        Vec3::new(0.0, -0.16, 0.06), Quat::from_rotation_x(0.30), Vec3::new(0.13, 0.20, 0.035));
    // knee hazard strip - within the §4.2 knee-plate allowance
    part(commands, s_root, kit.cube.clone(), kit.mech_hazard.clone(),
        Vec3::new(0.0, -0.075, 0.082), Quat::from_rotation_x(0.30), Vec3::new(0.10, 0.016, 0.012));
    part(commands, s_root, kit.cyl.clone(), kit.mech_metal.clone(),
        Vec3::new(0.0, -0.235, 0.045), Quat::from_rotation_x(0.25), Vec3::new(0.035, 0.10, 0.035));
    // ---- §owner MECH REFIT: the LEG density pass ------------------------
    //
    // Same argument as the hull. The leg was a raked plate a side and two
    // pistons; a walking machine's leg is where all the load goes, and it
    // should be the busiest part of it.
    {
        // THIGH: a second armour layer standing off the main plate, and
        // the hip actuator that drives it
        part(commands, t_root, kit.cube.clone(), kit.mech_khaki_lt.clone(),
            Vec3::new(side * 0.118, -0.06, 0.03), Quat::from_rotation_x(-0.22), Vec3::new(0.020, 0.13, 0.20));
        part(commands, t_root, kit.cube.clone(), kit.mech_shadow.clone(),
            Vec3::new(side * 0.104, -0.06, 0.03), Quat::from_rotation_x(-0.22), Vec3::new(0.010, 0.145, 0.215));
        // hip ram, barrel + rod
        part(commands, t_root, kit.cyl.clone(), kit.mech_khaki_dk.clone(),
            Vec3::new(side * -0.05, -0.10, -0.075), Quat::from_rotation_x(-0.30), Vec3::new(0.048, 0.22, 0.048));
        part(commands, t_root, kit.cyl.clone(), kit.mech_metal.clone(),
            Vec3::new(side * -0.05, -0.20, -0.045), Quat::from_rotation_x(-0.30), Vec3::new(0.026, 0.14, 0.026));
        // inner thigh cabling, half-sunk
        part(commands, t_root, kit.cyl.clone(), kit.mech_shadow.clone(),
            Vec3::new(side * -0.075, -0.14, 0.02), Quat::from_rotation_x(-0.20), Vec3::new(0.030, 0.26, 0.030));

        // KNEE: a real cap over the joint, with a pivot boss either side.
        // The knee is the one silhouette landmark on a leg and it was a
        // hazard stripe on a flat plate.
        part(commands, s_root, kit.ball.clone(), kit.mech_khaki.clone(),
            Vec3::new(0.0, -0.005, 0.075), Quat::IDENTITY, Vec3::new(0.175, 0.16, 0.165));
        part(commands, s_root, kit.cube.clone(), kit.mech_khaki_lt.clone(),
            Vec3::new(0.0, 0.045, 0.10), Quat::from_rotation_x(-0.35), Vec3::new(0.15, 0.055, 0.09));
        for bs in [-1.0_f32, 1.0] {
            part(commands, s_root, kit.cyl.clone(), kit.mech_metal.clone(),
                Vec3::new(bs * 0.088, -0.005, 0.055), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.075, 0.028, 0.075));
        }

        // SHIN: layered armour, a calf mass at the back so the leg is not
        // a slat, and a heat vent stack
        part(commands, s_root, kit.cube.clone(), kit.mech_khaki.clone(),
            Vec3::new(0.0, -0.155, -0.055), Quat::from_rotation_x(-0.12), Vec3::new(0.135, 0.22, 0.09));
        part(commands, s_root, kit.cube.clone(), kit.mech_khaki_lt.clone(),
            Vec3::new(0.0, -0.20, 0.082), Quat::from_rotation_x(0.30), Vec3::new(0.10, 0.10, 0.020));
        for k in 0..3 {
            part(commands, s_root, kit.cube.clone(), kit.mech_metal.clone(),
                Vec3::new(0.0, -0.11 - k as f32 * 0.038, -0.10), Quat::from_rotation_x(0.30), Vec3::new(0.10, 0.010, 0.045));
        }
        // ANKLE: a proper joint ring, not a bare cylinder
        part(commands, s_root, kit.cyl.clone(), kit.mech_shadow.clone(),
            Vec3::new(0.0, -0.275, 0.03), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.10, 0.14, 0.10));

        // STABILISER THRUSTER on the outer calf - the mech does not fly,
        // and this is not a jet: it is the attitude jet a top-heavy
        // walker needs to not fall over when it plants a foot hard. A
        // dark bell with a lit throat, angled down and out.
        part(commands, s_root, kit.cyl.clone(), kit.mech_khaki_dk.clone(),
            Vec3::new(side * 0.105, -0.115, -0.045), Quat::from_rotation_z(side * -0.45), Vec3::new(0.075, 0.11, 0.075));
        part(commands, s_root, kit.cyl.clone(), kit.mech_shadow.clone(),
            Vec3::new(side * 0.128, -0.165, -0.045), Quat::from_rotation_z(side * -0.45), Vec3::new(0.058, 0.03, 0.058));
        part(commands, s_root, kit.cyl.clone(), kit.core_glow.clone(),
            Vec3::new(side * 0.133, -0.178, -0.045), Quat::from_rotation_z(side * -0.45), Vec3::new(0.038, 0.012, 0.038));
    }
    // FOOT: wide pad, rear spur, and the cleat rows. Do NOT smooth them.
    part(commands, f_root, kit.cube.clone(), kit.mech_khaki_dk.clone(),
        Vec3::new(0.0, -0.03, 0.05), Quat::IDENTITY, Vec3::new(0.17, 0.045, 0.30));
    part(commands, f_root, kit.cube.clone(), kit.mech_khaki_dk.clone(),
        Vec3::new(0.0, -0.015, -0.115), Quat::from_rotation_x(0.45), Vec3::new(0.06, 0.035, 0.10));
    let cleat_front = part(commands, f_root, kit.cube.clone(), kit.mech_metal.clone(),
        Vec3::new(0.0, -0.055, 0.14), Quat::IDENTITY, Vec3::new(0.16, 0.018, 0.05));
    part(commands, f_root, kit.cube.clone(), kit.mech_metal.clone(),
        Vec3::new(0.0, -0.055, 0.00), Quat::IDENTITY, Vec3::new(0.16, 0.018, 0.05));
    // toe teeth - NOT part of `cleat_front`: they survive the stage-15
    // shed so the Cleats grip zone still has geometry to grab
    for tx in [-1.0_f32, 1.0] {
        part(commands, f_root, kit.cube.clone(), kit.mech_metal.clone(),
            Vec3::new(tx * 0.055, -0.045, 0.205), Quat::from_rotation_x(0.35), Vec3::new(0.035, 0.025, 0.045));
    }
    MechLegArmor { roots: [t_root, s_root, f_root], thigh_plate, shin_plate, cleat_front }
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
            // the pad floats the hull kit alone - no leg armour on a
            // totem, and its stage parts never hide (ids dropped)
            let (e, _) = spawn_armor_rig(commands, kit);
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

/// §owner: the CLASS SILHOUETTE - the shape that tells you, at the
/// range you decide to shoot from, which of the four you are looking
/// at. Four classes that PLAY differently have to LOOK different, or
/// the system is invisible in the only moment it matters.
///
/// Deliberately shape, not colour: the palette is spoken for. Team side
/// owns body colour (`branding::signal`) and the tunic stripe is the
/// player's own pick - a class hue would fight both.
///
/// Returns the group root, so a caller that needs to SWAP classes (the
/// Forge turntable) can build all four and toggle visibility instead of
/// rebuilding the whole rig on every click.
fn spawn_class_silhouette(
    commands: &mut Commands,
    kit: &ModelKit,
    look: &SoldierLook,
    parent: Entity,
    class: sim::Class,
) -> Entity {
    let group = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .set_parent(parent)
        .id();
    match class {
        // the baseline wears nothing extra, on purpose: it is the shape
        // every other class reads as a deviation FROM
        sim::Class::Line => {}
        sim::Class::Skirmisher => {
            // a short scarf streaming off one shoulder - light, fast
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(look.accent.clone()),
                    Transform::from_xyz(-0.10, 0.60, -0.13)
                        .with_rotation(Quat::from_rotation_x(-0.35))
                        .with_scale(Vec3::new(0.10, 0.26, 0.03)),
                ))
                .set_parent(group);
        }
        sim::Class::Warden => {
            // heavy pauldrons: the widest silhouette on the field
            for sx in [-1.0_f32, 1.0] {
                commands
                    .spawn((
                        Mesh3d(kit.ball.clone()),
                        MeshMaterial3d(look.shell2.clone()),
                        Transform::from_xyz(sx * 0.285, 0.655, 0.0)
                            .with_scale(Vec3::new(0.19, 0.13, 0.24)),
                    ))
                    .set_parent(group);
                commands
                    .spawn((
                        Mesh3d(kit.cube.clone()),
                        MeshMaterial3d(look.joint.clone()),
                        Transform::from_xyz(sx * 0.285, 0.700, 0.0)
                            .with_scale(Vec3::new(0.175, 0.022, 0.225)),
                    ))
                    .set_parent(group);
            }
        }
        sim::Class::Marksman => {
            // a single spotter's mantle over the off shoulder, and a
            // tall aerial - reads as "the one who is watching"
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(look.shell2.clone()),
                    Transform::from_xyz(-0.20, 0.575, 0.0)
                        .with_rotation(Quat::from_rotation_z(0.30))
                        .with_scale(Vec3::new(0.16, 0.20, 0.26)),
                ))
                .set_parent(group);
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.steel.clone()),
                    Transform::from_xyz(-0.16, 0.80, -0.08)
                        .with_rotation(Quat::from_rotation_z(-0.18))
                        .with_scale(Vec3::new(0.012, 0.34, 0.012)),
                ))
                .set_parent(group);
        }
    }
    group
}

/// Handles to one soldier's skeleton - what `FighterRig` wraps for
/// gameplay and the Forge turntable poses statically.
struct SoldierParts {
    root: Entity,
    leg_l: [Entity; 3],
    leg_r: [Entity; 3],
    /// §B.1 #19-20: the forefeet, [left, right].
    toes: [Entity; 2],
    /// §B.2: the twist segment, between the pelvis (root) and the thorax.
    lumbar: Entity,
    /// §B.2: the THORAX. Named `torso` throughout for continuity - it is
    /// the same pivot at the same height that the single trunk segment
    /// used to be, and renaming forty call sites would have obscured what
    /// this change actually did.
    torso: Entity,
    head: Entity,
    /// §B.1 #5-6: the shoulder girdle, [left, right]. The arms hang here.
    clavicles: [Entity; 2],
    arm_l: [Entity; 3],
    arm_r: [Entity; 3],
    weapon_root: Entity,
    weapons: [Entity; N_WEAPONS],
    /// §8.1: where a helmet mounts. Published so a caller that passed
    /// `None` for the helmet can fill it - see `spawn_soldier_body`.
    helmet_socket: Entity,
}

/// The four limb capsules, created once per rig batch.
struct LimbMeshes {
    thigh: Handle<Mesh>,
    shin: Handle<Mesh>,
    upper: Handle<Mesh>,
    fore: Handle<Mesh>,
}

/// One soldier's full material set. `emblem_center` is gold for the
/// player, the dark joint gloss for everyone else.
struct SoldierLook {
    shell: Handle<StandardMaterial>,
    shell2: Handle<StandardMaterial>,
    accent: Handle<StandardMaterial>,
    stripe: Handle<StandardMaterial>,
    hat: Handle<StandardMaterial>,
    joint: Handle<StandardMaterial>,
    knee: Handle<StandardMaterial>,
    eye: Handle<StandardMaterial>,
    emblem_center: Handle<StandardMaterial>,
}

/// Build ONE soldier body - root through the spine-mounted weapon set.
/// Shared by `spawn_fighter_rigs` (which adds FighterVis/FighterRig,
/// shield, arrow, and mech armour on top) and the Forge turntable
/// (which adds NEITHER, so `sync_fighters` and `rebuild_world` can
/// never touch the preview). Extracted rather than copied so the
/// geometry cannot drift from the gap/band constants the rig tests pin
/// (`NECK_*`, `YOKE_HALF_W`, `SHOULDER_X`, `ELBOW_*`, `WRIST_*`).
/// The caller inserts the root transform and any marker components.
fn spawn_soldier_body(
    commands: &mut Commands,
    kit: &ModelKit,
    limbs: &LimbMeshes,
    look: &SoldierLook,
    weapon_detail: bool,
    class: sim::Class,
    // §8.1 which helmet to mount. `Some(i)` indexes `HELMET_CHOICES` and
    // wraps, so a caller can hand it a raw fighter slot. `None` leaves the
    // socket EMPTY for the caller to fill - the Forge turntable needs all
    // five mounted at once, and an unconditional one here would leave it
    // wearing two.
    helmet: Option<usize>,
) -> SoldierParts {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // LEGS fill the 0.00–0.63 budget: thigh 0.29, shin 0.28, foot 0.06.
    // Every joint is a visible dark gap; the KNEE is the signature - a
    // glossy dark dome sitting proud of the shin.
    let mut legs = [[Entity::PLACEHOLDER; 3]; 2];
    let mut toes = [Entity::PLACEHOLDER; 2];
    for (li, lx) in [(-0.11_f32), 0.11].into_iter().enumerate() {
        let thigh = commands
            .spawn((Transform::from_xyz(lx, 0.63, 0.0), Visibility::default()))
            .set_parent(root)
            .id();
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(look.joint.clone()),
                Transform::from_scale(Vec3::new(0.13, 0.11, 0.13)),
            ))
            .set_parent(thigh);
        commands
            .spawn((
                Mesh3d(limbs.thigh.clone()),
                MeshMaterial3d(look.shell.clone()),
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
                MeshMaterial3d(look.knee.clone()),
                Transform::from_xyz(0.0, -0.005, 0.045)
                    .with_scale(Vec3::new(0.13, 0.13, 0.13)),
            ))
            .set_parent(shin);
        commands
            .spawn((
                Mesh3d(limbs.shin.clone()),
                MeshMaterial3d(look.shell2.clone()),
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
                MeshMaterial3d(look.joint.clone()),
                Transform::from_scale(Vec3::new(0.09, 0.07, 0.09)),
            ))
            .set_parent(foot);
        // the HINDFOOT shell - shortened at the front to make room for a
        // forefoot that can now hinge away from it
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(look.shell.clone()),
                Transform::from_xyz(0.0, -0.025, 0.025)
                    .with_scale(Vec3::new(0.14, 0.09, 0.17)),
            ))
            .set_parent(foot);
        // §B.1 #19-20 THE TOE / FOREFOOT. "The toe-off snap the sprint
        // spec requires" - and until it existed the sprint could not have
        // one, because there was nothing forward of the ankle to push
        // off. §B.6's toe-off test is the proof it landed.
        //
        // Hinged at the ball of the foot, so plantar flexion rotates the
        // forefoot about the same line a real one does.
        let toe = commands
            .spawn((Transform::from_xyz(0.0, -0.03, 0.105), Visibility::default()))
            .set_parent(foot)
            .id();
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(look.shell2.clone()),
                Transform::from_xyz(0.0, 0.005, 0.035)
                    .with_scale(Vec3::new(0.13, 0.07, 0.10)),
            ))
            .set_parent(toe);
        legs[li] = [thigh, shin, foot];
        toes[li] = toe;
    }
    let [leg_l, leg_r] = legs;
    // §B.2 THE THREE-PART TRUNK - "the critical fix".
    //
    //   root/PELVIS  -> LUMBAR -> THORAX
    //
    // The rig had two effective trunk segments: `root` carries the leg
    // yaw and `torso` carried an additive yaw on top, so hip-shoulder
    // separation was already non-zero. What it did NOT have was anything
    // BETWEEN them, so the entire twist happened at one joint - a spine
    // that behaved like a hinge. §B.2 calls the lumbar "the twist
    // segment: this is what makes hip-shoulder separation possible", and
    // possible is not the same as distributed.
    //
    // The lumbar sits at IDENTITY and takes a share of the separation in
    // `sync_fighters`, so the coil is spread over two joints. The thorax
    // keeps the pivot the old torso had, at exactly its old height, which
    // is what makes this a pure insertion: every shell, socket, arm and
    // head below hangs off the thorax at the local offsets it always had,
    // so an unposed rig is untouched geometry.
    // The LUMBAR carries the waist height and the THORAX sits at identity
    // on top of it - so both twist about the waist, exactly where the
    // single trunk segment used to.
    //
    // Note which one holds `WAIST_Y`. `sync_fighters` writes the thorax's
    // translation every frame (hip bob, sway, breathing), and that write
    // is expressed in ROOT space because it always was. Leaving the 0.63
    // on the thorax as well would add the waist height twice and lift the
    // whole upper body a full waist off the legs - which is exactly what
    // the first version of this did, and it is invisible to every test in
    // this file because they all measure the head band and the separation
    // ANGLE, neither of which cares where the torso is.
    let lumbar = commands
        .spawn((Transform::from_xyz(0.0, WAIST_Y, 0.0), Visibility::default()))
        .set_parent(root)
        .id();
    let torso = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .set_parent(lumbar)
        .id();
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(look.shell.clone()),
            Transform::from_xyz(0.0, 0.09, 0.0).with_scale(Vec3::new(0.34, 0.16, 0.26)),
        ))
        .set_parent(torso);
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(look.shell2.clone()),
            Transform::from_xyz(0.0, 0.235, 0.0).with_scale(Vec3::new(0.30, 0.24, 0.24)),
        ))
        .set_parent(torso);
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(look.shell.clone()),
            Transform::from_xyz(0.0, 0.455, 0.0).with_scale(Vec3::new(0.40, 0.30, 0.30)),
        ))
        .set_parent(torso);
    // §1.4 accent 1/3: the thin waist stripe (player: tunic pick)
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(look.stripe.clone()),
            Transform::from_xyz(0.0, 0.155, 0.0).with_scale(Vec3::new(0.345, 0.03, 0.27)),
        ))
        .set_parent(torso);
    // §1.4 accent 2/3: the chest emblem - a small ring inset on the
    // upper-left chest, dark center (the player's is gold)
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(look.accent.clone()),
            Transform::from_xyz(-0.09, 0.52, 0.145)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                .with_scale(Vec3::new(0.075, 0.012, 0.075)),
        ))
        .set_parent(torso);
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(look.emblem_center.clone()),
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
            MeshMaterial3d(look.shell.clone()),
            Transform::from_xyz(0.0, 0.625, 0.0)
                .with_scale(Vec3::new(YOKE_HALF_W * 2.0, 0.14, 0.24)),
        ))
        .set_parent(torso);
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(look.accent.clone()),
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
            MeshMaterial3d(look.joint.clone()),
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
            MeshMaterial3d(look.shell.clone()),
            Transform::from_xyz(0.0, 0.162, 0.01).with_scale(Vec3::new(0.38, 0.324, 0.35)),
        ))
        .set_parent(head);
    for ex in [-0.075_f32, 0.075] {
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(look.eye.clone()),
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
    // §8.1: the helmet is a library pick now, not five hand-written
    // spawns. Index 0 reproduces those five exactly.
    if let Some(h) = helmet {
        spawn_helmet(commands, kit, look, hat_socket, h);
    }
    // ARMS: shoulder → elbow → wrist off the yoke, every joint a dark
    // ball in a visible gap, white shell segments, mitten hands
    let mut arms = [[Entity::PLACEHOLDER; 3]; 2];
    let mut clavs = [Entity::PLACEHOLDER; 2];
    for (ai, ax) in [(-SHOULDER_X), SHOULDER_X].into_iter().enumerate() {
        // §B.1 #5-6 THE CLAVICLE. "The shoulder must TRAVEL, not just
        // rotate - throwing and recoil both need it."
        //
        // The travel already existed as a spring (`FighterRig::clav`),
        // but it was applied to the IK TARGET and nowhere else, so the
        // solver aimed at a moving shoulder while the arm still hung off
        // a fixed one. Making it a real bone is what closes that gap: the
        // arm is parented HERE, so when the girdle moves, everything
        // downstream of it moves too - which is what a shoulder girdle
        // is for.
        //
        // Its rest transform carries the offset the arm used to carry, so
        // an unanimated rig is in exactly the pose it was before.
        let clav = commands
            .spawn((Transform::from_xyz(ax, 0.62, 0.02), Visibility::default()))
            .set_parent(torso)
            .id();
        clavs[ai] = clav;
        let upper = commands
            .spawn((Transform::IDENTITY, Visibility::default()))
            .set_parent(clav)
            .id();
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(look.joint.clone()),
                Transform::from_scale(Vec3::new(0.11, 0.10, 0.11)),
            ))
            .set_parent(upper);
        commands
            .spawn((
                Mesh3d(limbs.upper.clone()),
                MeshMaterial3d(look.shell.clone()),
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
                MeshMaterial3d(look.joint.clone()),
                Transform::from_scale(Vec3::splat(ELBOW_R * 2.0)),
            ))
            .set_parent(fore);
        commands
            .spawn((
                Mesh3d(limbs.fore.clone()),
                MeshMaterial3d(look.shell2.clone()),
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
                MeshMaterial3d(look.joint.clone()),
                Transform::from_scale(Vec3::splat(WRIST_R * 2.0)),
            ))
            .set_parent(hand);
        commands
            .spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(look.shell.clone()),
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
        let model = spawn_weapon_model(commands, kit, wk, weapon_detail, false);
        commands
            .entity(model)
            .insert((Transform::IDENTITY, Visibility::Hidden))
            .set_parent(weapon_root);
        weapons[wi] = model;
    }
    spawn_class_silhouette(commands, kit, look, torso, class);
    SoldierParts {
        root,
        leg_l,
        leg_r,
        toes,
        lumbar,
        torso,
        head,
        clavicles: clavs,
        arm_l,
        arm_r,
        weapon_root,
        weapons,
        helmet_socket: hat_socket,
    }
}

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
    let limbs = LimbMeshes {
        thigh: mesh_thigh,
        shin: mesh_shin,
        upper: mesh_upper,
        fore: mesh_fore,
    };
    // §1.4 shared shell/joint materials - created ONCE per rebuild, cloned
    // per fighter only for the accent slot.
    //
    // Team identity is carried by the WHOLE BODY, not just a trim band:
    // allies are bright cold steel, enemies dark oxide. A band-only tell
    // vanishes the moment a fighter is edge-on or backlit, which is
    // exactly when you most need to know whether to shoot.
    // The second shell tone is the first one stepped down, so the two
    // never drift apart when a side's colour is retuned.
    let shade = |c: Color, k: f32| {
        let s = c.to_srgba();
        Color::srgb(s.red * k, s.green * k, s.blue * k)
    };
    let shell_ally = materials.add(metal(branding::signal::ALLY, 0.0, 0.42));
    let shell2_ally = materials.add(metal(shade(branding::signal::ALLY, 0.92), 0.0, 0.45));
    let shell_enemy = materials.add(metal(branding::signal::ENEMY, 0.0, 0.42));
    let shell2_enemy = materials.add(metal(shade(branding::signal::ENEMY, 0.82), 0.0, 0.45));
    let joint = materials.add(metal(Color::srgb_u8(0x17, 0x19, 0x1D), 0.85, 0.22));
    let knee = materials.add(metal(Color::srgb_u8(0x0E, 0x10, 0x13), 0.20, 0.08));
    let eye_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x0A, 0x0B, 0x0D),
        perceptual_roughness: 0.15,
        // a faint cool glow so the eyes still read at range
        emissive: LinearRgba::new(0.016, 0.021, 0.028, 1.0),
        ..default()
    });
    // The enemy accent glows harder than the ally one. Gold on a bright
    // shell already separates itself by hue; orange on a dark shell has
    // to do the work with light, and it is the tell that survives a
    // muzzle flash washing the scene out.
    let accent_of = |side: branding::signal::Side| {
        let (r, g, b) = side.accent_rgb();
        let e = match side {
            branding::signal::Side::Ally => 0.40,
            branding::signal::Side::Enemy => 0.85,
        };
        StandardMaterial {
            base_color: side.accent(),
            perceptual_roughness: 0.35,
            emissive: LinearRgba::new(r * e, g * e, b * e, 1.0),
            ..default()
        }
    };
    let accent_ally = materials.add(accent_of(branding::signal::Side::Ally));
    let accent_enemy = materials.add(accent_of(branding::signal::Side::Enemy));
    // Whose side the viewer is on. Read once, not assumed to be Blue -
    // the player takes whichever slot the sim gave them.
    let p_team = sim.fighters[sim.player].team;

    for (i, f) in sim.fighters.iter().enumerate() {
        let is_player = i == sim.player;
        let slot = i % 5;
        let side = branding::signal::side_of(f.team, p_team);
        let ally = side == branding::signal::Side::Ally;
        // Shadow the shared handles with this fighter's side, so every
        // `shell`/`shell2` use below stays a one-word reference.
        let shell = if ally { shell_ally.clone() } else { shell_enemy.clone() };
        let shell2 = if ally { shell2_ally.clone() } else { shell2_enemy.clone() };
        let accent = if ally { accent_ally.clone() } else { accent_enemy.clone() };
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
        let look = SoldierLook {
            shell: shell.clone(),
            shell2: shell2.clone(),
            accent: accent.clone(),
            stripe,
            hat,
            joint: joint.clone(),
            knee: knee.clone(),
            eye: eye_mat.clone(),
            emblem_center: if is_player {
                kit.gold.clone()
            } else {
                joint.clone()
            },
        };
        // §8.1: the player wears their Forge pick; each bot wears a helmet
        // derived from its slot, so a firefight has five silhouettes in it
        // rather than one repeated. Derived from the INDEX, never from rng
        // - a random helmet would differ between a live match and its
        // replay, and this whole build is a bit-identical-replay build.
        let helmet = if is_player {
            sel.helmet
        } else {
            slot % HELMET_CHOICES.len()
        };
        let parts =
            spawn_soldier_body(commands, kit, &limbs, &look, is_player, f.class, Some(helmet));
        commands.entity(parts.root).insert((
            Transform::from_xyz(f.pos[0], f.pos[1], f.pos[2]),
            FighterVis { index: i },
        ));
        let SoldierParts {
            root,
            leg_l,
            leg_r,
            toes,
            lumbar,
            torso,
            head,
            clavicles,
            arm_l,
            arm_r,
            weapon_root,
            weapons,
            // §8.1: a live fighter's helmet is mounted and never touched
            // again - only the Forge turntable needs the socket back.
            helmet_socket: _,
        } = parts;
        // the always-carried shield, on the left forearm
        let shield = spawn_shield_model(commands, kit, false);
        commands
            .entity(shield)
            .insert((
                Transform::from_xyz(0.0, -0.12, 0.09)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                Visibility::Hidden,
            ))
            .set_parent(arm_l[1]);
        // (The nocked arrow used to be spawned here, on the BOW hand, as a
        // featureless box at a fixed offset. It now lives inside the bow
        // model itself - see `spawn_weapon_model` - so it rides the draw
        // and the viewmodel gets one too.)
        // the mech hull kit + leg armour (Brief VIII-B D.1-D.6)
        let (armor_rig, hull_det) = spawn_armor_rig(commands, kit);
        // §owner MECH BARRIER: on the LEFT forearm cradle, which is the
        // arm that is not holding the gatling - a shield on the gun arm
        // would be a shield you cannot use while shooting.
        let (barrier_root, barrier) = spawn_mech_barrier(commands, kit);
        commands
            .entity(barrier_root)
            .insert(Transform {
                translation: Vec3::new(-0.645, 0.145, 0.30) * MECH_HULL_SCALE,
                rotation: Quat::from_rotation_x(-0.10),
                scale: Vec3::splat(MECH_HULL_SCALE),
            })
            .set_parent(armor_rig);
        let la_l = spawn_mech_leg_armor(commands, kit, leg_l[0], leg_l[1], leg_l[2], -1.0);
        let la_r = spawn_mech_leg_armor(commands, kit, leg_r[0], leg_r[1], leg_r[2], 1.0);
        commands
            .entity(armor_rig)
            .insert((
                Transform::from_scale(Vec3::splat(MECH_HULL_SCALE)),
                Visibility::Hidden,
            ))
            .set_parent(torso);
        commands.entity(root).insert(barrier);
        commands.entity(root).insert(FighterRig {
            phase: 0.0,
            prev_speed: 0.0,
            accel_lean: 0.0,
            sprint_t: 0.0,
            carry_t: 0.0,
            prev_yaw_vis: f.yaw,
            wr_lag_yaw: 0.0,
            hand_r: Vec3::NAN,
            hand_r_v: Vec3::ZERO,
            hand_l: Vec3::NAN,
            hand_l_v: Vec3::ZERO,
            pole_r_s: Vec3::NAN,
            pole_r_v: Vec3::ZERO,
            pole_l_s: Vec3::NAN,
            pole_l_v: Vec3::ZERO,
            clav: Vec3::NAN,
            clav_v: Vec3::ZERO,
            wr_lag_v: 0.0,
            leg_l,
            leg_r,
            torso,
            neck: head,
            arm_l,
            arm_r,
            toes,
            lumbar,
            clavicles,
            weapon_root,
            weapons,
            shield,
            armor_rig,
            mech_leg_armor: [la_l.roots, la_r.roots],
            mech_detach_70: [hull_det.skirt_l, hull_det.skirt_r, la_l.thigh_plate],
            mech_detach_40: [la_l.shin_plate, hull_det.drum_r, hull_det.antenna],
            mech_detach_15: [la_l.cleat_front],
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
    mut images: ResMut<Assets<Image>>,
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
        nade_ready: false,
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
    // §owner TEXTURE PIPELINE: generated once at startup, then shared.
    // Built BEFORE the model kit so the mech's own materials can mount
    // the metal grain, and inserted as a resource so `rebuild_world` -
    // which runs on every deploy and rematch - always finds it present.
    let tex_kit = build_texture_kit(&mut images);
    // §owner: the same `metal`/`flat` helpers, with a surface mounted.
    // Written as helpers rather than expanding every material inline so
    // the kit stays a readable table of colours instead of forty lines
    // of struct literal.
    let tex_metal = |c: Color, m: f32, r: f32, uv: f32| StandardMaterial {
        base_color: c,
        base_color_texture: Some(tex_kit.metal.clone()),
        normal_map_texture: Some(tex_kit.metal_n.clone()),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(uv)),
        metallic: m,
        perceptual_roughness: r,
        ..default()
    };
    let tex_wood = |c: Color, uv: Vec2| StandardMaterial {
        base_color: c,
        base_color_texture: Some(tex_kit.wood.clone()),
        normal_map_texture: Some(tex_kit.wood_n.clone()),
        uv_transform: bevy::math::Affine2::from_scale(uv),
        perceptual_roughness: 0.85,
        ..default()
    };
    // §owner: TANGENTS. A normal map without them is silently ignored -
    // Bevy needs a tangent basis to take a texel from tangent space into
    // world space, and the primitive builders do not generate one. Doing
    // it once here covers every model in the game, since all of them are
    // built from these three shared meshes.
    let with_tangents = |mut m: Mesh| -> Mesh {
        // failure is cosmetic (that mesh just keeps flat normals), so it
        // warns rather than panicking the whole game at startup
        if let Err(e) = m.generate_tangents() {
            warn!("tangent generation failed, normal maps disabled for a mesh: {e}");
        }
        m
    };
    let kit = ModelKit {
        cube: meshes.add(with_tangents(Cuboid::new(1.0, 1.0, 1.0).into())),
        cyl: meshes.add(with_tangents(Cylinder::new(0.5, 1.0).into())),
        ball: meshes.add(with_tangents(Sphere::new(0.5).mesh().uv(24, 12))),
        grey_light: materials.add(tex_metal(Color::srgb_u8(0xC8, 0xC9, 0xCB), 0.05, 0.60, 3.0)),
        grey_mid: materials.add(tex_metal(Color::srgb_u8(0x8A, 0x8C, 0x8F), 0.05, 0.60, 3.0)),
        grey_dark: materials.add(tex_metal(Color::srgb_u8(0x3A, 0x3C, 0x40), 0.05, 0.60, 3.0)),
        grey_black: materials.add(tex_metal(Color::srgb_u8(0x1E, 0x20, 0x24), 0.05, 0.60, 3.0)),
        gunmetal: materials.add(tex_metal(Color::srgb(0.16, 0.17, 0.19), 0.8, 0.45, 3.0)),
        steel: materials.add(tex_metal(Color::srgb(0.62, 0.64, 0.68), 0.95, 0.30, 3.0)),
        wood: materials.add(tex_wood(Color::srgb(0.42, 0.28, 0.15), Vec2::new(1.0, 2.5))),
        string: materials.add(metal(Color::srgb(0.85, 0.82, 0.70), 0.0, 0.9)),
        lens: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 1.0),
            emissive: LinearRgba::new(0.4, 1.6, 2.4, 1.0),
            unlit: true,
            ..default()
        }),
        olive: materials.add(tex_metal(Color::srgb(0.32, 0.35, 0.22), 0.2, 0.8, 2.0)),
        gold: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.80, 0.30),
            metallic: 1.0,
            perceptual_roughness: 0.25,
            emissive: LinearRgba::new(0.6, 0.45, 0.1, 1.0),
            ..default()
        }),
        white: materials.add(tex_metal(Color::srgb(0.92, 0.92, 0.90), 0.1, 0.6, 2.0)),
        med_glow: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.95, 0.4),
            emissive: LinearRgba::new(0.2, 1.8, 0.4, 1.0),
            unlit: true,
            ..default()
        }),
        armor_dark: materials.add(tex_metal(Color::srgb(0.14, 0.15, 0.18), 0.9, 0.35, 2.4)),
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
        // §owner: the chassis carries the brushed grain. Its plates
        // are the largest single-colour surfaces on any character, so
        // they are where flat shading showed worst.
        mech_khaki: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x8A, 0x87, 0x70),
            base_color_texture: Some(tex_kit.metal.clone()),
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(2.2)),
            metallic: 0.05,
            perceptual_roughness: 0.72,
            ..default()
        }),
        mech_khaki_dk: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x5F, 0x5E, 0x52),
            base_color_texture: Some(tex_kit.metal.clone()),
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(2.2)),
            metallic: 0.05,
            perceptual_roughness: 0.75,
            ..default()
        }),
        mech_khaki_lt: materials.add(tex_metal(Color::srgb_u8(0x9A, 0x93, 0x84), 0.05, 0.65, 2.2)),
        mech_shadow: materials.add(flat(0x33352F)),
        mech_metal: materials.add(tex_metal(Color::srgb_u8(0x2B, 0x2C, 0x2B), 0.15, 0.45, 2.6)),
        // §4.2: hazard chevrons - shoulder-pod cover and knee plates
        // ONLY (≤10% of surface; an accent, not a paint job)
        mech_hazard: materials.add(flat(0xD9A916)),
        mech_red: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0xC2, 0x3B, 0x2E),
            emissive: LinearRgba::new(1.6, 0.25, 0.12, 1.0),
            unlit: true,
            ..default()
        }),
        // FP-only translucent shield set. Metallic stays LOW: metallic
        // 1.0 + Blend reads as near-invisible smoked glass. Alpha 0.28
        // per slat because the three angled slats overlap and
        // double-blend at their edges (~0.3 net).
        // §owner MECH BARRIER: the energy field itself.
        //
        // Two materials because the brief asks for two contradictory
        // things at once - "transparent from the pilot's perspective for
        // visibility" and "visible to enemies through glowing energy
        // edges". A single translucent sheet cannot be both; a very faint
        // FILL plus a bright EDGE can. The fill is what you see through,
        // the edges are what they see.
        //
        // Unlit on the fill so it does not pick up the map's lighting and
        // go opaque in shadow - a barrier that dims when you back into
        // cover is a barrier the pilot cannot trust.
        barrier_fill: materials.add(StandardMaterial {
            base_color: Color::srgba(0.30, 0.72, 0.95, 0.085),
            emissive: LinearRgba::new(0.06, 0.20, 0.30, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        barrier_edge: materials.add(StandardMaterial {
            base_color: Color::srgba(0.55, 0.90, 1.0, 0.60),
            emissive: LinearRgba::new(1.2, 3.4, 4.6, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        vm_shield_dark: materials.add(StandardMaterial {
            base_color: Color::srgba(0.14, 0.15, 0.18, 0.28),
            metallic: 0.1,
            perceptual_roughness: 0.35,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        vm_shield_steel: materials.add(StandardMaterial {
            base_color: Color::srgba(0.62, 0.64, 0.68, 0.35),
            metallic: 0.2,
            perceptual_roughness: 0.30,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        vm_shield_gold: materials.add(StandardMaterial {
            base_color: Color::srgba(0.95, 0.80, 0.30, 0.45),
            metallic: 0.3,
            perceptual_roughness: 0.25,
            emissive: LinearRgba::new(0.18, 0.14, 0.03, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        // decorative parchment (branding::palette::PARCHMENT_DIM), never
        // the ally signal white
        mech_stencil: materials.add(metal(Color::srgb(0.68, 0.63, 0.55), 0.0, 0.8)),
        // unlit: an illuminated reticle does not take scene lighting, so
        // it stays legible against a dark wall and a bright skyline alike
        optic_red: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.14, 0.10),
            emissive: LinearRgba::new(6.0, 0.35, 0.20, 1.0),
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
    commands.insert_resource(tex_kit.clone());
    commands.insert_resource(kit.clone());

    // ---- §7.2 the Forge turntable stage --------------------------------
    // A mannequin on a slowly turning pedestal, lit and filmed by its own
    // camera into a texture. The soldier page frames that texture, so the
    // player SEES the kit they are assembling - the piece that turns the
    // Forge from a save file into an editor.
    {
        use bevy::render::camera::RenderTarget;
        use bevy::render::render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        };
        let size = Extent3d {
            width: FORGE_PREVIEW_PX,
            height: FORGE_PREVIEW_PX,
            depth_or_array_layers: 1,
        };
        let mut target = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("forge_turntable"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        target.resize(size);
        let image = images.add(target);

        // the stage's own camera - renders BEFORE the main pass
        commands.spawn((
            Camera3d::default(),
            Camera {
                target: RenderTarget::Image(image.clone()),
                order: -2,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.07, 0.055, 0.04)),
                ..default()
            },
            Transform::from_translation(FORGE_STAGE_POS + Vec3::new(1.7, 1.6, 2.4))
                .looking_at(FORGE_STAGE_POS + Vec3::new(0.0, 0.98, 0.0), Vec3::Y),
            RenderLayers::layer(FORGE_PREVIEW_LAYER),
        ));
        commands.spawn((
            PointLight {
                intensity: 300_000.0,
                range: 12.0,
                ..default()
            },
            Transform::from_translation(FORGE_STAGE_POS + Vec3::new(1.6, 2.6, 1.4)),
            RenderLayers::layer(FORGE_PREVIEW_LAYER),
        ));

        // unique cosmetic materials the sync system recolours in place -
        // SAME surface params as the gameplay rig, so the card matches
        // the field (the old mannequin used flatter ones)
        let hat_mat = materials.add(metal(Color::srgb(0.92, 0.90, 0.85), 0.05, 0.85));
        let tunic_mat = {
            let (_, (r, g, b)) = TUNIC_CHOICES[0];
            materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
                perceptual_roughness: 0.35,
                emissive: LinearRgba::new(r * 0.4, g * 0.4, b * 0.4, 1.0),
                ..default()
            })
        };
        let bronze_ped = materials.add(metal(Color::srgb(0.55, 0.42, 0.22), 0.3, 0.5));

        // the STAND - everything on it rotates together
        let stand = commands
            .spawn((
                Transform::from_translation(FORGE_STAGE_POS),
                Visibility::default(),
            ))
            .id();
        // pedestal
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(bronze_ped),
                Transform::from_xyz(0.0, 0.03, 0.0)
                    .with_scale(Vec3::new(0.95, 0.06, 0.95)),
            ))
            .set_parent(stand);
        // THE SOLDIER - the REAL gameplay rig (ally colours, player
        // detail), not a mannequin: the card shows the character the
        // player actually fields, gun in hand. It carries NEITHER
        // FighterVis NOR FighterRig, so `sync_fighters` never animates
        // it and `rebuild_world` never despawns it.
        let shade = |c: Color, k: f32| {
            let sc = c.to_srgba();
            Color::srgb(sc.red * k, sc.green * k, sc.blue * k)
        };
        let limbs = LimbMeshes {
            thigh: meshes.add(Capsule3d::new(0.072, 0.15)),
            shin: meshes.add(Capsule3d::new(0.060, 0.15)),
            upper: meshes.add(Capsule3d::new(0.055, 0.14)),
            fore: meshes.add(Capsule3d::new(0.048, 0.12)),
        };
        let joint = materials.add(metal(Color::srgb_u8(0x17, 0x19, 0x1D), 0.85, 0.22));
        let look = SoldierLook {
            shell: materials.add(metal(branding::signal::ALLY, 0.0, 0.42)),
            shell2: materials.add(metal(shade(branding::signal::ALLY, 0.92), 0.0, 0.45)),
            accent: {
                let (r, g, b) = branding::signal::Side::Ally.accent_rgb();
                materials.add(StandardMaterial {
                    base_color: branding::signal::Side::Ally.accent(),
                    perceptual_roughness: 0.35,
                    emissive: LinearRgba::new(r * 0.40, g * 0.40, b * 0.40, 1.0),
                    ..default()
                })
            },
            stripe: tunic_mat.clone(),
            hat: hat_mat.clone(),
            joint: joint.clone(),
            knee: materials.add(metal(Color::srgb_u8(0x0E, 0x10, 0x13), 0.20, 0.08)),
            eye: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x0A, 0x0B, 0x0D),
                perceptual_roughness: 0.15,
                emissive: LinearRgba::new(0.016, 0.021, 0.028, 1.0),
                ..default()
            }),
            emblem_center: kit.gold.clone(),
        };
        // Class::Line adds nothing, so the body is built bare and each
        // class's shape is hung beside it; `forge_preview_sync` shows
        // the picked one. Rebuilding the rig per click would fight the
        // layer-tagging latch (`tag_forge_preview_layer` stamps once).
        // §8.1: `None` - the socket is filled by the rack below, not by
        // one helmet, so the mannequin can change heads without a rebuild.
        let parts =
            spawn_soldier_body(&mut commands, &kit, &limbs, &look, true, sim::Class::Line, None);
        let helmets: [Entity; N_HELMETS] = std::array::from_fn(|i| {
            spawn_helmet(&mut commands, &kit, &look, parts.helmet_socket, i)
        });
        let mut class_rigs = [Entity::PLACEHOLDER; 4];
        for (i, c) in sim::Class::ALL.into_iter().enumerate() {
            let g = spawn_class_silhouette(&mut commands, &kit, &look, parts.torso, c);
            commands.entity(g).insert(Visibility::Hidden);
            class_rigs[i] = g;
        }
        commands
            .entity(parts.root)
            .insert(Transform::from_xyz(0.0, 0.06, 0.0)) // feet on the pedestal
            .set_parent(stand);
        commands.insert_resource(ForgePreview {
            image,
            stand,
            weapons: parts.weapons,
            hat_mat,
            tunic_mat,
            weapon_root: parts.weapon_root,
            arm_l: parts.arm_l,
            arm_r: parts.arm_r,
            class_rigs,
            helmets,
        });
    }

    // ---- shot / impact FX pools ----------------------------------------
    commands.insert_resource(FxAssets {
        tracer_mesh: meshes.add(Cuboid::new(0.02, 0.02, 1.0)),
        // Ally fire: pale gold, cool-hot core. Bright but not alarming.
        tracer_ally: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.95, 0.80),
            emissive: LinearRgba::new(3.4, 2.9, 1.6, 1.0),
            unlit: true,
            ..default()
        }),
        // Enemy fire: hot orange-red, pushed harder than the ally streak
        // so incoming rounds are the loudest thing on screen.
        tracer_enemy: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.55, 0.35),
            emissive: LinearRgba::new(5.0, 1.7, 0.5, 1.0),
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
        HudRoot,
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
        HudRoot,
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
        ))
        .id();

    // ---- §owner: the UI CAMERA, and why the HUD needs its own ----------
    //
    // The HUD used to render through MainCam (order 0). The viewmodel
    // camera is order 1 with no clear, so it composites AFTER - and
    // therefore OVER - everything MainCam drew, the HUD included. On foot
    // that went unnoticed because a rifle sits low and right of the ammo
    // block. In a mech it is unmissable: the mount is a much bigger body
    // and it ate the first characters of the ammo readout, which is how
    // "TURRET 300 / HEAT 0%" reached the screen as "TURRET 300 / EAT 0%".
    //
    // Same family as the gun-over-the-pause-menu bug, and the visibility
    // gate that fixed THAT one cannot help here: the HUD is supposed to
    // be on screen at the same time as the weapon. The only real answer
    // is ordering - the interface draws last, after every 3D pass, so
    // nothing in the world can ever composite on top of it.
    commands.spawn((
        Camera2d,
        Camera {
            // above MainCam (0) and the viewmodel (1)
            order: 2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        UiCam,
    ));

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
        let (tr, extra_rx) = vm_carry(wk);
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
    // the raised shield fills the view when it's up - the TRANSLUCENT
    // copy: it guards without blinding
    let vm_shield = spawn_shield_model(&mut commands, &kit, true);
    commands
        .entity(vm_shield)
        .insert((
            Transform::from_xyz(-0.10, -0.14, -0.55).with_scale(Vec3::splat(1.1)),
            Visibility::Hidden,
        ))
        .set_parent(vm_root);
    // §C.7: the two hull-mount viewmodels. MUST spawn here in setup -
    // `tag_viewmodel_layer` latches after its first sweep and would
    // never stamp a late spawn onto the vm camera's layer.
    let mech_turret = spawn_mech_turret_vm(&mut commands, &kit);
    commands
        .entity(mech_turret)
        .insert((
            // A hull mount is a MUCH chunkier body than a rifle, so it
            // cannot reuse the rifle carry distance: at the old
            // (0.16, -0.15, -0.34) x0.9 the housing's near face sat
            // 0.13 m from a 68-degree lens and ate 80% of the screen.
            // Pushed back and shrunk until the cluster reads as a mount
            // in the lower right instead of a wall.
            Transform {
                translation: Vec3::new(0.20, -0.22, -0.52),
                rotation: Quat::from_rotation_y(PI + 0.026),
                scale: Vec3::splat(0.62),
            },
            Visibility::Hidden,
        ))
        .set_parent(vm_root);
    let mech_pod = spawn_mech_pod_vm(&mut commands, &kit);
    commands
        .entity(mech_pod)
        .insert((
            // See the turret note: the pod box is chunkier still, and
            // at the old placement its near face was 0.26 m out and
            // filled half the frame.
            Transform {
                translation: Vec3::new(0.19, -0.20, -0.46),
                rotation: Quat::from_rotation_y(PI + 0.026),
                scale: Vec3::splat(0.72),
            },
            Visibility::Hidden,
        ))
        .set_parent(vm_root);
    commands.insert_resource(VmRig {
        root: vm_root,
        weapons: vm_weapons,
        shield: vm_shield,
        mech_turret,
        mech_pod,
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
    // §owner: the pod's pre-fire aim dots
    let mut rdots = Vec::new();
    for _ in 0..14 {
        rdots.push(
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
    commands.insert_resource(RocketAimVis(rdots));
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
    // §4.6 (Brief VIII): the crosshair is DRAWN, not typed. A zero-size
    // root at the exact screen centre (the old `+` glyph had to be nudged
    // to 49.6%/48.6% to fake centring, and still drifted with font
    // metrics), with ten absolutely-positioned children: five outlines
    // first so they sit BEHIND their five fills in the UI stack.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                ..default()
            },
            CrosshairRoot,
            HudRoot,
        ))
        .with_children(|c| {
            for outline in [true, false] {
                for idx in 0..CROSS_PIECES as u8 {
                    c.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        CrosshairPiece { idx, outline },
                    ));
                }
            }
        });
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
        HudRoot,
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
        HudRoot,
    ));
    // §9.1 (Brief IV): vertical weapon strip, right screen edge -
    // three guns plus the SHIELD essential on [4]
    for slot in 0..4usize {
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
            HudRoot,
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
        HudRoot,
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
            HudRoot,
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
        HudRoot,
    ));
    // §3.4: timer + score - TRUE top-center via a full-width centering
    // rail (data-driven top offset from HUD_ANCHORS)
    commands
        // Was a BARE `spawn(Node)` with no component of its own, so
        // nothing in the crate could query it - which is why the "5:00 /
        // BLUE 0-0 RED" line survived onto every menu screen.
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Percent(HUD_ANCHORS[3].2[1] * 100.0 - 1.5),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(branding::palette::PARCHMENT),
                ScoreTimerText,
            ));
        });
    // §4.7: the context progress bar - centred, ~58% down.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(CONTEXT_BAR_Y * 100.0),
                margin: UiRect::left(Val::Px(-CONTEXT_BAR_W_PX * 0.5)),
                width: Val::Px(CONTEXT_BAR_W_PX),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
            Visibility::Hidden,
            ContextBarRoot,
            HudRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont { font_size: 15.0, ..default() },
                TextColor(branding::palette::PARCHMENT),
                ContextBarLabel,
            ));
            // track
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(CONTEXT_BAR_H_PX),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                BorderRadius::all(Val::Px(2.0)),
            ))
            .with_children(|t| {
                // fill - width is driven every frame by `context_bar`
                t.spawn((
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(branding::palette::GOLD),
                    BorderRadius::all(Val::Px(2.0)),
                    ContextBarFill,
                ));
            });
        });

    // §4.5: the killfeed, as REAL ROWS.
    //
    // It used to be one flat `Text` with the whole feed newline-joined
    // into it, which structurally cannot satisfy the spec: a single Text
    // has ONE colour, so killer and victim names cannot take their own
    // team colours, and it has no box, so a local-player row cannot take
    // the 2px border the brief asks for. Five pre-spawned rows, each with
    // three coloured spans, can do both - and pre-spawning keeps it
    // allocation-free per frame, like every other pooled HUD element.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(-HUD_ANCHORS[4].2[0] * 100.0),
                top: Val::Percent(HUD_ANCHORS[4].2[1] * 100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(3.0),
                ..default()
            },
            FeedText,
            HudRoot,
        ))
        .with_children(|p| {
            for i in 0..KILLFEED_ROWS {
                p.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(5.0),
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                        border: UiRect::all(Val::Px(KILLFEED_BORDER_PX)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor(Color::NONE),
                    BorderRadius::all(Val::Px(4.0)),
                    Visibility::Hidden,
                    KillfeedRow(i),
                ))
                .with_children(|r| {
                    // killer | glyphs | victim - three spans so each
                    // name can carry its own side colour
                    for part in 0..3 {
                        r.spawn((
                            Text::new(""),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(branding::palette::PARCHMENT),
                            KillfeedCell(i, part),
                        ));
                    }
                });
            }
        });
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
            // 62%, one step below the context bar's 58% row - "BOARDING"
            // and a hit confirm used to overprint each other
            top: Val::Percent(62.0),
            ..default()
        },
        HitFeedText,
        HudRoot,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 34.0,
            ..default()
        },
        TextColor(branding::palette::GOLD),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(38.0),
            ..default()
        },
        BannerText,
        HudRoot,
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
            HudRoot,
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
            BackgroundColor(Color::srgba(0.10, 0.08, 0.06, 0.55)),
            HudRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 34.0,
                    ..default()
                },
                TextColor(branding::palette::PARCHMENT),
                // hard cap so no future status string can span the
                // screen and fold into the ammo corner again
                Node {
                    max_width: Val::Px(600.0),
                    ..default()
                },
                PanelInfoText,
            ));
            // §4.1: the depleting health bar, SEGMENTED. A solid bar
            // shows a ratio; a segmented one shows a COUNT - at ten
            // segments against a 100 HP pool each block is 10 HP, so a
            // glance answers "how many more of those can I take" without
            // reading, or trusting, the number beside it.
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    margin: UiRect::top(Val::Px(2.0)),
                    ..default()
                },
                VitalsBarRow,
            ))
            .with_children(|b| {
                for i in 0..VITALS_SEGMENTS {
                    b.spawn((
                        Node {
                            width: Val::Px(14.0),
                            height: Val::Px(7.0),
                            ..default()
                        },
                        BackgroundColor(branding::palette::PARCHMENT),
                        VitalsSeg(i),
                    ));
                }
            });
            // §4.1: the armour cluster, "to its right" in the brief's
            // layout - pips rather than a second bar, so armour and
            // health never read as the same quantity at a glance.
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(3.0),
                    margin: UiRect::top(Val::Px(3.0)),
                    ..default()
                },
                VitalsBarRow,
            ))
            .with_children(|b| {
                for i in 0..ARMOR_PIPS {
                    b.spawn((
                        Node {
                            width: Val::Px(11.0),
                            height: Val::Px(11.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor(branding::palette::BRONZE),
                        BorderRadius::all(Val::Px(2.0)),
                        BackgroundColor(Color::NONE),
                        ArmorPip(i),
                    ));
                }
            });
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
            BackgroundColor(Color::srgba(0.10, 0.08, 0.06, 0.55)),
            // the "30 / 120  FRAG x2" block the Settings panel collides
            // with in handback/brief-vii/menus/04-settings.png
            HudRoot,
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
            BackgroundColor(Color::srgba(0.10, 0.08, 0.06, 0.90)),
            Visibility::Hidden,
            ScoreboardRoot,
            HudRoot,
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
            HudRoot,
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
            // §owner: the reference reticle - THICK stadia posts running
            // in from the ring, thinning to the fine cross at centre.
            // Vertical pair
            for top in [true, false] {
                let mut n = Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    width: Val::Px(5.0),
                    height: Val::Percent(24.0),
                    margin: UiRect::left(Val::Px(-2.5)),
                    ..default()
                };
                if top {
                    n.top = Val::Percent(11.0);
                } else {
                    n.bottom = Val::Percent(11.0);
                }
                p.spawn((n, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95))));
            }
            // Horizontal pair
            for left in [true, false] {
                let mut n = Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(50.0),
                    height: Val::Px(5.0),
                    width: Val::Percent(17.0),
                    margin: UiRect::top(Val::Px(-2.5)),
                    ..default()
                };
                if left {
                    n.left = Val::Percent(21.0);
                } else {
                    n.right = Val::Percent(21.0);
                }
                p.spawn((n, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95))));
            }
            // §owner: the ILLUMINATED CENTRE. Every other firearm now
            // carries a red cross in a 1x optic; the scoped rifle is the
            // one sight that could not (its viewmodel is hidden while
            // zoomed), so its aiming mark lives here instead - same
            // colour, same shape, so the eye learns one thing.
            // The black stadia stay: red on its own vanishes against a
            // sunlit wall, black on its own vanishes in shadow.
            const SCOPE_RED: Color = Color::srgb(1.0, 0.16, 0.12);
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(2.0),
                    height: Val::Vh(5.4),
                    margin: UiRect::new(
                        Val::Px(-1.0),
                        Val::Px(0.0),
                        Val::Vh(-2.7),
                        Val::Px(0.0),
                    ),
                    ..default()
                },
                BackgroundColor(SCOPE_RED),
            ));
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Vh(5.4),
                    height: Val::Px(2.0),
                    margin: UiRect::new(
                        Val::Vh(-2.7),
                        Val::Px(0.0),
                        Val::Px(-1.0),
                        Val::Px(0.0),
                    ),
                    ..default()
                },
                BackgroundColor(SCOPE_RED),
            ));
        });

    // §4.3: AWARD TOASTS - stacked under the minimap, 2.5 s fade each.
    // The spec wrote them for a resource economy that does not exist;
    // they are driven by the award-worthy events the game DOES have -
    // kills, streaks, headshots, assists, captures - rather than faking
    // a currency.
    for i in 0..AWARD_SLOTS {
        commands.spawn((
            Text::new(""),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(branding::palette::GOLD),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                top: Val::Px(MINIMAP_TOP_PX + MINIMAP_PX + 10.0 + i as f32 * 20.0),
                ..default()
            },
            AwardToast(i),
            HudRoot,
        ));
    }

    // ---- minimap (M / settings to toggle) ------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                // TOP-left (owner). It sat bottom-left, which put it in
                // the same corner as the vitals cluster and left the top
                // -left anchor HUD_ANCHORS already reserves for it empty.
                top: Val::Px(MINIMAP_TOP_PX),
                width: Val::Px(MINIMAP_PX),
                height: Val::Px(MINIMAP_PX),
                border: UiRect::all(Val::Px(1.0)),
                // CLIP. In rotate-with-facing mode the world's corners
                // swing OUTSIDE the square - dots and rings were drawing
                // loose on the open HUD, which is the "glitched" look.
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.08, 0.06, 0.76)),
            BorderColor(branding::palette::BRONZE.with_alpha(0.55)),
            BorderRadius::all(Val::Px(4.0)),
            MinimapRoot,
            HudRoot,
        ))
        .with_children(|p| {
            // teammates (max 8) - WHITE squares
            for i in 0..8 {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(7.0),
                        height: Val::Px(7.0),
                        ..default()
                    },
                    BackgroundColor(branding::signal::ALLY),
                    Visibility::Hidden,
                    MinimapDot(i),
                ));
            }
            // §4.3: spotted enemies - hot dots, round (not square like
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
                    BackgroundColor(branding::signal::ENEMY_ACCENT),
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
                BackgroundColor(branding::signal::ALLY_ACCENT),
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
                    BackgroundColor(branding::signal::ALLY_ACCENT),
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
    tex: Res<TextureKit>,
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
        // tangents here too, or the ground's normal map is ignored -
        // and the ground is the surface that most needs the relief
        Mesh3d(meshes.add({
            let mut m: Mesh = Plane3d::default().mesh().size(half * 2.2, half * 2.2).into();
            let _ = m.generate_tangents();
            m
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: ground_c,
            base_color_texture: Some(tex.ground.clone()),
            normal_map_texture: Some(tex.ground_n.clone()),
            // the plane's UVs run 0..1 across its whole span, so without
            // a transform one 128px tile stretches over the entire map.
            // Tiled per ~4 m, which is about where grit stops reading as
            // grit and starts reading as noise.
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(half * 0.55)),
            perceptual_roughness: 1.0,
            ..default()
        })),
        CoverVis,
    ));
    let border_mat = materials.add(StandardMaterial {
        base_color: border_c,
        base_color_texture: Some(tex.stone.clone()),
        normal_map_texture: Some(tex.stone_n.clone()),
        // the border runs the whole map edge, so it needs the densest
        // tiling of anything - one course per couple of metres
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(half * 0.35, 1.0)),
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
        base_color_texture: Some(tex.wood.clone()),
        normal_map_texture: Some(tex.wood_n.clone()),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(1.6)),
        perceptual_roughness: 0.9,
        ..default()
    });
    let stone_mat = materials.add(StandardMaterial {
        base_color: match map {
            MapKind::Arena => Color::srgb(0.52, 0.48, 0.42),
            _ => Color::srgb(0.56, 0.56, 0.54),
        },
        base_color_texture: Some(tex.stone.clone()),
        normal_map_texture: Some(tex.stone_n.clone()),
        // cover blocks vary hugely in size; a modest tile keeps the
        // course height believable on a low wall without turning a big
        // one into gravel
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(1.25)),
        perceptual_roughness: 0.95,
        ..default()
    });
    let hedge_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.38, 0.14),
        // foliage borrows the GROUND generator, not a leaf one: its
        // broad mottling plus grit is exactly the clumped-and-speckled
        // read a hedge wants, and a fifth generator for one surface
        // would be a texture nobody could name
        base_color_texture: Some(tex.ground.clone()),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(3.0)),
        perceptual_roughness: 1.0,
        ..default()
    });
    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.24, 0.13),
        base_color_texture: Some(tex.wood.clone()),
        // a trunk wants its grain running up it, not around it
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(1.0, 3.0)),
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
                .clamp(-LOOK_PITCH_LIMIT, LOOK_PITCH_LIMIT);
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

    // SPACE jump, Q roll (or tap crouch at a sprint), 4 shield,
    // O or V first/third person, Z/X lean, 1-3 weapon slots, M minimap.
    // CTRL crouch, C armor ability, T inspect. Edge inputs latch until a
    // sim step runs. (The v6 mapping this comment used to describe -
    // LEFT aim / RIGHT-or-T fire - has been dead since Brief VI; its
    // leftovers were still being printed to the player in three places.)
    // §1 (Brief VI): CS:GO grammar - LEFT fires, always. RIGHT is the
    // ALT function: a two-stage zoom CYCLE on scoped weapons (Rule 2), a
    // draw on projectile weapons (bow/spear, Brief II grammar), and - as
    // of the owner's 2026-08-04 note - a HOLD-TO-FOCUS on everything
    // else. swap_mouse still swaps.
    //
    // WHY THIS OVERRIDES BRIEF VI. That brief gave standard guns no ADS
    // state at all, on CS:GO's authority. But CS:GO is first-person only,
    // and this game is THIRD-person primary: the hip camera sits 2.2 m
    // back and off the shoulder, so a distant target is both small and
    // parallaxed away from where the barrel points. CS:GO's grammar
    // solves an aiming problem this game does not have, and leaves the
    // one it does have - "which of those dots am I pointing at" -
    // unanswered. Focus is that answer.
    //
    // Nothing new is invented for it: `zoom_deg` already exists on all 11
    // guns, the third-person boom already eases in on `ads_t`
    // (`tp_boom_aim`), and the sim already applies ADS_SPREAD_MULT /
    // ADS_SPEED_MULT to any `cmd.ads`. The whole pipeline was built and
    // then fenced off at this one line.
    let (aim_btn, fire_btn) = mouse_map(settings.swap_mouse);
    let p_gun = game.sim.fighters[game.sim.player].gun;
    let scoped_gun = gun(p_gun).scoped;
    let alt_capable = scoped_gun || gun(p_gun).projectile.is_some();
    // §C.7 addendum: in FIRST person with the ROCKETS mount selected,
    // the aim hold IS the missile pre-aim (cmd.pod_aim below) - it must
    // not double as ADS, or targeting drags the visor into a zoom.
    // Third person keeps the normal focus pull-in unchanged. Gate on
    // `first_person` (the TARGET), not person_t: mid-blend flicker.
    let pod_aim_owns_rmb = cam.first_person
        && game.sim.fighters[game.sim.player].in_mech()
        && game.sim.fighters[game.sim.player].mech_weapon == sim::MechWeapon::Rockets;
    // §5.2 (Brief VI): scoped-class zoom is a two-stage CYCLE (40° →
    // 10° → out), and EVERY shot auto-unscopes - the bolt is cycled
    // out of the glass
    if scoped_gun && !pod_aim_owns_rmb && buttons.just_pressed(aim_btn) {
        cam.zoom_stage = (cam.zoom_stage + 1) % 3;
    }
    if !scoped_gun || pod_aim_owns_rmb {
        cam.zoom_stage = 0;
    }
    let pf = game.sim.fighters[game.sim.player].fire_cd;
    if scoped_gun && pf > cam.prev_fire_cd + 1e-6 {
        cam.zoom_stage = 0;
    }
    cam.prev_fire_cd = pf;
    // Scoped guns keep the click-to-cycle stages; everything else is a
    // plain hold. `alt_capable` no longer gates this - it only records
    // which weapons have the RICHER alt behaviour (draw / scope), which
    // the viewmodel and arc-preview systems still ask about.
    let ads = if pod_aim_owns_rmb {
        false
    } else if scoped_gun {
        cam.zoom_stage > 0
    } else {
        buttons.pressed(aim_btn)
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
    // the shield is inventory slot 4 now - an essential item beside the
    // three guns, not a verb key. Same toggle cmd downstream, so the
    // sim's contract (and its tests) never move.
    if keys.just_pressed(KeyCode::Digit4) {
        game.pending_shield = true;
    }
    // §5 (owner, revised): G PUTS THE GRENADE IN YOUR HAND.
    //
    // It used to only cycle which throwable was selected, and the actual
    // throw lived on a separate hold-H-and-release - two hands, three
    // keys, and no way to see what you were about to throw until you were
    // already committed to throwing it. Now:
    //
    //   G          equip / stow the throwable (tap again to put it away)
    //   RIGHT CLICK aim it - the arc preview appears
    //   LEFT CLICK  throw
    //
    // G on an ALREADY-equipped grenade cycles to the next type rather than
    // stowing, so the old cycling is still reachable without a second key.
    if keys.just_pressed(KeyCode::KeyG) {
        if game.nade_ready {
            game.pending_cycle_throw = true;
        } else {
            game.nade_ready = true;
        }
    }
    // B stows without throwing, and still cancels a live aim.
    if keys.just_pressed(KeyCode::KeyB) {
        game.nade_ready = false;
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
    // §3.6 (owner, 2026-08-04): SHIFT is the CS:GO WALK - slow, silent,
    // steady on the trigger. Sprint moves to ALT.
    //
    // The two could not share Shift. Sprint is a commitment the game
    // charges you for (the sprint-out fire gate, §3.4's carry) and walk
    // is the opposite commitment; a modifier that means "more speed" and
    // "less speed" depending on context is not a control, it is a coin
    // toss. Alt is the free key with no chord risk here: the window runs
    // with the cursor grabbed, and Alt is bound to nothing else in the
    // game.
    //
    // Walk is NOT gated on `!ads` the way sprint is. Focus-and-creep is
    // the exact combination the mechanic exists to enable.
    // §3.6 (owner, revised): SHIFT IS SPRINT AGAIN. The steady walk did
    // not need its own key - it needed a STANCE.
    //
    // Hold RIGHT-CLICK (focus) and press SHIFT and you drop into a
    // steady, silent walk. Then - and this is the point - LET GO OF
    // RIGHT-CLICK and you stay in it, for as long as Shift is held. So
    // the approach is: focus, settle, release the aim, and creep in.
    // Shift on its own, from a standing start, still sprints.
    //
    // Latched rather than held, because the whole use case is a long
    // quiet approach and holding two buttons for the length of one is
    // a hand cramp, not a mechanic.
    let shift = keys.pressed(KeyCode::ShiftLeft);
    if !shift {
        cam.steady = false; // releasing the key always ends the stance
    } else if ads {
        cam.steady = true; // RMB + Shift enters it
    }
    let steady = shift && cam.steady;
    let walking = steady;
    let sprinting = shift && !steady && !ads;
    if keys.just_pressed(KeyCode::KeyQ)
        || (sprinting && keys.just_pressed(KeyCode::ControlLeft))
    {
        game.pending_dodge = true;
    }
    // The click that throws also empties the hand - otherwise the flag
    // stays set and the next grenade starts cooking on its own.
    if game.nade_ready && buttons.just_pressed(fire_btn) {
        game.nade_ready = false;
    }
    let mut cmd = PlayerCmd {
        move_x: world.x,
        move_z: world.y,
        sprint: sprinting,
        walk: walking,
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
        // §5: the grenade is HELD from the moment it is equipped, and
        // leaves on the falling edge - which is what the sim already
        // watches for. So LEFT CLICK throws simply by dropping this
        // false for one tick.
        //
        // H and Mouse4 stay wired as the old hold-to-cook, so muscle
        // memory and the capture scripts both keep working.
        throw_hold: (game.nade_ready && !buttons.just_pressed(fire_btn))
            || keys.pressed(KeyCode::KeyH)
            || buttons.pressed(MouseButton::Back),
        // §1 (Brief V): B cancels the aimed throw, grenade kept
        throw_cancel: keys.just_pressed(KeyCode::KeyB),
        // §4.6 (Brief VI): U dismounts the mech
        exit_mech: keys.just_pressed(KeyCode::KeyU),
        // §5.3 (Brief VI) / §C.7 (Brief VIII): missile targeting is pure
        // PRE-AIM now - lock accrual and the amber arc; the launch is
        // LMB through the ROCKETS mount. With that mount selected, RMB
        // is the targeting hold (the finger that already means "aim"
        // everywhere else); Y stays wired for muscle memory and the
        // capture scripts.
        pod_aim: keys.pressed(KeyCode::KeyY)
            || (buttons.pressed(MouseButton::Right)
                && game.sim.fighters[game.sim.player].in_mech()
                && game.sim.fighters[game.sim.player].mech_weapon
                    == sim::MechWeapon::Rockets),
        cycle_throw: game.pending_cycle_throw,
        // §5/§6 (Brief III): F is the KNIFE now; the armor ability
        // (brace / flame / repulsor) moved to held C
        ability: keys.pressed(KeyCode::KeyC),
        knife_hold: keys.pressed(KeyCode::KeyF),
    };

    // §C: the player's shot clock, whichever weapon they are actually
    // firing - a mech pilot's shots run on the hull mount's own cycle,
    // not `fire_cd`. See `shot_clock`.
    let prev_fire_cd = shot_clock(&game.sim.fighters[game.sim.player]);
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
    if p.alive() && shot_clock(p) > prev_fire_cd {
        // §C: a hull mount's kick comes from its OWN identity, not from
        // whatever rifle the pilot happens to be carrying. Reading
        // `gun(p.gun).kick` inside a mech was the same class of mistake
        // as `apply_hit` re-reading the held gun's damage - the pilot's
        // inventory silently driving a hull weapon's feel.
        let kick = if p.in_mech() {
            match p.mech_weapon {
                // the gatling's mass eats its own recoil; heat is its cost
                sim::MechWeapon::Gatling => 0.0016,
                // the autocannon is the reason the brace stance exists
                sim::MechWeapon::Autocannon => 0.0180,
                // §C.7: a launch is a THUMP, not a crack - between the
                // two gun mounts, nearer the belt than the cannon
                sim::MechWeapon::Rockets => 0.0110,
            }
        } else {
            gun(p.gun).kick
        };
        // Whatever the sim damps, the camera damps. The sim scales a
        // braced mech's punch by MECH_BRACE_RECOIL_DAMP (sim.rs, the
        // spray block) but this path only ever asked about `lean`, so a
        // planted mech soaked its recoil in the simulation and still took
        // the full kick in the view - the brace stance visibly did
        // nothing for the thing the player actually feels. A pilot does
        // not lean; on foot you cannot mech-brace. The two are exclusive,
        // so this reads as one ladder rather than two.
        let brace = if p.in_mech() {
            if p.mech_brace {
                sim::MECH_BRACE_RECOIL_DAMP
            } else {
                1.0
            }
        } else if p.lean.abs() > 0.1 {
            LEAN_RECOIL_MULT
        } else {
            1.0
        };
        // Clamp to the SAME limit the mouse obeys. Recoil clamps the
        // accumulated pitch, not just its own delta, so a narrower limit
        // here does not "restrain the kick" - it teleports an aim the
        // player legitimately held into range the instant they pull the
        // trigger.
        // §2: this channel has to obey the same two exemptions the
        // sim's punch does, or the view kicks for a shot the simulation
        // never punched. A drawn bow and a thrown spear produce NO
        // punch (sim.rs gates the spray block on `projectile.is_none()`)
        // yet were permanently walking the player's aim up the screen;
        // and a scoped rifle's punch is scaled to 25/78 while the view
        // took the full 3 degrees per shot.
        let spec_now = gun(p.gun);
        if spec_now.projectile.is_none() && p.gun != GunKind::Fists {
            let scoped_scale = if spec_now.scoped && cam.ads {
                25.0 / 78.0
            } else {
                1.0
            };
            // §owner: the aim only starts climbing once the burst has
            // earned it. Same `spray_ramp` the sim's punch and bloom
            // read, so tapping three rounds walks the crosshair
            // essentially nowhere and holding the trigger does.
            //
            // `spray_i` has ALREADY been advanced by the shot being
            // reacted to here, so step back one to score the round that
            // actually fired rather than the next one.
            let fired_i = (p.spray_i.max(1.0) as usize).saturating_sub(1);
            let ramp = sim::spray_ramp(fired_i);
            cam.pitch = recoil_kicked_pitch(
                cam.pitch,
                kick * scoped_scale * ramp,
                p.bloom * ramp,
                brace,
            );
            cam.recoil = (cam.recoil + 0.6).min(1.0);
        }
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
    // the draw, handed to `bow_string_sync` - see `BowDraw`
    mut bow_draws: Query<&mut BowDraw>,
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
        // §owner ATHLETIC MOTION: a sprinter driving off the line leans
        // 28-32 degrees into it. This was clamped to 0.07 rad - FOUR
        // degrees - which is a nod, not an acceleration.
        //
        // Asymmetric on purpose: you throw yourself forward far harder
        // than you rock back stopping, and a symmetric clamp made hard
        // braking look like a stumble.
        //
        // The head-band law still rules it. `gait_pose` derives the hip
        // height needed to keep the head base on the 0.82 hit line from
        // whatever pitch it is handed, so a deeper lean RAISES the hip
        // to compensate rather than dragging the head out of its band.
        // That is what makes this safe to open up: the invariant is
        // enforced downstream, not by this clamp.
        let lean_target = (accel * 0.055).clamp(-0.14, ATHLETIC_LEAN_MAX);
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
        for (li, (leg, off)) in [(rig.leg_l, 0.0_f32), (rig.leg_r, PI)]
            .into_iter()
            .enumerate()
        {
            let (thigh, shin, foot);
            // §B.1 #19-20: plantar flexion of the forefoot, positive =
            // toes pushing DOWN and back.
            let mut toe;
            if rolling {
                // tucked ball
                thigh = -1.55;
                shin = 2.30;
                foot = -0.55;
                toe = 0.0;
            } else if f.crouch && !airborne {
                // half-squat under the §0.2-tuned crouch lean
                thigh = -0.85;
                shin = 1.55;
                foot = -0.42;
                toe = 0.0;
            } else if airborne {
                // jump tuck
                thigh = -0.85;
                shin = 1.50;
                foot = -0.40;
                // the forefoot POINTS in the air. Free of the ground it
                // has nothing to push against, and a flat foot mid-jump
                // is the tell that a leg is a prop.
                toe = TOE_OFF_MAX * 0.45;
            } else {
                // the gait: elliptical foot path (60% stance / 40% swing
                // read), knee bends hardest on recovery, and the ankle
                // rolls heel-strike → toe-off - the *human* read
                let sw = (rig.phase + off).sin() * amp;
                let lift = (rig.phase + off + 0.9).sin().max(0.0) * amp;
                // §owner ATHLETIC MOTION: KNEE DRIVE. The recovery knee
                // used to peak near 49 deg, which is a jog. A sprinter
                // folds the heel up toward the seat - the brief asks for
                // ~130 deg, and the drive is what sells speed far more
                // than stride length does.
                //
                // Scaled by `amp` (speed fraction), so a walk keeps its
                // old shallow bend and only a genuine sprint folds hard.
                thigh = sw * 0.9;
                shin = 0.10 + lift * KNEE_DRIVE_MAX;
                foot = -(thigh + shin) * 0.55
                    + (rig.phase + off - 1.2).sin() * 0.20 * amp
                    + (rig.phase + off + 0.6).sin() * 0.22 * amp; // ankle roll
                // §B.1 #19-20 TOE-OFF. "No toe rotation means the run is
                // still a glide" - and it was, because there was nothing
                // forward of the ankle to push with.
                //
                // The snap fires at CONTACT EXIT: the instant the heel
                // has left and the whole body is going over the ball of
                // the foot. That is the back of the stance phase, which
                // in this gait's phase convention is where the leg is
                // furthest behind - so the drive is keyed to the same
                // sine the stride is, a quarter-cycle late, and rectified
                // because a toe pushes and never pulls.
                //
                // Scaled by `amp`, so a walk gets a roll and only a real
                // sprint gets a snap.
                toe = toe_off_angle(rig.phase + off, amp);
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
            // the forefoot. Clamped at the joint rather than at every
            // source, so no branch above can hyperextend a toe.
            toe = toe.clamp(0.0, TOE_OFF_MAX);
            if let Ok((mut t, _)) = parts.get_mut(rig.toes[li]) {
                t.rotation = Quat::from_rotation_x(toe);
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
        // §owner SUPPRESSION: the sim now measures rounds passing close,
        // for every fighter, which is a strictly better source than this
        // client-side arrow proximity check - it covers BULLETS, which are
        // what suppression is actually made of. The arrow check stays as
        // the floor: a spear going past your head at 60 m/s deserves a
        // flinch whether or not the sim scored it, and `max` means neither
        // source can cancel the other.
        //
        // The flinch is the same one it always was: this feeds the body's
        // shake, nothing else. What it does NOT do is touch the player's
        // aim - see `Fighter::suppress_t`.
        ls.suppress_t = ls
            .suppress_t
            .max((f.suppress_t / sim::SUPPRESS_MAX_S).clamp(0.0, 1.0) * 0.28);
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
        // §B.2 THE TRUNK, IN THREE.
        //
        // The whole twist used to land on one joint. It is split here:
        // the LUMBAR takes `LUMBAR_TWIST_SHARE` of the yaw the trunk is
        // asked for and the THORAX takes the rest, so a coil bends
        // through a spine instead of pivoting on a hinge. Their sum is
        // exactly what the single joint carried before, which is what
        // keeps the separation test - and the ±60° additive-aim contract
        // it guards - reading the same total.
        //
        // Position and pitch stay on the thorax. Only the TWIST is shared
        // out: the lumbar is the twist segment §B.2 names, and giving it
        // a share of the sway and the breathing bob as well would be
        // double-counting motion the thorax already applies.
        let trunk_yaw = 0.045 * rig.phase.sin() * amp + spear_yaw + torso_aim;
        if let Ok((mut t, _)) = parts.get_mut(rig.lumbar) {
            t.rotation = Quat::from_rotation_y(trunk_yaw * LUMBAR_TWIST_SHARE);
        }
        if let Ok((mut t, _)) = parts.get_mut(rig.torso) {
            // §1.4 pelvis layers: lateral sway toward the stance foot,
            // pelvis yaw with the spine counter-rotating most of it back
            // (net ±1.5° - which is also all the arm swing an armed
            // carry gets: the upper body moves through the spine, not
            // the shoulders)
            // §B.2: MINUS the waist, because the thorax now hangs off the
            // lumbar and the lumbar already carries it. `hip_y` is a
            // root-space height and has to stay one - it is the same
            // number the legs are posed with.
            t.translation.y = thorax_local_y(
                hip_y,
                if f.crouch && !rolling { 0.12 } else { 0.0 },
                breath,
            );
            // amplitudes tuned UP so the gait reads from the 2.6 m boom
            t.translation.x = 0.048 * rig.phase.sin() * amp + wshift;
            // §5.2: `torso_aim` puts the shoulders back on the aim that
            // the legs have not caught up to yet - the turn-in-place read
            t.rotation = Quat::from_rotation_y(trunk_yaw * (1.0 - LUMBAR_TWIST_SHARE))
                * Quat::from_rotation_x(
                    (torso_pitch + flinch + 0.07 * settle + relaxed_e * 0.05).min(0.185),
                )
                * Quat::from_rotation_z(sway_r);
        }
        // §B.1 #5-6: the shoulder girdle TRAVELS. `rig.clav` is the same
        // spring it always was; what is new is that it now moves a BONE
        // the arm hangs from, rather than only nudging an IK target the
        // arm was solved toward. The rest offset is baked into the
        // clavicle's spawn transform, so this is a pure delta.
        // Added the SAME to both sides, not mirrored - `clav` is a
        // shift of the whole girdle (the shoulders travelling together
        // through a turn or a throw), which is why `sh_l` and `sh_r` both
        // add it unmodified. Mirroring it here would have the shoulders
        // shrugging toward each other instead.
        for (ci, sign) in [(0usize, -1.0_f32), (1, 1.0)] {
            if let Ok((mut t, _)) = parts.get_mut(rig.clavicles[ci]) {
                let c = if rig.clav.is_nan() { Vec3::ZERO } else { rig.clav };
                t.translation = Vec3::new(sign * SHOULDER_X, 0.62, 0.02) + c;
            }
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
                    && !(f.shield_up && ALL_WEAPONS[wi] == GunKind::Bow)
                    // the hull mount IS the weapon; the rifle is stowed
                    && !in_mech;
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
        // The string follows the sim's draw clock, not the ADS toggle it
        // used to guess from. See `bow_draw_visual`.
        let bow_draw = if f.gun == GunKind::Bow {
            bow_draw_visual(
                f.bow_draw_t,
                f.fire_cd,
                gun(GunKind::Bow).fire_period,
                is_player,
            )
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
            // §owner: a JAVELIN THROW, in two beats you can read from
            // across the field.
            //
            // The old motion travelled about six centimetres and rotated
            // a little - technically present, invisible in play. A throw
            // is the most telegraphed action in the game and SHOULD be:
            // it is a committed, single-shot, empty-your-hands decision,
            // and the enemy is entitled to see it coming.
            //
            // COCK (first ~65% of the wind): the arm hauls the shaft up
            // and back over the shoulder, out to the side of the head.
            // WHIP (last ~35%): the hips are already opening under it
            // (`torso_coil_yaw`), and the arm drives through hard, past
            // neutral, so the release looks thrown rather than dropped.
            let wp = if f.spear_wind_t > 0.0 {
                (1.0 - f.spear_wind_t / SPEAR_WINDUP_S).clamp(0.0, 1.0)
            } else {
                0.0
            };
            const COCK_FRAC: f32 = 0.65;
            let (cock, whip) = if wp < COCK_FRAC {
                (ease_out(wp / COCK_FRAC), 0.0)
            } else {
                (1.0, ease_out((wp - COCK_FRAC) / (1.0 - COCK_FRAC)))
            };
            (
                Vec3::new(
                    0.16 + 0.10 * cock - 0.06 * whip,
                    0.72 + 0.17 * cock - 0.10 * whip,
                    0.02 - 0.34 * cock + 0.78 * whip,
                ),
                Quat::from_rotation_x(
                    wr_pitch - 1.35 - 0.62 * cock + 1.45 * whip + jerk * 1.5,
                ),
            )
        } else if f.gun == GunKind::Bow {
            // The bow stands in front of the LEFT side, and it COMES UP as
            // it is drawn.
            //
            // That rise is what makes a horizontal bow readable. Held flat
            // at rest the string plane is a chest-height line and the draw
            // hand ends at the sternum, which is a real hold but a dull
            // one. Bringing the riser up to just under the shoulder puts
            // the anchor at the armpit-to-jaw line, which is the pose an
            // archer actually finishes in - and it does it WITHOUT tilting
            // the bow, because the whole point of the horizontal hold is
            // that it stays horizontal.
            (
                Vec3::new(-0.04, 0.48 + 0.11 * bow_draw, 0.16 - 0.02 * bow_draw),
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
        // §2.5 CLAVICLE (k=45): a real shoulder is not a fixed pivot -
        // it PROTRACTS as the arm reaches across or forward, and it is
        // the slowest link in the chain, which is why it gets the
        // softest spring in the table. Without it the arm rotates out of
        // a socket bolted to the ribcage, which is the other half of the
        // "keyframed" look the hand spring fixes.
        //
        // Driven by how far the grip sits from a neutral carry: reaching
        // forward or across pulls the shoulder after it, a little.
        let reach = grip_t.map_or(Vec3::ZERO, |t| t - Vec3::new(0.14, 0.50, 0.20));
        let clav_target = Vec3::new(
            reach.x.clamp(-0.09, 0.09) * 0.30,
            reach.y.clamp(-0.12, 0.12) * 0.22,
            reach.z.clamp(-0.12, 0.16) * 0.26,
        );
        let clav = if rig.clav.is_nan() {
            rig.clav = clav_target;
            clav_target
        } else {
            let (nx, nv) =
                damped_spring3(rig.clav, rig.clav_v, clav_target, SPRING_K_SHOULDER, dt);
            rig.clav = nx;
            rig.clav_v = nv;
            nx
        };
        let sh_l = Vec3::new(-0.26, 0.62, 0.02) + clav;
        let sh_r = Vec3::new(0.26, 0.62, 0.02) + clav;
        let pole_l = Vec3::new(-0.574, -0.80, 0.15); // down-and-out 35°
        let pole_r = Vec3::new(0.574, -0.80, 0.15);
        // (shoulder quat, elbow flex, wrist pitch) per arm
        // §owner ATHLETIC MOTION: ARM DRIVE. A sprinting soldier drives
        // the arms hard and bends the elbow toward a right angle; the
        // old +-17 deg with a straight arm read as a stroll.
        //
        // NOTE this is the UNARMED pose. An armed fighter's arms are
        // overridden below by the weapon-grip IK, which is correct -
        // you cannot pump a rifle like a sprinter's fist - so the drive
        // shows on empty hands and on the off hand.
        let arm_bend = 0.15 + 0.85 * amp.clamp(0.0, 1.0);
        let mut left = (
            Quat::from_rotation_x(swing * ARM_DRIVE_SWING),
            arm_bend,
            0.0_f32,
        );
        let mut right = (
            Quat::from_rotation_x(-swing * ARM_DRIVE_SWING),
            arm_bend,
            0.0_f32,
        );
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
                    // §3.3: a REAL anchor - and it is the STRING's anchor,
                    // not a guess at one.
                    //
                    // The hand target comes out of `bow_nock_local`, the
                    // same function that places the string halves and the
                    // arrow, offset only by where fingers sit relative to
                    // the cord they hook. It cannot be off the string,
                    // because "on the string" is now the only thing this
                    // expression can say.
                    //
                    // What it replaces was three literals - x 0.03, y
                    // 0.14·draw, z -0.09 - 0.18·draw - tuned against the
                    // VERTICAL bow. A vertical string spans Y, so lifting
                    // the hand 14 cm still had it touching string. Turned
                    // horizontal, that same lift is a hand held a palm's
                    // width clear of the cord, pulling nothing.
                    //
                    // The follow-through survives: on release the hand
                    // flies back past the anchor, which is what sells a
                    // loose as a release rather than a fade.
                    let release_fly = if jerk > 0.55 { (jerk - 0.55) * 0.5 } else { 0.0 };
                    let nock = wr_pos
                        + wr_rot
                            * (bow_nock_local(bow_draw)
                                + BOW_HAND_OFF
                                + Vec3::new(0.0, 0.0, -release_fly));
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
                    //
                    // §2.5: the socket is a HARD SNAP - it is wherever
                    // the weapon is this frame - so the hand springs onto
                    // it (k=120) and the elbow pole follows more slowly
                    // (k=60). That difference is the whole read: the hand
                    // arrives first and the elbow swings in after it,
                    // which is what stops an arm looking keyframed.
                    if let Some(t) = grip_t {
                        let sprung = if rig.hand_r.is_nan() {
                            rig.hand_r = t; // first pose snaps, never springs in
                            t
                        } else {
                            let (nx, nv) = damped_spring3(
                                rig.hand_r,
                                rig.hand_r_v,
                                t,
                                SPRING_K_HAND_FOLLOW,
                                dt,
                            );
                            rig.hand_r = nx;
                            rig.hand_r_v = nv;
                            nx
                        };
                        let pole = if rig.pole_r_s.is_nan() {
                            rig.pole_r_s = pole_r;
                            pole_r
                        } else {
                            let (nx, nv) = damped_spring3(
                                rig.pole_r_s,
                                rig.pole_r_v,
                                pole_r,
                                SPRING_K_ELBOW_POLE,
                                dt,
                            );
                            rig.pole_r_s = nx;
                            rig.pole_r_v = nv;
                            nx
                        };
                        let (q, e) = solve_arm_ik(sh_r, sprung, pole);
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
                    let sprung_l = if rig.hand_l.is_nan() {
                        rig.hand_l = lt;
                        lt
                    } else {
                        let (nx, nv) =
                            damped_spring3(rig.hand_l, rig.hand_l_v, lt, SPRING_K_HAND_FOLLOW, dt);
                        rig.hand_l = nx;
                        rig.hand_l_v = nv;
                        nx
                    };
                    let pole_ls = if rig.pole_l_s.is_nan() {
                        rig.pole_l_s = pole_l;
                        pole_l
                    } else {
                        let (nx, nv) =
                            damped_spring3(rig.pole_l_s, rig.pole_l_v, pole_l, SPRING_K_ELBOW_POLE, dt);
                        rig.pole_l_s = nx;
                        rig.pole_l_v = nv;
                        nx
                    };
                    let (q, e) = solve_arm_ik(sh_l, sprung_l, pole_ls);
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
                // §owner MELEE v2: the wind COCKS to the line the blade
                // will travel, because this is the frame the defender
                // has to read to answer it. A left swing draws back over
                // the left shoulder, a right over the right, an overhead
                // straight up beside the ear as before.
                //
                // Third person carries this as much as first: the
                // attacker's silhouette is the only tell the defender
                // actually gets.
                let e = ease_out((ph / w).clamp(0.0, 1.0));
                let cocked = match f.knife_dir {
                    sim::MeleeDir::Left => Vec3::new(-0.34, 0.74, -0.02),
                    sim::MeleeDir::Right => Vec3::new(0.52, 0.72, 0.02),
                    sim::MeleeDir::Overhead => Vec3::new(0.36, 0.80, -0.06),
                };
                Vec3::new(0.16, 0.44, 0.28).lerp(cocked, e)
            } else {
                // strike snaps ACROSS its own line, then eases back to
                // the guard - a left swing finishes right and vice versa
                let active = if f.melee_axe {
                    AXE_QUICK_ACTIVE_S + AXE_QUICK_RECOVER_S
                } else {
                    KNIFE_QUICK_ACTIVE_S + KNIFE_QUICK_RECOVER_S
                };
                let r = ((ph - w) / active).clamp(0.0, 1.0);
                let hit = match f.knife_dir {
                    sim::MeleeDir::Left => Vec3::new(0.46, 0.30, 0.40),
                    sim::MeleeDir::Right => Vec3::new(-0.40, 0.28, 0.42),
                    sim::MeleeDir::Overhead => Vec3::new(-0.26, 0.26, 0.44),
                };
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
        // Hand the draw to `bow_string_sync`, which owns the string and
        // the arrow on it. The arrow used to hang off `arm_l[2]` - the BOW
        // hand - at a fixed offset, so it tracked the hand and not the
        // bow, and never moved with the draw at all.
        if let Some(bow_slot) = weapon_slot(GunKind::Bow) {
            if let Ok(mut d) = bow_draws.get_mut(rig.weapons[bow_slot]) {
                d.pull = bow_draw;
                d.nocked = arrow_vis != Visibility::Hidden;
            }
        }
        // §6: the powered shell shows while the Robot Suit is worn
        if let Ok((_, mut v)) = parts.get_mut(rig.armor_rig) {
            *v = if f.armor_set == ArmorSet::RobotSuit {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        // The mech is HEADLESS and ARMLESS - a walking weapons platform,
        // not a big soldier (Brief VIII-B D.1). Transforms still animate;
        // only visibility gates, so the band and connectivity tests are
        // untouched. `neck` is the head pivot: head shell, eyes and hat
        // are its descendants and inherit the hide.
        let suit = f.armor_set == ArmorSet::RobotSuit;
        let body_vis = if suit {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        for e in [rig.neck, rig.arm_l[0], rig.arm_r[0]] {
            if let Ok((_, mut v)) = parts.get_mut(e) {
                *v = body_vis;
            }
        }
        for legs in rig.mech_leg_armor {
            for e in legs {
                if let Ok((_, mut v)) = parts.get_mut(e) {
                    *v = if suit {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
            }
        }
        // §6.3 / D.6: parts shear off as the sim's threshold bits latch.
        // The bitmask is SIM truth (replay-identical); this is pure
        // presentation - its first render-side consumer.
        for (bit, list) in [
            (0b001u8, &rig.mech_detach_70[..]),
            (0b010u8, &rig.mech_detach_40[..]),
            (0b100u8, &rig.mech_detach_15[..]),
        ] {
            let gone = suit && f.mech_plates_dropped & bit != 0;
            for &e in list {
                if let Ok((_, mut v)) = parts.get_mut(e) {
                    *v = if gone {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    };
                }
            }
        }
    }
}

fn sync_tracers(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    cam_ctl: Res<CamCtl>,
    cam_q: Query<&GlobalTransform, With<MainCam>>,
    mut pool: ResMut<TracerPool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<TracerMarker>>,
) {
    // The local player's own streaks, while in first person, are drawn
    // from the WEAPON on screen instead of from the eye the ray was cast
    // from. Everyone else's - and their own in third person - keep the
    // sim's origin, which is already at the right place on the body.
    let fp_origin: Option<Vec3> = if cam_ctl.first_person {
        cam_q.get_single().ok().map(|g| {
            let p = &game.sim.fighters[game.sim.player];
            g.transform_point(fp_muzzle_local(p))
        })
    } else {
        None
    };
    while pool.0.len() < game.sim.tracers.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.tracer_mesh.clone()),
                // placeholder - the per-tracer pass below assigns the
                // real side material before this is ever made visible
                MeshMaterial3d(assets.tracer_ally.clone()),
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
                let a = match fp_origin {
                    // our own shot, seen down our own sights: start it at
                    // the muzzle that is actually on screen
                    Some(m) if tr.shooter == game.sim.player => m,
                    _ => a0 + (seg / sl) * (0.45_f32).min(sl * 0.4) - Vec3::Y * 0.12,
                };
                let mid = (a + b) * 0.5;
                let len = (b - a).length().max(0.05);
                let dir = (b - a) / len;
                *tf = Transform::from_translation(mid)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(1.0, 1.0, len));
                *vis = Visibility::Visible;
                let p_team = game.sim.fighters[game.sim.player].team;
                let mat = match branding::signal::side_of(tr.team, p_team) {
                    branding::signal::Side::Ally => assets.tracer_ally.clone(),
                    branding::signal::Side::Enemy => assets.tracer_enemy.clone(),
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
    time: Res<Time>,
    game: Res<Game>,
    kit: Res<ModelKit>,
    mut pool: ResMut<MissilePool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<MissileMarker>>,
    mut q2: Query<(&mut Transform, &mut Visibility), Without<MissileMarker>>,
) {
    while pool.0.len() < game.sim.missiles.len() {
        let root = commands
            .spawn((Transform::IDENTITY, Visibility::Hidden, MissileMarker))
            .id();
        let (arrow, spin) = spawn_arrow_model(&mut commands, &kit);
        commands.entity(arrow).set_parent(root);
        let spear = spawn_spear_model(&mut commands, &kit);
        commands.entity(spear).set_parent(root);
        pool.0.push(MissileSlot { root, arrow, spear, spin });
    }
    let dt = time.delta_secs().min(0.05);
    for (idx, slot) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis)) = q.get_mut(slot.root) else {
            continue;
        };
        match game.sim.missiles.get(idx) {
            Some(m) => {
                let dir = Vec3::from_array(m.vel).normalize_or(Vec3::Z);
                // §owner: a shaft in flight points where it is GOING, and
                // a stuck one keeps the angle it bit at (the sim leaves
                // `vel` intact on impact precisely so this still reads).
                let (len, thick) = if m.is_spear { (1.9, 1.0) } else { (0.85, 1.0) };
                *tf = Transform::from_translation(Vec3::from_array(m.pos))
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(thick, thick, len));
                *vis = Visibility::Visible;
                // one slot, two shapes
                for (e, on) in [(slot.arrow, !m.is_spear), (slot.spear, m.is_spear)] {
                    if let Ok((_, mut v)) = q2.get_mut(e) {
                        *v = if on {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        };
                    }
                }
                // fletching spin - vanes bite the air and roll the shaft,
                // which is what makes an arrow FLY instead of drift. It
                // stops the moment the shaft does.
                if !m.is_spear && m.stuck_t.is_none() {
                    if let Ok((mut st, _)) = q2.get_mut(slot.spin) {
                        st.rotation *= Quat::from_rotation_z(ARROW_SPIN_RAD_S * dt);
                    }
                }
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
    state: Res<State<GameState>>,
    mut roots: Query<(&HealthBarVis, &mut Transform, &mut Visibility), Without<BarFill>>,
    mut fills: Query<
        (&mut Transform, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>),
        (With<BarFill>, Without<HealthBarVis>),
    >,
) {
    // These are WORLD-space bars, so `HudRoot` cannot reach them - they
    // are not UI nodes. They are why green bars floated over the NPCs
    // behind the Intro and Settings screens. Gated here rather than by
    // pulling the system out of its `.chain()`, because its ordering
    // against the other sync systems is load-bearing.
    let show_bars = hud_visible(state.get());
    for (hb, mut tf, mut vis) in &mut roots {
        // same deploy-frame safety as the rigs: stale bars must not panic
        let Some(f) = game.sim.fighters.get(hb.index) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let self_view =
            cam.person_t < 0.5 || (cam.ads && gun(f.gun).scoped && !f.shield_up);
        if !show_bars || !f.alive() || (hb.index == game.sim.player && self_view) {
            // dead men carry no bar; in first person (or scoped glass)
            // neither do YOU - the HUD panel already shows your numbers;
            // and nobody does while a menu is up
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
    //
    // §B.3: a pilot sees from the VISOR, not from where their own head
    // would be. The hull puts the eye at 90% of a 1.7x chassis - well
    // above infantry eye height - which is the whole reason a mech
    // feels like a different vehicle rather than a tall soldier.
    // Lean shift is deliberately NOT applied in a mech: the pilot is
    // strapped into a hull that does not peek.
    let eye = if p.in_mech() {
        Vec3::new(p.pos[0], sim::mech_visor_eye_y(p.pos[1]), p.pos[2])
    } else {
        let eye_h = (p.height() - 0.16).max(0.55);
        Vec3::new(p.pos[0], p.pos[1] + eye_h, p.pos[2])
            + screen_right * (p.lean * LEAN_SHIFT)
    };

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
        // Addendum §A: the radius a footfall is FELT scales with the
        // chassis. 6.0 m was tuned at the old 1.15x scale and never
        // followed it to 1.7 - a machine half again as big should shake
        // the ground further. Derived, so the next scale change follows.
        let felt = (6.0 / 1.15) * MECH_SCALE;
        if d < felt {
            // periodic thump matched to the walk cadence
            let pulse = (game.sim.t * std::f32::consts::TAU * 1.7).sin().max(0.0).powi(6);
            shake += 0.2 * 0.09 * (1.0 - d / felt) * pulse;
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

    // §4.7: death -> killer-cam -> spectate. Overrides the composed
    // transform LAST, so every death reuses the same camera plumbing
    // (FOV, projection, shake) and only the framing changes. Suicides
    // and missing killers fall through to the ordinary corpse view -
    // there is nobody worth watching.
    if let Some((k, spectating)) = death_phase(&game.sim, game.sim.player) {
        let kf = &game.sim.fighters[k];
        let khead = Vec3::new(kf.pos[0], kf.pos[1] + kf.height() * 0.8, kf.pos[2]);
        if !spectating {
            // killer-cam: stay AT the corpse, turn to face who did it.
            // The translation composed above is already the corpse eye.
            tf.look_at(khead, Vec3::Y);
        } else {
            // spectate: a third-person boom on the killer, behind their
            // facing - "SPECTATING <name>" in the HUD names them. Scale
            // the boom with their height so a mech is framed, not filled.
            let back = Vec3::new(kf.yaw.sin(), 0.0, kf.yaw.cos());
            let boom = 2.6 * (kf.height() / BODY_HEIGHT);
            tf.translation = khead - back * boom + Vec3::Y * (0.4 * boom);
            tf.look_at(khead, Vec3::Y);
        }
    }

    // §3.4: FOV rides ads_t (ease-out, framerate-independent) - never the
    // `+= (target-fov)*k` exponential that stalls and never arrives
    if let Projection::Perspective(persp) = &mut *proj {
        // §5.2 (Brief VI): scoped-class two-stage zoom - 40° then 10°
        let zoom = if p.armed() && !p.shield_up {
            if gun(p.gun).scoped && cam_ctl.zoom_stage == 2 {
                10.0
            } else if gun(p.gun).projectile.is_some() {
                // §owner: drawing a bow or cocking a spear TIGHTENS THE
                // AIM - it does not zoom the world. The zoom was hiding
                // the arc and making the draw feel like a scope instead
                // of a weapon coming up to full power.
                settings.fov_deg()
            } else {
                gun(p.gun).zoom_deg
            }
        } else {
            settings.fov_deg()
        };
        // §5.1: THIRD-PERSON AIM also pulls the FOV in, on top of
        // whatever the weapon asked for. First person is untouched -
        // there the weapon's own zoom is the whole story, and stacking
        // a boom-camera delta on it would double-count.
        let zoom = if cam_ctl.person_t > 0.5 {
            zoom + TP_FOV_AIM_DELTA
        } else {
            zoom
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
#[derive(Component)]
struct TurntableCard;

/// §7.2: attach the turntable card to the intro AFTER both exist.
///
/// It cannot be spawned inside `open_intro`: the app boots into Intro and
/// that first `OnEnter` runs before `setup` has built the stage - the
/// same startup-ordering trap that once cost the intro its key art. A
/// late attach needs no ordering at all: whenever an IntroRoot exists
/// with no card and the stage is ready, the card appears.
fn attach_turntable_card(
    mut commands: Commands,
    fp: Option<Res<ForgePreview>>,
    intro: Query<Entity, With<IntroRoot>>,
    existing: Query<(), With<TurntableCard>>,
) {
    let Some(fp) = fp else { return };
    if !existing.is_empty() {
        return;
    }
    let Ok(root) = intro.get_single() else { return };
    // One card per page that wants one. They share the SAME render
    // target - the turntable is a single stage rendered to one texture,
    // so a second card costs a UI node and nothing else. Only one page
    // is ever visible, so they never both draw.
    for (page, caption) in [
        (IntroPage::SOLDIER, "your soldier, as equipped"),
        (IntroPage::ARMOURY, "your soldier, under the plate"),
    ] {
        commands.entity(root).with_children(|p| {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(2.5),
                    top: Val::Percent(22.0),
                    width: Val::Px(264.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(menu_ui::RULE_STAMP_PX)),
                    padding: UiRect::all(Val::Px(menu_ui::U3)),
                    row_gap: Val::Px(menu_ui::U2),
                    ..default()
                },
                BackgroundColor(menu_ui::shadow_a(menu_ui::PLATE_A)),
                BorderColor(branding::palette::BRONZE),
                BorderRadius::ZERO,
                ZIndex(menu_ui::ZL_PLATE),
                OnIntroPage(page),
                TurntableCard,
            ))
            .with_children(|card| {
                menu_ui::eyebrow(card, "TURNTABLE");
                card.spawn((
                    Node {
                        width: Val::Px(236.0),
                        height: Val::Px(236.0),
                        border: UiRect::all(Val::Px(menu_ui::RULE_HAIR_PX)),
                        ..default()
                    },
                    BorderColor(menu_ui::gold_a(menu_ui::FRAME_INNER_A)),
                    ImageNode {
                        image: fp.image.clone(),
                        ..default()
                    },
                ));
                card.spawn((
                    Text::new(caption.to_string()),
                    TextFont {
                        font_size: menu_ui::T_MICRO,
                        ..default()
                    },
                    TextColor(branding::palette::PARCHMENT_DIM),
                ));
            });
        });
    }
}

/// §7.2: stamp the whole turntable stage onto its layer once the spawn
/// flush has run - RenderLayers does not propagate to children in Bevy
/// 0.15, the same reason `tag_viewmodel_layer` below exists.
fn tag_forge_preview_layer(
    mut done: Local<bool>,
    mut commands: Commands,
    fp: Res<ForgePreview>,
    children: Query<&Children>,
) {
    if *done {
        return;
    }
    let mut stack = vec![fp.stand];
    let mut count = 0usize;
    while let Some(e) = stack.pop() {
        commands
            .entity(e)
            .insert(RenderLayers::layer(FORGE_PREVIEW_LAYER));
        count += 1;
        if let Ok(ch) = children.get(e) {
            stack.extend(ch.iter().copied());
        }
    }
    if count > 1 {
        *done = true;
    }
}

/// §7.2: the turntable TURNS. Intro only - the stage is invisible
/// everywhere else, so spinning it would be free but pointless.
fn forge_preview_spin(
    time: Res<Time>,
    fp: Res<ForgePreview>,
    mut q: Query<&mut Transform>,
) {
    if let Ok(mut tf) = q.get_mut(fp.stand) {
        tf.rotate_y(0.55 * time.delta_secs());
    }
}

/// §7.2: the mannequin WEARS the current picks. Hat and tunic recolour
/// their unique materials in place; the weapon rack shows the selected
/// primary and hides the rest.
fn forge_preview_sync(
    sel: Res<Selected>,
    fp: Res<ForgePreview>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut vis: Query<&mut Visibility>,
    mut tfs: Query<&mut Transform>,
) {
    if !sel.is_changed() {
        return;
    }
    let (_, (hr, hg, hb)) = HAT_CHOICES[sel.hat % HAT_CHOICES.len()];
    if let Some(mat) = materials.get_mut(&fp.hat_mat) {
        mat.base_color = Color::srgb(hr, hg, hb);
    }
    let (_, (tr, tg, tb)) = TUNIC_CHOICES[sel.tunic % TUNIC_CHOICES.len()];
    if let Some(mat) = materials.get_mut(&fp.tunic_mat) {
        mat.base_color = Color::srgb(tr, tg, tb);
        // emissive parity with the live rig's stripe (x0.4), so the
        // card's tunic glows exactly like the field one
        mat.emissive = LinearRgba::new(tr * 0.4, tg * 0.4, tb * 0.4, 1.0);
    }
    // §8.1: the picked helmet, and only that one
    for (i, e) in fp.helmets.iter().enumerate() {
        if let Ok(mut v) = vis.get_mut(*e) {
            *v = if i == sel.helmet % HELMET_CHOICES.len() {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    // the picked class's shape, and only that one
    for (i, e) in fp.class_rigs.iter().enumerate() {
        if let Ok(mut v) = vis.get_mut(*e) {
            *v = if sim::Class::ALL[i] == sel.class {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    let show = ALL_WEAPONS.iter().position(|w| *w == sel.loadout[0]);
    for (wi, e) in fp.weapons.iter().enumerate() {
        if let Ok(mut v) = vis.get_mut(*e) {
            *v = if Some(wi) == show {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
    // one-shot static carry pose: the weapon root takes the patrol
    // carry and both hands IK onto the selected gun's own grip sockets
    // - the same solver `sync_fighters` runs live, frozen mid-stride
    let kind = sel.loadout[0];
    let (wr_pos, wr_rot) = match kind {
        GunKind::Bow => (Vec3::new(-0.04, 0.48, 0.16), Quat::from_rotation_x(0.28)),
        GunKind::Spear => (Vec3::new(0.16, 0.72, 0.02), Quat::from_rotation_x(-1.35)),
        _ => (Vec3::new(WR_X, 0.50, WR_Z_HIP), Quat::from_rotation_x(0.31)),
    };
    if let Ok(mut t) = tfs.get_mut(fp.weapon_root) {
        t.translation = wr_pos;
        t.rotation = wr_rot;
    }
    let sockets = weapon_hand_specs(kind);
    let sh_l = Vec3::new(-SHOULDER_X, 0.62, 0.02);
    let sh_r = Vec3::new(SHOULDER_X, 0.62, 0.02);
    let (pole_l, pole_r) = (Vec3::new(-0.574, -0.80, 0.15), Vec3::new(0.574, -0.80, 0.15));
    let mut left = (Quat::from_rotation_x(0.08), 0.15_f32);
    let mut right = (Quat::from_rotation_x(-0.08), 0.15_f32);
    if let Some((p, ..)) = sockets.first() {
        right = solve_arm_ik(sh_r, wr_pos + wr_rot * *p, pole_r);
    }
    if let Some((p, ..)) = sockets.get(1) {
        left = solve_arm_ik(sh_l, wr_pos + wr_rot * *p, pole_l);
    }
    for (arm, (sh, elbow)) in [(fp.arm_l, left), (fp.arm_r, right)] {
        if let Ok(mut t) = tfs.get_mut(arm[0]) {
            t.rotation = sh;
        }
        if let Ok(mut t) = tfs.get_mut(arm[1]) {
            t.rotation = Quat::from_rotation_x(-elbow.max(0.0));
        }
    }
}

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
    /// §3.4 low-ready blend 0..1 and its spring velocity. Two fields, not
    /// one, because the ready-up overshoot needs a second-order state -
    /// position alone cannot remember which way it was travelling.
    lowready: f32,
    lowready_v: f32,
    inspect: bool,
    inspect_t: f32,
    /// §3.2: the spear windup fraction, EASED. `spear_wind_t` snaps from
    /// its last tick straight to 0 on release, and this value drives
    /// ~30 cm of translation and ~31 deg of rotation - reading it raw
    /// teleported the viewmodel in a single frame.
    spear_wind_ease: f32,
    /// §2.5 finger-settle spring state (k=220) - see `fp_viewmodel`.
    finger: f32,
    finger_v: f32,
}

#[allow(clippy::too_many_arguments)]
fn fp_viewmodel(
    time: Res<Time>,
    state: Res<State<GameState>>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    vm: Res<VmRig>,
    mut motion: EventReader<MouseMotion>,
    mut st: Local<VmState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<
        (&mut Transform, &mut Visibility),
        (Without<MainCam>, Without<TriggerFinger>, Without<ReticleDot>),
    >,
    mut trig: Query<(&mut Transform, &TriggerFinger), (Without<MainCam>, Without<ReticleDot>)>,
    // the draw, handed to `bow_string_sync` - see `BowDraw`
    mut bow_draws: Query<&mut BowDraw>,
    // Every &mut Transform query here must be provably disjoint from
    // every other or Bevy panics B0001 at startup - and "no entity would
    // ever have both" is not proof, it has to be in the FILTER. Hence
    // Without<TriggerFinger> here and Without<ReticleDot> on the other
    // two, rather than relying on the archetypes happening not to
    // overlap.
    mut dots: Query<
        (&mut Transform, &ReticleDot),
        (Without<MainCam>, Without<TriggerFinger>),
    >,
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
    // §2.5 FINGER SETTLE (k=220): the curve above is a hard target, so
    // driving the joint straight from it makes the finger TELEPORT
    // between rest and trigger. The stiffest spring in the table -
    // fingers settle fast, but they do settle, and that last few
    // milliseconds of overshoot-free travel is the whole difference
    // between a hand and a hinge.
    let finger_target = -0.12 + press * -0.38;
    {
        let (nx, nv) = damped_spring(
            Vec2::new(st.finger, 0.0),
            Vec2::new(st.finger_v, 0.0),
            Vec2::new(finger_target, 0.0),
            SPRING_K_FINGER_SETTLE,
            dt,
        );
        st.finger = nx.x;
        st.finger_v = nv.x;
    }
    for (mut t, tf_) in &mut trig {
        // the per-finger rest offset still rides the ADS blend; only the
        // moving part is sprung
        let rest = (tf_.rest + 0.12) * on_trigger;
        t.rotation = Quat::from_rotation_x(st.finger + rest);
    }
    let show = vm_rendered(state.get(), cam_ctl.person_t, p.alive(), p.roll_t);
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
    // §C: in a chassis the rifle is STOWED (same rule the body rig
    // applies) - the mount viewmodels below own the frame instead
    let slot = if p.in_mech() { None } else { weapon_slot(p.gun) };
    let scoped = cam_ctl.ads && spec.scoped && !p.in_mech();
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
        *v = if p.shield_up && !p.in_mech() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    // §C.7: the two hull-mount viewmodels - TURRET (gatling cluster,
    // also stands in for the bot-only autocannon) and the ROCKETS pod,
    // swapped by the selected mount exactly as the weapon strip maps it
    let pod_sel = p.in_mech() && p.mech_weapon == sim::MechWeapon::Rockets;
    if let Ok((_, mut v)) = q.get_mut(vm.mech_turret) {
        *v = if p.in_mech() && !pod_sel {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok((_, mut v)) = q.get_mut(vm.mech_pod) {
        *v = if pod_sel {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    // How drawn the bow is, from the same function the body rig uses so
    // the two views cannot disagree.
    let bow_pull = if p.gun == GunKind::Bow {
        bow_draw_visual(p.bow_draw_t, p.fire_cd, spec.fire_period, true)
    } else {
        0.0
    };
    // Hand it to `bow_string_sync` exactly as the body rig does. Same
    // number, same shape function, so first and third person cannot show
    // two different bows - which they did, because first person showed no
    // string movement and no arrow at all.
    if let Some(bow_slot) = weapon_slot(GunKind::Bow) {
        if let Ok(mut d) = bow_draws.get_mut(vm.weapons[bow_slot]) {
            d.pull = bow_pull;
            d.nocked = p.ammo > 0 && p.reload_t <= 0.0;
        }
    }
    let speed = (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt();
    // §2.2 suppression during ADS: ×(1 − 0.85-ads_t) - a trace of life
    // stays at full zoom, but a scoped gun does not swim
    // §owner: focused = STILL. Sway and breathe all but stop (0.85 ->
    // 0.97 suppression), and the recoil's visible rotation is damped
    // separately below - the bullet still goes where the sim says.
    let supp = 1.0 - cam_ctl.ads_t * 0.97;
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
    // §3.4 low-ready / obstruction. The ray runs from the EYE along the
    // camera's look direction: that is the line the barrel is drawn
    // along, so it is the line that can clip. Suppressed while scoped
    // (no viewmodel exists to rotate) and while unarmed (no barrel).
    {
        let eye = [
            p.pos[0],
            p.pos[1] + EYE_REL.min(p.height() - 0.12),
            p.pos[2],
        ];
        let (sy, cy) = cam_ctl.yaw.sin_cos();
        let pitch = cam_ctl.pitch;
        let fwd = [sy * pitch.cos(), -pitch.sin(), cy * pitch.cos()];
        let blocked = p.armed()
            && !vm_hidden_while_scoped(spec.scoped, cam_ctl.ads)
            && muzzle_blocked(&game.sim, eye, fwd);
        // Deref the Local ONCE - the borrow checker can split disjoint
        // fields of a plain `&mut VmState`, but not two Deref calls.
        let s = &mut *st;
        ready_up_step(
            &mut s.lowready,
            &mut s.lowready_v,
            if blocked { 1.0 } else { 0.0 },
            dt,
        );
    }
    let lr = st.lowready;
    // §C: a hull mount never plays the rifle's reload theatre
    let reloading = p.reload_t > 0.0 && !p.in_mech();
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
    // §C.7: a piloted mount kicks on ITS OWN cycle (shot_clock), never
    // the stowed rifle's fire_cd - same rule every other FX site obeys.
    let (cycle_cd, cycle_period) = if p.in_mech() {
        (
            shot_clock(p),
            match p.mech_weapon {
                sim::MechWeapon::Gatling => sim::GATLING_FIRE_PERIOD,
                sim::MechWeapon::Autocannon => sim::AUTOCANNON_CYCLE_S,
                sim::MechWeapon::Rockets => sim::POD_RELAUNCH_S,
            },
        )
    } else {
        (p.fire_cd, spec.fire_period)
    };
    // How far through this shot's kick-return window we are, 1 at the
    // instant of firing and decaying to 0. Shared by the hip-fire weapon
    // kick and the aimed reticle drift below - one shot, one curve.
    let kick_phase = if (p.armed() || p.in_mech()) && cycle_cd > 0.0 {
        ((VM_KICK_RETURN_S - (cycle_period - cycle_cd)) / VM_KICK_RETURN_S).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // §owner: AIMED, THE WEAPON DOES NOT MOVE. Not "moves less" - does
    // not move. Focus is the stance you take to make a precise shot, and
    // a sight picture that jumps is one you have to re-acquire between
    // every round, which is the opposite of what aiming is for.
    //
    // The recoil still has to be VISIBLE or the shot has no weight, so
    // it moves to the one place that costs no readability: the dot
    // floats inside the glass (`reticle_drift`), the housing does not.
    // Hip fire keeps the full weapon kick - that is where the gun
    // bucking is the point.
    let aim_e = ease_out(cam_ctl.ads_t);
    let kick_vm = kick_phase * (1.0 - aim_e);
    // The aimed half of that trade: the dot rides the same shot curve
    // the weapon no longer does. Up and slightly right, because that is
    // the direction the sim's own punch throws the bullet - the mark
    // moving WITH the shot is a readout, not decoration.
    {
        let drift = kick_phase * aim_e * RETICLE_DRIFT_M;
        for (mut t, dot) in &mut dots {
            t.translation = dot.rest + Vec3::new(drift * 0.35, drift, 0.0);
        }
    }
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
    // §3.6: this used to be a LOCAL guarantee that standard guns get
    // ZERO aim-shift, deliberately independent of `cam.ads` so that
    // changing the input gate could not silently move the viewmodel.
    // The input gate has now changed ON PURPOSE (see `input_and_step`:
    // right-click focuses every weapon), so the guarantee has to go with
    // it - otherwise focus would zoom the FOV and pull the third-person
    // boom in while the first-person gun stayed at the hip, which reads
    // as a bug rather than a stance.
    //
    // The guard's real intent is kept: the shift is still driven by
    // `ads_t` alone, so the pose can never drift out of step with the
    // zoom, and an unarmed player still gets nothing.
    let ads_e = if p.armed() { ease_out(cam_ctl.ads_t) } else { 0.0 };
    // §owner: focus ALIGNS THE SIGHTS. For a gun with an iron pair the
    // shift is derived per-gun: x cancels the carry's rightward offset
    // exactly, y lifts the weapon until its sight line (scaled by the
    // 0.9 model scale) sits on the eye line, z pulls it in. The old
    // one-size Vec3(-0.11, 0.052, ..) centred nothing precisely - every
    // gun landed near the middle and none ON it.
    let ads_shift = if p.in_mech() {
        // a hull mount has no iron pair - `sight_line_y` reads the
        // STOWED rifle; the mount gets a small generic pull-in
        Vec3::new(-0.03, 0.015, 0.05) * ads_e
    } else if let Some(sy) = sight_line_y(p.gun) {
        let (tr, _) = vm_carry(p.gun);
        Vec3::new(-tr.x, -(tr.y + sy * 0.9), 0.10) * ads_e
    } else {
        // no iron pair - still cancel THIS gun's own lateral carry. The
        // old hardcoded -0.11 was the default carry's x, so it doubled
        // the bow's -0.10 into -0.21 instead of zeroing it, and left the
        // spear at +0.04. A nominal sight height stands in for the
        // missing one.
        let (tr, _) = vm_carry(p.gun);
        Vec3::new(-tr.x, -(tr.y + 0.0866 * 0.9), 0.10) * ads_e
    };
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
        // §owner MELEE v2: the swing has to SHOW its line, or the
        // defender is being asked to read something invisible. The
        // wind-up cocks to the side the blade will travel from and the
        // strike whips across to the other - an overhead keeps the
        // original up-and-over. `side` is +1 for a right swing, -1 for a
        // left, 0 for an overhead.
        let side = match p.knife_dir {
            sim::MeleeDir::Left => -1.0_f32,
            sim::MeleeDir::Right => 1.0,
            sim::MeleeDir::Overhead => 0.0,
        };
        let lateral = side.abs();
        if ph < w {
            let e = ease_out((ph / w).clamp(0.0, 1.0)) * amp;
            (
                Vec3::new(
                    // a side swing cocks OUT to its own side; an
                    // overhead keeps the old inward wind
                    (0.10 - 0.34 * lateral) * e + 0.30 * side * e,
                    (0.09 + 0.05 * lateral) * e,
                    0.05 * e,
                ),
                Vec3::new(
                    (0.20 - 0.10 * lateral) * e,
                    (-0.45 + 0.30 * lateral) * e - 0.85 * side * e,
                    0.35 * e + 0.55 * side * e,
                ),
            )
        } else {
            let r = ((ph - w) / (total - w)).clamp(0.0, 1.0);
            // snap through, then settle home over the recovery
            let e = (1.0 - ease_out(r)) * amp;
            (
                Vec3::new(
                    (-0.14 + 0.06 * lateral) * e - 0.34 * side * e,
                    -0.06 * e,
                    -0.10 * e,
                ),
                Vec3::new(
                    (-0.30 + 0.14 * lateral) * e,
                    (0.55 - 0.30 * lateral) * e + 0.95 * side * e,
                    -0.50 * e - 0.65 * side * e,
                ),
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
        // viewmodel is not rendered at all; unscope restores next frame.
        // In a mech the glass is stowed - the mount stays on screen.
        *vmvis = if vm_hidden_while_scoped(spec.scoped && !p.in_mech(), cam_ctl.ads) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        // §2/§3 pose for the bow, the one main.rs marked "pending".
        // Drawing brings the bow up and in toward the aiming eye and
        // settles it closer to the face; an undrawn bow hangs low and
        // left, out of the sightline. This is the whole reason the ADS
        // ramp was ever pointed at projectile weapons - it was standing
        // in for a pose that did not exist.
        // The draw pose YIELDS to the grenade coil, because the two
        // cannot both be true of the same body: the hand that cooks a
        // grenade is the hand that would be on the string. The sim does
        // not gate them against each other - `step_bow_draw` and the
        // throw hold are separate keys - so both clocks really can run at
        // once, and until now the viewmodel added both shifts and rode up
        // 6.5 cm into the crosshair zone.
        let bow_t = VM_BOW_DRAW_SHIFT * bow_pull * (1.0 - gr);
        // §owner SUPPRESSION, the PLAYER's half.
        //
        // Rounds cracking past shake the weapon in your hands - and ONLY
        // the weapon. It is a viewmodel translation, so it moves the
        // picture and not the shot: the round still leaves the camera ray
        // exactly where the crosshair is. Bots pay for suppression in
        // accuracy (`SUPPRESS_AIM_PENALTY`); a human pays in composure,
        // which is the only currency a human should be charged in. Taking
        // a player's aim away is not tension, it is the game playing
        // itself.
        //
        // Two frequencies that do not share a period, so it reads as
        // rattle rather than as a wobble on a dial.
        let sup_shake = {
            let a = (p.suppress_t / sim::SUPPRESS_MAX_S).clamp(0.0, 1.0);
            let t = time.elapsed_secs();
            Vec3::new(
                (t * 31.0).sin() * VM_SUPPRESS_SHAKE.x,
                (t * 23.0).sin() * VM_SUPPRESS_SHAKE.y,
                0.0,
            ) * a
        };
        tf.translation = ads_shift
            + bow_t
            + sup_shake
            + VM_INSPECT_SHIFT * ie
            + rl_t
            + mel_t
            + VM_GRENADE_SHIFT * gr
            + carry_offset(s, st.theta, p.grounded, kick_vm, sp, dip, wind);
        // §3.4: low-ready is ROTATION ONLY - it appears in these three
        // Quats and nowhere in `tf.translation` above. Muzzle UP is
        // NEGATIVE pitch here (sprint's `+ sp * 0.61` is what lowers the
        // weapon), and inward yaw shares sprint's positive sign.
        tf.rotation = Quat::from_rotation_y(
            sway_rad.x + sp * 0.35 - wind * 0.25 + 0.85 * ie + drift + rl_e.y + mel_e.y
                + lr * LOWREADY_YAW,
        ) * Quat::from_rotation_x(
            kick_vm * 0.16 * VIEW_KICK_TRIM
                + breathe
                + sway_rad.y
                + st.pitch_lag * supp
                + rl_e.x
                + mel_e.x
                - 0.12 * gr
                + sp * 0.61
                - wind * 0.55
                + 0.22 * ie
                - lr * LOWREADY_PITCH,
        ) * Quat::from_rotation_z(kick_vm * 0.07 * VIEW_KICK_TRIM + rl_e.z + mel_e.z + 0.08 * gr);
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
            let p_team = game.sim.fighters[game.sim.player].team;
            let (r, g, b) = match cp.owner {
                Some(t) => branding::signal::side_of(t, p_team).accent_rgb(),
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
        // §4.3: the root needs its Node too - `minimap_scale` resizes it
        Query<(&mut Visibility, &mut Node), With<MinimapRoot>>,
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
    // Was a second, drifting copy of the "is the HUD up" question, and it
    // gave a different answer (it counted Paused as in-match, so the
    // minimap survived onto the pause menu). One predicate now.
    let in_match = hud_visible(state.get());
    let show = settings.minimap && in_match;
    for (mut v, _) in &mut qs.p0() {
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
    // §4.3: scale 0.25-1.0, default 0.7. The ROOT panel and the
    // projection extent are derived from the same number in the same
    // place - sizing the panel without rescaling `px` would leave every
    // marker plotted for the old size and spilling out of the frame.
    let scale = (settings.minimap_scale as f32 / 100.0).clamp(0.25, 1.0);
    let root_px = MINIMAP_PX * scale;
    let px = root_px - 10.0 * scale;
    for (_, mut node) in &mut qs.p0() {
        node.width = Val::Px(root_px);
        node.height = Val::Px(root_px);
    }
    // §9: the horizontal axis is MIRRORED, because in this game's yaw
    // convention screen-right is -X when facing +Z (camera_system derives
    // `screen_right = -right`, and damage_indicator honours the same
    // rule). Mapping +X to map-right drew every teammate, enemy and
    // objective on the wrong side: glance down, see an ally on your left,
    // turn left, and they are actually on your right.
    // §4.3: ROTATE WITH FACING (tunable). When on, the world is spun by
    // -yaw about the PLAYER, so "up" on the map is always the direction
    // the player is looking - the read becomes "that contact is on my
    // left" instead of "that contact is north-west, and I am facing...".
    //
    // The rotation is applied to world coordinates BEFORE the existing
    // mirror-and-project, deliberately: doing it after would have to
    // undo and redo the §9 axis mirror, and that mirror is exactly the
    // thing this file has already got wrong once.
    let me_pos = simr.fighters[simr.player].pos;
    let rotate = settings.minimap_rotate;
    let (rs, rc) = if rotate {
        let a = -simr.fighters[simr.player].yaw;
        (a.sin(), a.cos())
    } else {
        (0.0, 1.0)
    };
    let to_map = |x: f32, z: f32| {
        let (x, z) = if rotate {
            let (dx, dz) = (x - me_pos[0], z - me_pos[2]);
            (
                me_pos[0] + dx * rc - dz * rs,
                me_pos[2] + dx * rs + dz * rc,
            )
        } else {
            (x, z)
        };
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
            let (r, g, b) = branding::signal::Side::Enemy.accent_rgb();
            *bg = BackgroundColor(Color::srgba(r, g, b, slot.fade));
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
                    Some(t) => branding::signal::side_of(t, p_team).accent(),
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
        // §4.3: in ROTATE mode the world spins under a fixed needle, so
        // the needle itself must stop turning - otherwise the player's
        // facing gets applied twice and the arrow counter-rotates against
        // the map it is supposed to be leading.
        tfm.rotation = Quat::from_rotation_z(if rotate { 0.0 } else { me.yaw });
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

    // your own shot - the shot clock jumps up on the tick you fire (an
    // ammo delta misses the shot fired the same tick a reload completes).
    //
    // §C.7: this must read `shot_clock`, not `fire_cd`. A pilot's shots
    // run on the MOUNT's cycle, so every mech burst was silent to the
    // pilot firing it while everyone else heard it - the one FX site
    // still asking the stowed rifle whether anything had happened. The
    // `prev_gun` guard also has to relax in a chassis: the carried gun
    // never changes while piloting, but swapping MOUNTS must not be
    // mistaken for a weapon swap (which suppresses the shot that lands
    // on the same tick).
    let clock_now = shot_clock(p);
    let same_weapon = if p.in_mech() {
        p.mech_weapon == st.prev_mech_weapon
    } else {
        p.gun == st.prev_gun
    };
    if p.alive() && same_weapon && clock_now > st.prev_fire_cd {
        let snd = if p.in_mech() {
            match p.mech_weapon {
                // the hull gatling is the M249's big brother
                sim::MechWeapon::Gatling => &sfx.shot_mg,
                // the autocannon is a single heavy crack
                sim::MechWeapon::Autocannon => &sfx.shot_sniper,
                // a launch is a shotgun-class thump, not a rifle report
                sim::MechWeapon::Rockets => &sfx.shot_shotgun,
            }
        } else {
            shot_sound(&sfx, p.gun)
        };
        play(&mut commands, snd, 0.8);
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
    st.prev_fire_cd = shot_clock(p);
    st.prev_mech_weapon = p.mech_weapon;
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
            } else if p.shield_up && !p.in_mech() {
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
                simr.score[0] as u32, simr.score[1] as u32, simr.cfg.tdm_target
            ),
            Mode::Koth => format!(
                "HILL   BLUE {:>3.0}s - {:<3.0}s RED   (hold {:.0}s)",
                simr.score[0], simr.score[1], KOTH_TARGET_S
            ),
            Mode::Training => {
                "TRAINING RANGE   targets reset themselves - nothing shoots back"
                    .to_string()
            }
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

    // §4.5: the killfeed rows are their own system (`killfeed_rows`) -
    // they need Node/BorderColor access this text-only ParamSet has not
    // got, and cramming them in here is what kept the feed a flat string.

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
            // §owner: the SHOOTER's half of that exchange. The pod needs
            // POD_LOCK_S of unbroken sight before it will home, and the
            // victim has been warned since the instant it started - but
            // the pilot doing the locking got no readout at all, so the
            // only way to know whether a launch would track was to spend
            // a rocket and watch. Both ends of the mechanic are on
            // screen now.
            None if p.in_mech()
                && p.mech_weapon == sim::MechWeapon::Rockets
                && p.pod_lock_t > 0.0 =>
            {
                if p.pod_lock_t >= sim::POD_LOCK_S {
                    "LOCKED - release to launch".to_string()
                } else {
                    let pct = (p.pod_lock_t / sim::POD_LOCK_S * 100.0).min(99.0);
                    format!("LOCKING {pct:.0}%")
                }
            }
            None => String::new(),
        };
    }

    // status panel: loadout slots, weapon, ammo kind, HP/armor numbers
    if let Ok(mut t) = texts.p5().get_single_mut() {
        **t = if !p.alive() {
            // §4.7: the HUD names the phase the camera is in - same
            // `death_phase` the camera reads, so they cannot disagree.
            match death_phase(&game.sim, game.sim.player) {
                Some((k, true)) => format!(
                    "SPECTATING {}\nrespawn in {:.1}s",
                    game.sim.fighters[k].name,
                    p.respawn_t.max(0.0)
                ),
                _ => format!("DOWN - respawn in {:.1}s", p.respawn_t.max(0.0)),
            }
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
            // hull climbing: a pool that drains and DROPS you is not
            // allowed to be invisible. Ten segments, same vocabulary as
            // the pod tubes.
            let grip = if p.climbing.is_some() {
                let n = ((p.grip_pool / sim::CLIMB_GRIP_MAX) * 10.0).round() as i32;
                let bar: String = (0..10)
                    .map(|i| if i < n { '#' } else { '.' })
                    .collect();
                format!("
GRIP [{bar}] {:.0}%", p.grip_pool)
            } else {
                String::new()
            };
            format!(
                "+ {:.0}{regen}{}{}{grip}",
                p.health.max(0.0),
                if p.shield_up && !p.in_mech() { "  [SHIELD]" } else { "" },
                match p.armor_set {
                    ArmorSet::None => String::new(),
                    ArmorSet::RobotSuit => {
                        // §4.6: chassis VITALS only - the mounts own the
                        // bottom-right corner, the dismount bind lives on
                        // the equip hint. The old one-liner spanned the
                        // whole screen and folded into the ammo panel.
                        // (Also un-swaps the old args: POWER is p.armor,
                        // the 0..100 core - p.mech_rounds is turret belt.)
                        // §owner: the chassis has state the pilot could
                        // not see. BRACE changes how much recoil and how
                        // much rocket damage you eat, and the POWER
                        // STRIDE is on a heat budget that decides whether
                        // the Q burst is even available - both were
                        // simulated, neither was ever on screen, so the
                        // pilot was flying half blind.
                        let brace = if p.mech_brace { "  [BRACED]" } else { "" };
                        // §owner MECH SHIELD: the barrier's pool. Shown
                        // whenever it is raised OR still regrowing, so
                        // the pilot can see both what is left and when it
                        // is safe to push again.
                        let barrier = if p.shield_up || p.mech_shield_hp < sim::MECH_SHIELD_HP {
                            let n = ((p.mech_shield_hp / sim::MECH_SHIELD_HP) * 10.0)
                                .round()
                                .clamp(0.0, 10.0) as i32;
                            let bar: String =
                                (0..10).map(|i| if i < n { '#' } else { '.' }).collect();
                            let tag = if !p.shield_up {
                                "REGROW"
                            } else if p.mech_shield_hp > 0.0 {
                                "BARRIER"
                            } else {
                                "COLLAPSED"
                            };
                            format!("  {tag} [{bar}]")
                        } else {
                            String::new()
                        };
                        // ten segments of stride heat: full bar = ready,
                        // empty = still cooling. Shown only once it is
                        // actually spent, so a fresh chassis stays clean.
                        let stride = if p.stride_t > 0.0 {
                            "  STRIDE!".to_string()
                        } else if p.stride_heat > 1.0 {
                            let n = (((100.0 - p.stride_heat) / 100.0) * 10.0)
                                .round()
                                .clamp(0.0, 10.0) as i32;
                            let bar: String =
                                (0..10).map(|i| if i < n { '#' } else { '.' }).collect();
                            format!("  STRIDE [{bar}]")
                        } else {
                            String::new()
                        };
                        format!("
MECH  HULL {:.0}  PWR {:.0}{brace}{barrier}{stride}", p.hull, p.armor)
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
        } else if p.in_mech() {
            // §C.7: the corner shows the MOUNT, never the stowed rifle -
            // this branch must precede every infantry-flavored one or a
            // stale shield/reload/cook state paints over the belt.
            match p.mech_weapon {
                sim::MechWeapon::Rockets => {
                    let tubes: String = (0..POD_TUBES)
                        .map(|i| if i < p.pod_ammo { '#' } else { '.' })
                        .collect();
                    format!("POD {} / {}
[{tubes}]", p.pod_ammo, POD_TUBES)
                }
                _ => {
                    if p.gatling_vent_t > 0.0 {
                        format!("TURRET {}
VENTING {:.1}s", p.mech_rounds, p.gatling_vent_t)
                    } else {
                        format!("TURRET {}
HEAT {:.0}%", p.mech_rounds, p.gatling_heat)
                    }
                }
            }
        } else if p.spear_charge_t > 0.0 {
            // §owner JAVELIN: the wind, on screen. A charge you cannot
            // see is a charge you cannot time, and this one has a real
            // ceiling - the bar filling tells you when holding longer
            // has stopped buying anything.
            let frac = (p.spear_charge_t / sim::SPEAR_CHARGE_FULL_S).clamp(0.0, 1.0);
            let n = (frac * 10.0).round() as i32;
            let bar: String = (0..10).map(|i| if i < n { '#' } else { '.' }).collect();
            let tag = if frac >= 1.0 { "FULL" } else { "WIND" };
            format!("JAVELIN {tag}
[{bar}]")
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

    // (the crosshair's colour + geometry moved to `crosshair_render`,
    // §4.6 - it is drawn now, so it is a Node/BackgroundColor job)
}

// ---- §4.7 context progress bar -------------------------------------------
// "Context progress bar centred ~58% down, ALL-CAPS label, release
// cancels."
//
// GENERIC on purpose. Three different systems in this game already run a
// timed, interruptible channel - the extraction hold, mech entry, mech
// exit - and each had shipped its own ad-hoc readout or none at all. One
// bar, one resolver, so a fourth channel is a match arm rather than
// another bespoke widget.

/// §4.7: vertical position, as a fraction of screen height.
const CONTEXT_BAR_Y: f32 = 0.58;
const CONTEXT_BAR_W_PX: f32 = 260.0;
const CONTEXT_BAR_H_PX: f32 = 9.0;

#[derive(Component)]
struct ContextBarRoot;
#[derive(Component)]
struct ContextBarFill;
#[derive(Component)]
struct ContextBarLabel;

/// §4.7: what the player is currently channelling, if anything.
/// `(label, progress 0..1)`. The label is returned in whatever case the
/// caller wrote it; the renderer upper-cases it, so a future channel
/// cannot forget to.
fn context_channel(sim: &TdmSim) -> Option<(&'static str, f32)> {
    let p = &sim.fighters[sim.player];
    if !p.alive() {
        return None;
    }
    // Mech entry/exit outranks the extraction hold: it is the shorter,
    // more urgent window, and it is the one that has the player's hands.
    if p.mech_transition_t > 0.0 {
        let (label, total) = if p.mech_exiting {
            ("dismounting", MECH_EXIT_S)
        } else {
            ("boarding", MECH_ENTER_S)
        };
        // the timer COUNTS DOWN, so progress is what has elapsed
        return Some((label, 1.0 - (p.mech_transition_t / total).clamp(0.0, 1.0)));
    }
    if sim.mode == Mode::Extraction && sim.extract_hold > 0.0 {
        return Some((
            "extracting",
            (sim.extract_hold / EXTRACT_HOLD_S).clamp(0.0, 1.0),
        ));
    }
    None
}

/// §4.7: draw it. Hidden entirely when nothing is being channelled - a
/// progress bar sitting at zero in the middle of the screen is clutter
/// that teaches the player to stop looking there.
fn context_bar(
    game: Res<Game>,
    mut root: Query<&mut Visibility, With<ContextBarRoot>>,
    mut fill: Query<&mut Node, With<ContextBarFill>>,
    mut label: Query<&mut Text, With<ContextBarLabel>>,
) {
    let ch = context_channel(&game.sim);
    if let Ok(mut v) = root.get_single_mut() {
        *v = match ch {
            Some(_) => Visibility::Visible,
            None => Visibility::Hidden,
        };
    }
    let Some((text, t)) = ch else { return };
    if let Ok(mut n) = fill.get_single_mut() {
        n.width = Val::Percent(t.clamp(0.0, 1.0) * 100.0);
    }
    if let Ok(mut l) = label.get_single_mut() {
        // §4.7: ALL-CAPS label, enforced here so no caller can forget
        **l = text.to_uppercase();
    }
}

/// §4.5: the killfeed, drawn as rows.
///
/// "Newest at bottom, right-aligned: `Killer [+Assist] [modifiers]
/// Victim`. Names in team colour. Local-player rows get a 2px #B50000
/// border on rgba(0,0,0,0.5), radius 4px. Max 5 rows."
///
/// Names take the SIDE colour, not a fixed blue/orange: the brief wrote
/// its hexes back when teams were absolute, and a feed that calls the
/// player's own kills "orange" because they happened to draw Red would
/// undo the whole point of §3.6's relative palette.
fn killfeed_rows(
    game: Res<Game>,
    mut rows: Query<(&KillfeedRow, &mut Visibility, &mut BorderColor, &mut BackgroundColor)>,
    mut cells: Query<(&KillfeedCell, &mut Text, &mut TextColor)>,
) {
    let simr = &game.sim;
    let p_team = simr.fighters[simr.player].team;
    // newest at the BOTTOM: take the last N, keep chronological order
    let feed: Vec<_> = {
        let n = simr.kill_feed.len();
        simr.kill_feed[n.saturating_sub(KILLFEED_ROWS)..].iter().collect()
    };

    for (row, mut vis, mut border, mut bg) in &mut rows {
        match feed.get(row.0) {
            Some((ev, _)) => {
                *vis = Visibility::Inherited;
                let mine = ev.killer == simr.player || ev.victim == simr.player;
                *border = BorderColor(if mine { KILLFEED_MINE_BORDER } else { Color::NONE });
                *bg = BackgroundColor(if mine {
                    KILLFEED_MINE_BG
                } else {
                    Color::srgba(0.0, 0.0, 0.0, 0.28)
                });
            }
            None => {
                *vis = Visibility::Hidden;
                *border = BorderColor(Color::NONE);
                *bg = BackgroundColor(Color::NONE);
            }
        }
    }

    let side_color = |i: usize| {
        branding::signal::side_of(simr.fighters[i].team, p_team).accent()
    };
    for (cell, mut text, mut color) in &mut cells {
        let Some((ev, _)) = feed.get(cell.0) else {
            **text = String::new();
            continue;
        };
        match cell.1 {
            0 => {
                let assist = match ev.assist {
                    Some(a) => format!(" +{}", simr.fighters[a].name),
                    None => String::new(),
                };
                **text = format!("{}{assist}", simr.fighters[ev.killer].name);
                *color = TextColor(side_color(ev.killer));
            }
            1 => {
                // glyphs sit between the two names, in neutral parchment
                // so they never compete with the side colours either side
                **text = format!("{}>", feed_glyphs(ev.headshot, ev.noscope, ev.blind, ev.smoke, ev.wallbang));
                *color = TextColor(branding::palette::PARCHMENT_DIM);
            }
            _ => {
                **text = simr.fighters[ev.victim].name.to_string();
                *color = TextColor(side_color(ev.victim));
            }
        }
    }
}

/// §4.1: drive the segmented health bar and the armour pip cluster.
///
/// The segments share `vitals_color` with the NUMBER above them, so the
/// bar going red and the number going red can never disagree - they are
/// one decision read twice, not two thresholds maintained separately.
///
/// A partly-drained segment dims rather than shrinking. Shrinking the
/// last block would reintroduce exactly the continuous-ratio read the
/// segmentation exists to replace.
fn vitals_bars(
    game: Res<Game>,
    settings: Res<GameSettings>,
    mut q: ParamSet<(
        Query<(&VitalsSeg, &mut BackgroundColor)>,
        Query<(&ArmorPip, &mut BackgroundColor, &mut BorderColor)>,
        Query<&mut Visibility, With<VitalsBarRow>>,
    )>,
) {
    // §4.1 `hud_vitals_style`: 1 = numbers only. Hide the rows and stop -
    // no point computing fills nothing will show.
    if settings.hud_vitals_style == 1 {
        for mut v in &mut q.p2() {
            *v = Visibility::Hidden;
        }
        return;
    }
    for mut v in &mut q.p2() {
        *v = Visibility::Inherited;
    }
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];

    let hp = p.health.max(0.0);
    let lit = vitals_color(hp, simr.t);
    // How many whole segments are alive, and how far into the next one.
    let filled = hp / (MAX_HEALTH / VITALS_SEGMENTS as f32);
    for (seg, mut bg) in &mut q.p0() {
        let i = seg.0 as f32;
        let c = if filled >= i + 1.0 {
            lit
        } else if filled > i {
            // the partial block: same hue, dimmed, so it still reads as
            // "this one is going" rather than as empty
            let s = lit.to_srgba();
            Color::srgba(s.red, s.green, s.blue, 0.45)
        } else {
            Color::srgba(0.35, 0.32, 0.28, 0.55) // spent
        };
        *bg = BackgroundColor(c);
    }

    // Armour pips.
    //
    // This game has no infantry armour POOL - `Fighter::armor` is the
    // Robot Suit's power core and is flat zero for every other set, so a
    // pip cluster driven straight off it would sit empty for the entire
    // match on four of the five sets. Infantry protection is instead the
    // SET's flat damage reduction, which is a real, readable quantity
    // and the one a player actually wants beside their health.
    //
    // So the cluster answers "how protected am I" from whichever model
    // applies: the live power core in a chassis, the equipped set's
    // torso plate on foot. Scaled against Folk's 45, the heaviest set,
    // so a full cluster means "the best there is" rather than an
    // arbitrary ceiling.
    let in_mech = p.armor_set == ArmorSet::RobotSuit && p.hull > 0.0;
    let pips = if in_mech {
        p.armor.max(0.0) / (POWER_MAX / ARMOR_PIPS as f32)
    } else {
        let torso = armor_spec(p.armor_set).flat_torso;
        torso / (ARMOR_PIP_REFERENCE / ARMOR_PIPS as f32)
    };
    for (pip, mut bg, mut border) in &mut q.p1() {
        let full = pips >= pip.0 as f32 + 1.0;
        let partial = !full && pips > pip.0 as f32;
        *bg = BackgroundColor(if full {
            branding::palette::GOLD
        } else if partial {
            let s = branding::palette::GOLD.to_srgba();
            Color::srgba(s.red, s.green, s.blue, 0.40)
        } else {
            Color::NONE
        });
        // an empty pip keeps its outline: the cluster's SIZE is the tell
        // for how much armour the set can hold, and that must not shrink
        // as it drains
        *border = BorderColor(if full || partial {
            branding::palette::GOLD
        } else {
            branding::palette::BRONZE
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
        // §C.7: while piloting, "low" means the MOUNT's own pool, not
        // the stowed rifle's magazine
        let (a, m) = if p.in_mech() {
            match p.mech_weapon {
                sim::MechWeapon::Rockets => (p.pod_ammo as u32, POD_TUBES as u32),
                _ => (p.mech_rounds, MECH_ROUNDS),
            }
        } else {
            (p.ammo, gun(p.gun).mag)
        };
        *c = TextColor(if ammo_is_low(a, m) {
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

/// §4.6 (Brief VIII): draw the crosshair.
///
/// Owns ALL of it - geometry, colour, the scoped-hide rule and the
/// kill-confirm pop - because the previous split (colour in `hud_system`,
/// glyph in `crosshair_kill_pop`) existed only to dodge a `&mut Text`
/// alias, and there is no `Text` here any more.
///
/// The pop is unchanged in meaning: after your kill the cross becomes an
/// X for the same half second. It is now a 45° rotation of the drawn
/// geometry plus a small outward kick, instead of swapping one glyph for
/// another, so it inherits the player's own size/thickness/colour.
fn crosshair_render(
    game: Res<Game>,
    cam: Res<CamCtl>,
    settings: Res<GameSettings>,
    mut root: Query<&mut Transform, With<CrosshairRoot>>,
    mut pieces: Query<(&CrosshairPiece, &mut Node, &mut BackgroundColor)>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];

    let fresh_hit_head = simr
        .hits
        .iter()
        .rev()
        .find(|(ev, ttl)| ev.shooter == simr.player && *ttl > 2.0)
        .map(|(ev, _)| ev.zone == HitZone::Head);
    let fresh_kill = simr
        .kill_feed
        .iter()
        .rev()
        .any(|(ev, ttl)| ev.killer == simr.player && *ttl > 4.5);
    // §5.2 (Brief VI): scoped-class weapons draw NO crosshair while
    // unscoped - the no-scope prayer is the tradeoff. A mech fires hull
    // mounts, never the stowed glass - the pilot keeps a crosshair.
    let noscope_hidden = gun(p.gun).scoped && !cam.ads && !p.in_mech();
    let fb = crosshair_feedback(
        noscope_hidden,
        fresh_kill,
        fresh_hit_head,
        cam.blocked && p.alive(),
    );
    let fill = crosshair_color(fb, crosshair_rgb(&settings), settings.cross_alpha);
    let hidden = fb == CrossFeedback::Hidden;

    // the same live cone the stability bracket reads, so a DYNAMIC
    // crosshair and the bracket bloom off one number
    let spread = simr.aim_spread_of(simr.player, cam.ads_t > 0.9);
    let gap = crosshair_gap_px(settings.cross_gap, spread, settings.cross_dynamic)
        + if fresh_kill { CROSS_KILL_POP_PX } else { 0.0 };
    let arms = crosshair_arm_rects(
        settings.cross_size as f32,
        gap,
        settings.cross_thickness as f32,
    );
    let dot = crosshair_dot_rect(settings.cross_thickness as f32);
    let outline_px = if settings.cross_outline {
        settings.cross_outline_px as f32
    } else {
        0.0
    };

    if let Ok(mut tf) = root.get_single_mut() {
        tf.rotation = Quat::from_rotation_z(if fresh_kill { PI * 0.25 } else { 0.0 });
    }

    for (piece, mut node, mut bg) in &mut pieces {
        let idx = piece.idx as usize;
        let shown = !hidden
            && (idx == CROSS_PIECES - 1 || crosshair_arm_shown(idx, settings.cross_t_shape))
            && (idx != CROSS_PIECES - 1 || settings.cross_dot)
            && !(piece.outline && !settings.cross_outline);
        if !shown {
            *bg = BackgroundColor(Color::NONE);
            node.width = Val::Px(0.0);
            node.height = Val::Px(0.0);
            continue;
        }
        let base = if idx == CROSS_PIECES - 1 { dot } else { arms[idx] };
        let r = if piece.outline {
            crosshair_outline_rect(base, outline_px)
        } else {
            base
        };
        node.left = Val::Px(r.left);
        node.top = Val::Px(r.top);
        node.width = Val::Px(r.w);
        node.height = Val::Px(r.h);
        *bg = BackgroundColor(if piece.outline {
            // a dark backing, never a second colour to tune - its job is
            // contrast against a bright wall, nothing else
            Color::srgba(0.0, 0.0, 0.0, 0.75 * (settings.cross_alpha as f32 / 255.0))
        } else {
            fill
        });
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
    // §4.6: the SAME px-per-radian the dynamic crosshair uses, so the
    // bracket and the arms cannot bloom at two different rates.
    let px = 12.0 + spread * CROSS_SPREAD_PX_PER_RAD;
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
    mut last: Local<(u32, u32, i32, u8, u32, u8, i32, i32)>,
    mut idle_t: Local<f32>,
    mut q: Query<
        &mut TextColor,
        Or<(With<PanelInfoText>, With<PanelAmmoText>, With<HudText>)>,
    >,
) {
    let p = &game.sim.fighters[game.sim.player];
    // the mech trio (belt, tubes, hull) joins the snapshot - none of the
    // infantry fields move in a chassis, so the HUD used to fade to 45%
    // mid-firefight while piloting
    let snap = (
        p.ammo,
        p.reserve,
        p.health as i32,
        p.throw_sel + if p.shield_up { 100 } else { 0 },
        p.mech_rounds,
        p.pod_ammo,
        p.hull as i32,
        p.grip_pool as i32,
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
            // §4.8 columns: K/A/D/DMG per spec. Ping is deliberately
            // absent - this build has zero netcode, and a fabricated
            // "0ms" column would be decoration pretending to be data.
            // Per-player Score is likewise skipped: only team score
            // exists, and K already IS the per-player scoring stat.
            s += &format!(
                "{label}  -  {} pts\n  {:<12}{:>4}{:>4}{:>4}{:>7}   {}\n",
                match game.sim.mode {
                    Mode::Tdm => format!("{:.0}", game.sim.score[TdmSim::team_idx(team)]),
                    Mode::Koth => format!("{:.0}s", game.sim.score[TdmSim::team_idx(team)]),
                    Mode::Extraction => format!("{} horde", game.sim.zombies.len()),
                    // no team score on the range - kills are the readout
                    Mode::Training => "range".to_string(),
                },
                "NAME",
                "K",
                "A",
                "D",
                "DMG",
                "WEAPON"
            );
            let mut rows: Vec<(usize, &Fighter)> = game
                .sim
                .fighters
                .iter()
                .enumerate()
                .filter(|(_, f)| f.team == team)
                .collect();
            rows.sort_by(|(_, a), (_, b)| {
                b.kills.cmp(&a.kills).then(a.deaths.cmp(&b.deaths))
            });
            let me = game.sim.player;
            for (idx, f) in rows {
                // §4.8 "local row highlighted": a text scoreboard cannot
                // draw the spec's bordered row, so the local player gets
                // the same `> ` marker the weapon strip already uses -
                // one convention for "this is you" across the whole HUD.
                s += &format!(
                    "{} {:<12}{:>4}{:>4}{:>4}{:>7.0}   {}\n",
                    if idx == me { ">" } else { " " },
                    f.name,
                    f.kills,
                    f.assists,
                    f.deaths,
                    f.dmg_dealt,
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

/// Which screen edge a world point sits behind, given the camera yaw.
///
/// 0 front / 1 right / 2 behind / 3 left. Extracted because suppression
/// now asks the same question the damage flash does, and two copies of a
/// screen-convention this fiddly would drift: screen-right is
/// (−cos yaw, sin yaw) in this game's convention (the playtest-verified
/// A/D mapping), so a shooter at rel −π/2 sits screen-RIGHT - the right
/// strip, not the left.
fn edge_toward(from: [f32; 3], at: [f32; 3], yaw: f32) -> Option<usize> {
    let dx = from[0] - at[0];
    let dz = from[2] - at[2];
    if dx * dx + dz * dz < 1e-4 {
        return None;
    }
    let rel = wrap_angle(dx.atan2(dz) - yaw);
    let (c, s) = (rel.cos(), rel.sin());
    Some(if c > 0.5 {
        0
    } else if c < -0.5 {
        2
    } else if s < 0.0 {
        1
    } else {
        3
    })
}

/// Damage RED tops out at 0.55 alpha; suppression's pale gold at 0.25.
///
/// The GAP is what makes "hit" and "shot at" two different messages
/// rather than two strengths of one, and it is what lets the two share a
/// widget at all: a hit wins the strip on alpha alone, with no priority
/// rule bolted on to arbitrate. Kept at or under half the damage ceiling
/// so the separation is unmistakable rather than merely present -
/// `being_hit_outshouts_being_shot_at` is what set this number, after the
/// first guess (0.28) failed its own rule by a hundredth.
const SUPPRESS_EDGE_ALPHA: f32 = 0.25;

/// The screen edge facing the shooter flashes - RED when you are hit,
/// pale gold when rounds merely crack past you.
///
/// §owner SUPPRESSION's player-facing half. The mechanic shipped with a
/// viewmodel shake and nothing else, which told you that something was
/// happening but not what or where - and a shake is easy to read as
/// recoil, which is the one thing it is not.
///
/// It reuses the damage flash's own strips deliberately, rather than
/// getting a HUD element of its own. The player already knows how to read
/// "that edge means over there"; suppression is the SAME information one
/// step earlier - rounds from that bearing, and they have not hit you
/// yet. A second, differently-shaped directional widget would be asking
/// them to learn the same idea twice.
///
/// That is also the whole answer to "does it need a directional tell":
/// yes, because "you are being shot at" only tells a player to move,
/// while "from THERE" tells them where to move to.
fn damage_indicator(
    game: Res<Game>,
    cam: Res<CamCtl>,
    mut edges: Query<(&DmgEdge, &mut BackgroundColor)>,
) {
    let pi = game.sim.player;
    let p = &game.sim.fighters[pi];
    let ppos = p.pos;
    let mut inten = [0.0_f32; 4]; // top(front) right bottom(behind) left
    for (ev, ttl) in &game.sim.hits {
        if ev.victim != pi {
            continue;
        }
        let w = (ttl / 2.2).clamp(0.0, 1.0);
        if let Some(idx) = edge_toward(ev.from, ppos, cam.yaw) {
            inten[idx] = inten[idx].max(w);
        }
    }
    // suppression, on the same strips at a quieter weight
    let mut supp = [0.0_f32; 4];
    if p.suppress_t > 0.0 {
        let w = (p.suppress_t / sim::SUPPRESS_MAX_S).clamp(0.0, 1.0);
        if let Some(idx) = edge_toward(p.suppress_from, ppos, cam.yaw) {
            supp[idx] = w;
        }
    }
    for (e, mut bg) in &mut edges {
        let i = e.0 as usize;
        let a_dmg = inten[i] * 0.55;
        let a_sup = supp[i] * SUPPRESS_EDGE_ALPHA;
        // A hit is the louder message and always wins the strip - not by
        // a priority rule bolted on top, but because its alpha ceiling is
        // twice suppression's, so the comparison decides it by itself.
        let (r, g, b) = if a_dmg >= a_sup {
            (0.85, 0.08, 0.08)
        } else {
            (0.95, 0.86, 0.55)
        };
        *bg = BackgroundColor(Color::srgba(r, g, b, a_dmg.max(a_sup)));
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
fn open_controls(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    brand: Option<Res<branding::BrandAssets>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let aspect = windows
        .get_single()
        .map(|w| w.resolution.width() / w.resolution.height().max(1.0))
        .unwrap_or(menu_ui::KEY_ART_ASPECT);
    // Controls used to take only Commands, which was safe purely because
    // it is reachable only from Paused (whose opener had already released
    // the cursor). Any future entry path would have soft-locked it with a
    // grabbed, invisible cursor - so it releases for itself now.
    release_cursor(&mut cam, &mut windows);
    cam.ads = false;

    let brand = brand.as_deref();
    let root = menu_ui::spawn_surface(&mut commands, brand, aspect);
    commands.entity(root).insert(ControlsRoot).with_children(|p| {
        menu_ui::plate(p, menu_ui::PLATE_W_CONTROLS, |b| {
            menu_ui::title(b, "CONTROLS");
            menu_ui::rule_and_boss(b, true);
            // Four group columns, wrapping. The old screen was the whole
            // 27-row registry as ONE format!-padded Text with no width
            // bound: the longest row rendered ~1300px wide inside the
            // game's own 1280px window, overrunning both edges.
            // THREE columns, sized by flex rather than a fixed width.
            // The first cut of this screen wrapped four fixed-460px
            // group columns 2x2 - and the second row fell off the plate,
            // exactly the overflow the settings screen had already
            // taught. Column 1 stacks the two short groups; the capture
            // gate caught it, not the code review.
            const COLS: [&[BindGroup]; 3] = [
                &[BindGroup::Move, BindGroup::Gear],
                &[BindGroup::Fight],
                &[BindGroup::View],
            ];
            b.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(menu_ui::U6),
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|row| {
                for groups in COLS {
                    row.spawn(Node {
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(menu_ui::U),
                        ..default()
                    })
                    .with_children(|col| {
                        for g in groups {
                            menu_ui::eyebrow(col, g.title());
                            for bind in BIND_REGISTRY.iter().filter(|x| x.group == *g) {
                                menu_ui::bind_row(col, bind.key, bind.action, bind.essential);
                            }
                        }
                    });
                }
            });
            menu_ui::seal_footer(b, brand, Some(("ESC", "BACK")));
        });
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
                BackgroundColor(menu_ui::shadow_a(0.88)),
                GlobalZIndex(25),
                FirstRunRoot,
                HudRoot,
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("GOOD TO KNOW"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(branding::palette::GOLD),
                ));
                // the SAME gold-ruled keycaps the Controls screen uses - one
                // visual language for a key, on both surfaces
                for b in BIND_REGISTRY.iter().filter(|b| b.essential) {
                    menu_ui::bind_row(p, b.key, b.action, true);
                }
                p.spawn((
                    Text::new(
                        "ARMOR SETS lie on glowing pads - walk over one.\nFull list: ESC > Controls.  (any key to dismiss)",
                    ),
                    TextFont {
                        font_size: menu_ui::T_MICRO,
                        ..default()
                    },
                    TextColor(branding::palette::PARCHMENT_DIM),
                    Node {
                        margin: UiRect::top(Val::Px(menu_ui::U3)),
                        ..default()
                    },
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
    // hull climbing: the grab is invisible without this - a stripped
    // zone looks like any other piece of a mech. Uses the sim's own
    // `climb_target`, so the prompt cannot offer a grab the step would
    // refuse.
    if p.climbing.is_some() {
        **t = "U - LET GO".to_string();
        return;
    }
    if !p.in_mech() && p.alive() && game.sim.climb_target(game.sim.player).is_some() {
        // distinguish "in reach" from "in reach but too spent to hold" -
        // the same 5.0 floor the attach verb enforces
        **t = if p.grip_pool > 5.0 {
            "U - GRAB THE HULL".to_string()
        } else {
            "GRIP SPENT - let it recover".to_string()
        };
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
    pick_row_on(p, label, items, w, IntroPage::SOLDIER)
}

/// `pick_row`, tagged with the page it belongs to. The row is spawned
/// exactly as before - the tag only decides when it is VISIBLE, so
/// paging cannot change what any row does or how it is wired.
fn pick_row_on<C: Component + Copy>(
    p: &mut ChildBuilder,
    label: &str,
    items: &[(&str, C)],
    w: f32,
    page: u8,
) {
    p.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        },
        OnIntroPage(page),
    ))
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                // category labels: were cool periwinkle against a warm
                // gold title. Bronze puts them in the same metal family
                // as the selection they label.
                TextColor(branding::palette::BRONZE),
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
    brand: Option<Res<branding::BrandAssets>>,
    mut page: ResMut<IntroPage>,
) {
    let aspect = windows
        .get_single()
        .map(|w| w.resolution.width() / w.resolution.height().max(1.0))
        .unwrap_or(menu_ui::KEY_ART_ASPECT);
    release_cursor(&mut cam, &mut windows);
    cam.ads = false;
    // Always open on the title. Touching the resource also marks it
    // CHANGED, which is what makes intro_paging run its first pass -
    // without it every page would be visible at once on frame 1.
    page.0 = IntroPage::TITLE;

    let brand = brand.as_deref();
    let root = menu_ui::spawn_surface(&mut commands, brand, aspect);
    commands.entity(root).insert(IntroRoot).with_children(|p| {
        // THE WORDMARK IS THE TITLE. The old screen drew the key art at
        // GlobalZIndex(-10) and then painted a 94%-opaque navy wash over
        // it at implicit z 0 - at most 6% of the art could ever reach the
        // eye, which is the real reason it never appeared. It also set a
        // 40px gold text heading reading "JOHN KINGDOM - ARENA" directly
        // beneath the wordmark PNG that says the same words.
        if let Some(b) = brand {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(24.0),
                    // width only - the image's own measure derives the
                    // height from its aspect. Setting both stretches.
                    width: Val::Percent(52.0),
                    top: Val::Percent(7.0),
                    ..default()
                },
                ImageNode {
                    image: b.wordmark.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.96),
                    ..default()
                },
                ZIndex(menu_ui::ZL_MARK),
                OnIntroPage(IntroPage::TITLE),
            ));
        }

        menu_ui::plate(p, menu_ui::PLATE_W_INTRO, |b| {
            // heading + subtitle are PAGE-DRIVEN: the title page is named
            // by the wordmark above, the other two by the question they
            // answer. intro_paging rewrites both on every page change.
            b.spawn((
                Text::new(""),
                TextFont { font_size: menu_ui::T_HEAD, ..default() },
                TextColor(branding::palette::GOLD),
                Node { display: Display::None, ..default() },
                IntroHeading,
            ));
            menu_ui::rule_and_boss(b, true);
            b.spawn((
                Text::new(IntroPage(IntroPage::TITLE).subtitle()),
                TextFont { font_size: menu_ui::T_MICRO, ..default() },
                TextColor(branding::palette::PARCHMENT_DIM),
                IntroSubtitle,
            ));

            // ---- SOLDIER page ------------------------------------------
            let sold = OnIntroPage(IntroPage::SOLDIER);
            // §owner: CLASS leads the page - it is the choice that
            // frames every one below it, and the only one that changes
            // how the soldier plays rather than what he carries.
            let classes: Vec<(&str, ClassButton)> = sim::Class::ALL
                .iter()
                .map(|c| (sim::class_spec(*c).name, ClassButton(*c)))
                .collect();
            menu_ui::pill_row(b, "CLASS", &classes, sold);
            let prim: Vec<(&str, LoadoutButton)> = PRIMARIES
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(0, *g)))
                .collect();
            menu_ui::pill_row(b, "PRIMARY", &prim, sold);
            let sec: Vec<(&str, LoadoutButton)> = SECONDARIES
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(1, *g)))
                .collect();
            menu_ui::pill_row(b, "SECONDARY", &sec, sold);
            let spc: Vec<(&str, LoadoutButton)> = SPECIALS
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(2, *g)))
                .collect();
            menu_ui::pill_row(b, "SPECIAL", &spc, sold);
            // §6 (Brief IV): the melee slot is a CHOICE now
            let melee: Vec<(&str, MeleeButton)> = vec![
                ("Combat Knife", MeleeButton(false)),
                ("War Axe", MeleeButton(true)),
            ];
            menu_ui::pill_row(b, "MELEE", &melee, sold);
            // §8 (Brief IV): 6-point grenade budget presets
            let nades: Vec<(&str, NadeButton)> = GRENADE_PRESETS
                .iter()
                .enumerate()
                .map(|(i, (_, n))| (*n, NadeButton(i)))
                .collect();
            menu_ui::pill_row(b, "GRENADES", &nades, sold);
            // §8.1: SHAPE first, then the tint that paints it - the row
            // order is the order the two choices actually compose in.
            let helmets: Vec<(&str, CosmeticButton)> = HELMET_CHOICES
                .iter()
                .enumerate()
                .map(|(i, (n, _))| (*n, CosmeticButton(CosmeticSlot::Helmet, i)))
                .collect();
            menu_ui::pill_row(b, "HELMET", &helmets, sold);
            let hats: Vec<(&str, CosmeticButton)> = HAT_CHOICES
                .iter()
                .enumerate()
                .map(|(i, (n, _))| (*n, CosmeticButton(CosmeticSlot::HatTint, i)))
                .collect();
            menu_ui::pill_row(b, "HELMET TINT", &hats, sold);
            let tunics: Vec<(&str, CosmeticButton)> = TUNIC_CHOICES
                .iter()
                .enumerate()
                .map(|(i, (n, _))| (*n, CosmeticButton(CosmeticSlot::Tunic, i)))
                .collect();
            menu_ui::pill_row(b, "OUTFIT", &tunics, sold);
            // §7.2: the Forge, on screen at last
            let saves: Vec<(&str, ForgeUiButton)> = vec![
                ("SAVE 1", ForgeUiButton::Save(1)),
                ("SAVE 2", ForgeUiButton::Save(2)),
                ("SAVE 3", ForgeUiButton::Save(3)),
                ("RANDOMIZE", ForgeUiButton::Randomize),
            ];
            menu_ui::pill_row(b, "FORGE", &saves, sold);
            let loads: Vec<(&str, ForgeUiButton)> = vec![
                ("LOAD 1", ForgeUiButton::Load(1)),
                ("LOAD 2", ForgeUiButton::Load(2)),
                ("LOAD 3", ForgeUiButton::Load(3)),
            ];
            menu_ui::pill_row(b, "", &loads, sold);
            // the spec readout, in its own engraved sub-plate
            b.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(menu_ui::RULE_HAIR_PX)),
                    padding: UiRect::all(Val::Px(menu_ui::U3)),
                    margin: UiRect::top(Val::Px(menu_ui::U3)),
                    ..default()
                },
                BorderColor(menu_ui::bronze_a(0.40)),
                BackgroundColor(Color::NONE),
                BorderRadius::ZERO,
                sold,
            ))
            .with_children(|sp| {
                sp.spawn((
                    Text::new(""),
                    TextFont { font_size: menu_ui::T_DATA, ..default() },
                    TextColor(branding::palette::PARCHMENT),
                    TechReadout,
                ));
            });

            // ---- ARMOURY page ------------------------------------------
            // §C tier 2: four rows, one per body region, in the order a
            // harness goes on. Grouped by REGION rather than listed as 24
            // flat pills, because the decision a player is actually
            // making is regional - "do I want my arms covered" - and
            // because one row of 24 would be illegible at the pill widths
            // every other row on these pages uses.
            let arm = OnIntroPage(IntroPage::ARMOURY);
            // Every armoury row lives in a column that stops short of the
            // plate's right edge, because this is the one page with a
            // turntable card overlapping it. The other pages' rows run
            // full width and pass under the card harmlessly - their
            // longest row ends before it - but six pills of plate names
            // would run straight into the soldier.
            b.spawn((
                Node {
                    width: Val::Percent(ARMOURY_ROW_W_PCT),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                arm,
            ))
            .with_children(|col| {
                // The five standard harnesses LEAD the page. Anyone who
                // does not want to think about plate should be able to
                // answer the whole question in one click and page on; the
                // grid below is for anyone who does.
                let presets: Vec<(&str, ArmorPresetButton)> = sim::ArmorPreset::ALL
                    .iter()
                    .map(|p| (p.name(), ArmorPresetButton(*p)))
                    .collect();
                menu_ui::pill_row(col, "PRESET", &presets, arm);
                for (label, pieces) in ARMOUR_ROWS {
                    let items: Vec<(&str, ArmorButton)> = pieces
                        .iter()
                        .map(|p| (p.short_name(), ArmorButton(*p)))
                        .collect();
                    menu_ui::pill_row(col, label, &items, arm);
                }
                // The number that makes the grid a DECISION rather than a
                // row of switches. Without a live weight read against the
                // ceiling every plate looks free, and the only sensible
                // play is to wear all of it.
                col.spawn((
                    Text::new(String::new()),
                    TextFont { font_size: menu_ui::T_DATA, ..default() },
                    TextColor(branding::palette::GOLD),
                    Node { margin: UiRect::top(Val::Px(menu_ui::U2)), ..default() },
                    ArmorWeightText,
                    arm,
                ));
                col.spawn((
                    Text::new(
                        "a bare segment takes 25% more where the plate is missing.\n\
                         over the ceiling, every kilo costs 0.15 m/s."
                            .to_string(),
                    ),
                    TextFont { font_size: menu_ui::T_MICRO, ..default() },
                    TextColor(branding::palette::PARCHMENT_DIM),
                    Node { margin: UiRect::top(Val::Px(menu_ui::U2)), ..default() },
                    arm,
                ));
            });

            // ---- MATCH page --------------------------------------------
            let mtch = OnIntroPage(IntroPage::MATCH);
            let maps: Vec<(&str, MapButton)> = MapKind::ALL
                .iter()
                // §12: Battlefield left the PvP rotation - it is the
                // zombie-extraction adventure map now
                .filter(|m| **m != MapKind::Battlefield)
                .map(|m| (m.name(), MapButton(*m)))
                .collect();
            menu_ui::pill_row(b, "BATTLEFIELD", &maps, mtch);
            let diffs: Vec<(&str, DiffButton)> = Difficulty::ALL
                .iter()
                .map(|d| (d.name(), DiffButton(*d)))
                .collect();
            menu_ui::pill_row(b, "DIFFICULTY", &diffs, mtch);
            // §owner: 8v8 withdrawn. The battle-size row keeps its
            // shape for whatever replaces it rather than being deleted
            // outright - a one-option row still tells the player the
            // axis exists.
            let sizes: Vec<(&str, SizeButton)> = vec![("5 v 5", SizeButton(5))];
            menu_ui::pill_row(b, "BATTLE SIZE", &sizes, mtch);
            let targets: Vec<(&str, ScoreButton)> = vec![
                ("30 KILLS", ScoreButton(30)),
                ("60 KILLS", ScoreButton(60)),
            ];
            menu_ui::pill_row(b, "TDM SCORE", &targets, mtch);
            // Name and objective are separate cells now. As one 46-char
            // string they forced a 620px fixed width that wrapped and
            // overlapped the row beneath it at any smaller size.
            // §owner: ZOMBIE EXTRACTION is withdrawn from the menu. The
            // mode still exists in the sim - its tests and its systems
            // stay - but nothing user-facing offers it. Removing the ROW
            // is the whole change; the handler dispatches on the variant
            // and dead variants dispatch nothing.
            for (name, obj, which) in [
                ("TEAM DEATHMATCH", "first to your chosen score", ModeButton::Tdm),
                ("KING OF THE HILL", "hold the center 90 s", ModeButton::Koth),
                (
                    "TRAINING RANGE",
                    "still targets, nothing shoots back - learn the spray",
                    ModeButton::Training,
                ),
            ] {
                menu_ui::menu_row(
                    b,
                    (which, mtch),
                    menu_ui::RowKind::Normal,
                    name,
                    Some(obj),
                    None,
                );
            }

            // ---- nav ---------------------------------------------------
            // NO OnIntroPage here, deliberately: intro_paging's nav_vis
            // query is filtered Without<OnIntroPage>, so tagging these
            // would make it stop matching and the BACK/NEXT hide logic
            // would die silently, with no error.
            b.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(menu_ui::U3),
                margin: UiRect::top(Val::Px(menu_ui::U4)),
                ..default()
            })
            .with_children(|row| {
                for (label, which) in
                    [("< BACK", IntroNav::Back), ("NEXT >", IntroNav::Next)]
                {
                    menu_ui::menu_row(
                        row,
                        which,
                        menu_ui::RowKind::Normal,
                        label,
                        None,
                        None,
                    );
                }
            });

            menu_ui::page_pips(b, IntroPage::LAST + 1);
            menu_ui::seal_footer(b, brand, None);
            b.spawn((
                Text::new(""),
                TextFont { font_size: menu_ui::T_MICRO, ..default() },
                TextColor(branding::palette::GOLD),
                Node {
                    margin: UiRect::top(Val::Px(menu_ui::U2)),
                    ..default()
                },
                LobbyToast,
            ));
        });
    });
}

/// The big heading — the game's name on the title page, the page's own
/// question on the others.
#[derive(Component)]
struct IntroHeading;

/// The line under the heading, rewritten per page.
#[derive(Component)]
struct IntroSubtitle;

/// Shows only the current page's rows, and keeps the nav honest.
///
/// Everything on the intro screen is spawned ONCE, tagged with its page.
/// This system decides visibility - so no page change ever respawns the
/// tree, and the teardown path `close_intro` owns stays exactly as it
/// was. That is deliberate: respawning per page is how the lingering-
/// entity bug documented on `close_intro` would come back.
fn intro_paging(
    page: Res<IntroPage>,
    mut rows: Query<(&OnIntroPage, &mut Node)>,
    mut nav_vis: Query<(&IntroNav, &mut Node), Without<OnIntroPage>>,
    mut heading: Query<(&mut Text, &mut Node), (With<IntroHeading>, Without<OnIntroPage>, Without<IntroNav>, Without<IntroSubtitle>)>,
    mut subtitle: Query<&mut Text, (With<IntroSubtitle>, Without<IntroHeading>)>,
) {
    if !page.is_changed() {
        return;
    }
    // The title page is named by the WORDMARK PNG above the plate, so
    // the text heading hides there entirely rather than repeating the
    // same four words in a different typeface directly beneath it. The
    // other two pages name the question they answer.
    for (mut t, mut node) in &mut heading {
        if page.0 == IntroPage::TITLE {
            node.display = Display::None;
        } else {
            node.display = Display::Flex;
            **t = page.heading().to_string();
        }
    }
    for mut t in &mut subtitle {
        **t = page.subtitle().to_string();
    }
    // `Display::None` rather than `Visibility::Hidden`: a hidden node
    // still occupies layout space, which would leave the current page
    // floating in the middle of ten invisible rows.
    for (owner, mut node) in &mut rows {
        node.display = if owner.0 == page.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (which, mut node) in &mut nav_vis {
        let hide = match which {
            // nothing to go back to on the title page
            IntroNav::Back => page.0 == IntroPage::TITLE,
            // the deploy buttons ARE the forward action on the last
            // page; a NEXT that only clamps would lie about doing
            // something
            IntroNav::Next => page.0 == IntroPage::LAST,
        };
        node.display = if hide { Display::None } else { Display::Flex };
    }
}

/// Keyboard paging — the title page literally says "ENTER to begin",
/// so Enter must do that. Right/Left arrows page too; everything is
/// clamped the same way the buttons are (no wrapping - see the note on
/// `intro_nav_buttons`).
fn intro_keyboard_paging(keys: Res<ButtonInput<KeyCode>>, mut page: ResMut<IntroPage>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::ArrowRight) {
        page.0 = (page.0 + 1).min(IntroPage::LAST);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::Backspace) {
        page.0 = page.0.saturating_sub(1);
    }
}

/// Drives the nav buttons and paints their hover state.
fn intro_nav_buttons(
    mut page: ResMut<IntroPage>,
    mut q: Query<(&Interaction, &IntroNav, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.30, 0.24, 0.15)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.17, 0.14, 0.11)),
            Interaction::Pressed => {
                let p = page.0;
                page.0 = match which {
                    IntroNav::Back => p.saturating_sub(1),
                    // clamped, not wrapped: DEPLOY on the last page is
                    // handled by the mode buttons, and wrapping back to
                    // the title from there would read as a bug
                    IntroNav::Next => (p + 1).min(IntroPage::LAST),
                };
            }
        }
    }
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
        tdm_target: sel.tdm_target,
        class: sel.class,
        melee_axe: sel.melee_axe,
        grenade_preset: sel.grenade_preset,
        armor_pieces: Some(sel.armor),
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
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.30, 0.24, 0.15)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.17, 0.14, 0.11)),
            Interaction::Pressed => {
                let mode = match which {
                    ModeButton::Tdm => Mode::Tdm,
                    ModeButton::Koth => Mode::Koth,
                    ModeButton::Extraction => Mode::Extraction,
                    ModeButton::Training => Mode::Training,
                };
                start_match(&sel, mode, &mut game, &mut next);
            }
        }
    }
}

/// Shared select-highlight painter for all the pick-rows.
///
/// Rethemed to the key art: selection was a flat green that fought the
/// gold-and-bronze palette, and idle was a cool blue-grey. Selection is
/// now struck BRONZE — the same metal as the emblem's frame — so the
/// chosen item reads as part of the same object the title is made of.
///
/// One painter drives every pick-row on the loadout screen, which is
/// why this is a three-line change rather than forty scattered ones.
/// Paint one intro pill from the design system.
///
/// Was three hand-picked literals - a "struck bronze" selected state, a
/// hover, and a warm shadow - none of which came from the palette and
/// none of which matched the pause menu's treatment of the same idea.
/// Now it is the SAME `row_colors` every other interactive row uses, so
/// the two screens cannot drift.
///
/// The keel moves with the fill. The boss is left at its idle colour
/// here: reaching it needs a `Children` walk and a second query in all
/// seven of these handlers, and the plume ground plus the gold keel are
/// already two unmistakable signals for one binary state.
fn paint(
    bg: &mut BackgroundColor,
    border: &mut BorderColor,
    selected: bool,
    hovered: bool,
) {
    let state = menu_ui::row_state(
        selected,
        if hovered {
            Interaction::Hovered
        } else {
            Interaction::None
        },
    );
    let (fill, keel, _) = menu_ui::row_colors(menu_ui::RowKind::Normal, state);
    *bg = BackgroundColor(fill);
    *border = BorderColor(keel);
}

/// §7.2: the Forge rows. Same paint as every row, same save/load the
/// hotkeys use, same toast for feedback.
fn intro_forge_buttons(
    mut q: Query<
        (
            &Interaction,
            &ForgeUiButton,
            &menu_ui::PlateRow,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut bosses: Query<
        &mut BackgroundColor,
        (With<menu_ui::RowBoss>, Without<menu_ui::PlateRow>),
    >,
    mut sel: ResMut<Selected>,
    mut toast: ResMut<Toast>,
) {
    for (i, which, row, kids, mut bg, mut border) in &mut q {
        menu_ui::paint_row(row.kind, false, *i, &mut bg, &mut border, Some(kids), &mut bosses);
        if *i != Interaction::Pressed {
            continue;
        }
        match *which {
            ForgeUiButton::Save(slot) => {
                let p = ForgeProfile::from_selected(&sel);
                toast.text = match forge_save(slot, &p) {
                    Ok(()) => format!("FORGE: saved to slot {slot}"),
                    Err(_) => format!("FORGE: could not save slot {slot}"),
                };
                toast.t = 1.8;
            }
            ForgeUiButton::Load(slot) => {
                toast.text = match forge_load(slot) {
                    Some(p) => {
                        p.apply_to(&mut sel);
                        format!("FORGE: loaded slot {slot}")
                    }
                    None => format!("FORGE: slot {slot} is empty"),
                };
                toast.t = 1.8;
            }
            ForgeUiButton::Randomize => {
                // Client-side cosmetic dice - deliberately NOT the sim's
                // seeded RNG. Determinism governs the simulation; which
                // hat fate hands you in the lobby is allowed to be fate.
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() ^ u64::from(d.subsec_nanos()))
                    .unwrap_or(7);
                let mut r = seed;
                let mut roll = |n: usize| -> usize {
                    r = r
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    ((r >> 33) as usize) % n.max(1)
                };
                sel.loadout[0] = PRIMARIES[roll(PRIMARIES.len())];
                sel.loadout[1] = SECONDARIES[roll(SECONDARIES.len())];
                sel.loadout[2] = SPECIALS[roll(SPECIALS.len())];
                sel.melee_axe = roll(2) == 1;
                sel.grenade_preset = roll(GRENADE_PRESETS.len());
                sel.hat = roll(HAT_CHOICES.len());
                sel.tunic = roll(TUNIC_CHOICES.len());
                sel.helmet = roll(HELMET_CHOICES.len());
                // §C tier 2: roll the harness too - but only DOWN from
                // the class default, never up past the ceiling. A
                // randomize that could hand you a movement penalty would
                // be a button that makes you worse, and the player did
                // not ask to be gambled with; they asked for a look.
                {
                    let mut a = sim::default_harness(sel.class);
                    for p in sim::ArmorPiece::ALL {
                        if a.has(p) && roll(3) == 0 {
                            a.set(p, false);
                        }
                    }
                    sel.armor = a;
                }
                toast.text = "FORGE: fate rolled".to_string();
                toast.t = 1.8;
            }
        }
    }
}

/// §C tier 2: the armour grid - toggles, and the weight they cost.
///
/// A TOGGLE, unlike every other row on this page. That is why the press
/// handler flips a bit instead of assigning one, and why the paint pass
/// asks `has` rather than comparing against a single chosen value.
///
/// The press must fire on the RISING edge. Every picker row on this page
/// can safely assign on `Pressed` every frame it is held - assigning the
/// same value twice is a no-op - but a toggle held for ten frames would
/// flip ten times and land on whichever parity the frame rate happened to
/// produce. `Local<Option<Entity>>` remembers which pill is mid-press.
fn intro_armor_buttons(
    mut q: Query<
        (Entity, &Interaction, &ArmorButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut held: Local<Option<Entity>>,
    mut sel: ResMut<Selected>,
    mut readout: Query<&mut Text, With<ArmorWeightText>>,
) {
    let mut pressed_now = None;
    for (e, i, ab, _, _) in &mut q {
        if *i == Interaction::Pressed {
            pressed_now = Some(e);
            if *held != Some(e) {
                sel.armor.toggle(ab.0);
            }
        }
    }
    *held = pressed_now;
    for (_, i, ab, mut bg, mut border) in &mut q {
        paint(
            &mut bg,
            &mut border,
            sel.armor.has(ab.0),
            *i == Interaction::Hovered,
        );
    }
    // the live cost. Stated as the penalty in m/s rather than as a bare
    // overage in kg, because m/s is the unit the player experiences and
    // kg is the unit the spreadsheet does.
    for mut t in &mut readout {
        let kg = sel.armor.weight_kg();
        let budget = sim::class_spec(sel.class).weight_budget_kg;
        let pen = sim::armor_weight_movement_penalty(kg, budget);
        **t = if pen > 0.0 {
            format!("PLATE {kg:.1} / {budget:.0} kg   OVER by {:.1} kg - move -{pen:.2} m/s", kg - budget)
        } else {
            format!("PLATE {kg:.1} / {budget:.0} kg   {:.1} kg spare - no penalty", budget - kg)
        };
    }
}

/// §C tier 2: the five standard harnesses.
///
/// A plain PICKER, unlike the grid it sits above - pressing one assigns
/// a whole harness, so it can assign every frame it is held the way the
/// class and weapon rows do, with no rising-edge bookkeeping.
///
/// It lights only while the harness still MATCHES a preset. Touch one
/// plate in the grid and every preset goes dark, because at that point
/// none of them describes what is equipped, and a row that stayed lit
/// would be telling you something false about your own soldier.
fn intro_armor_preset_buttons(
    mut q: Query<
        (&Interaction, &ArmorPresetButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, pb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.armor = sim::preset_harness(pb.0, sel.class);
        }
    }
    let now = sim::preset_of(sel.armor, sel.class);
    for (i, pb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, now == Some(pb.0), *i == Interaction::Hovered);
    }
}

fn intro_map_buttons(
    mut q: Query<
        (&Interaction, &MapButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, mb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.map = mb.0;
        }
    }
    for (i, mb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.map == mb.0, *i == Interaction::Hovered);
    }
}

fn intro_loadout_buttons(
    mut q: Query<
        (&Interaction, &LoadoutButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, lb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.loadout[lb.0] = lb.1;
        }
    }
    for (i, lb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.loadout[lb.0] == lb.1, *i == Interaction::Hovered);
    }
}

fn intro_cosmetic_buttons(
    mut q: Query<
        (&Interaction, &CosmeticButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, cb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            match cb.0 {
                CosmeticSlot::Helmet => sel.helmet = cb.1,
                CosmeticSlot::HatTint => sel.hat = cb.1,
                CosmeticSlot::Tunic => sel.tunic = cb.1,
            }
        }
    }
    for (i, cb, mut bg, mut border) in &mut q {
        let selected = cb.1
            == match cb.0 {
                CosmeticSlot::Helmet => sel.helmet,
                CosmeticSlot::HatTint => sel.hat,
                CosmeticSlot::Tunic => sel.tunic,
            };
        paint(&mut bg, &mut border, selected, *i == Interaction::Hovered);
    }
}

fn intro_score_buttons(
    mut q: Query<
        (&Interaction, &ScoreButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, sb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.tdm_target = sb.0;
        }
    }
    for (i, sb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.tdm_target == sb.0, *i == Interaction::Hovered);
    }
}

fn intro_class_buttons(
    mut q: Query<
        (&Interaction, &ClassButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, cb, _, _) in &mut q {
        if *i == Interaction::Pressed && sel.class != cb.0 {
            sel.class = cb.0;
            // §C tier 2: the harness follows the class. Each class has
            // its own weight ceiling, so a LINE harness carried onto a
            // SKIRMISHER is instantly 22 kg against a 20 kg budget - the
            // player would be handed a penalty by a button that says
            // nothing about armour. Resetting to the new class's default
            // always lands under budget, and anything they had built is
            // one click from being rebuilt on a body that can carry it.
            sel.armor = sim::default_harness(cb.0);
        }
    }
    for (i, cb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.class == cb.0, *i == Interaction::Hovered);
    }
}

fn intro_melee_buttons(
    mut q: Query<
        (&Interaction, &MeleeButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, mb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.melee_axe = mb.0;
        }
    }
    for (i, mb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.melee_axe == mb.0, *i == Interaction::Hovered);
    }
}

fn intro_nade_buttons(
    mut q: Query<
        (&Interaction, &NadeButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, nb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.grenade_preset = nb.0;
        }
    }
    for (i, nb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.grenade_preset == nb.0, *i == Interaction::Hovered);
    }
}

fn intro_diff_buttons(
    mut q: Query<
        (&Interaction, &DiffButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, db, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.difficulty = db.0;
        }
    }
    for (i, db, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.difficulty == db.0, *i == Interaction::Hovered);
    }
}

fn intro_size_buttons(
    mut q: Query<
        (&Interaction, &SizeButton, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut sel: ResMut<Selected>,
) {
    for (i, sb, _, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.per_team = sb.0;
        }
    }
    for (i, sb, mut bg, mut border) in &mut q {
        paint(&mut bg, &mut border, sel.per_team == sb.0, *i == Interaction::Hovered);
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

fn close_intro(mut commands: Commands, intro: Query<Entity, With<IntroRoot>>) {
    // ONE query, ONE loop.
    //
    // This used to chain four - IntroRoot, TechReadout, LobbyToast and
    // the branding - because `open_intro` spawned those as TOP-LEVEL
    // entities rather than children, so despawning the root alone left
    // the loadout spec sitting in the corner for the whole match (visible
    // in every committed mech capture). The fix then was to despawn them
    // all here, which introduced a SECOND bug: the toast carried both
    // `LobbyToast` and `IntroRoot`, so `despawn_recursive` ran on it
    // twice and Bevy logged B0003 on every exit from the lobby.
    //
    // Everything is now a child of the surface root, so one recursive
    // despawn is both necessary and sufficient. The history stays because
    // re-parenting anything to the top level would bring both bugs back.
    for e in &intro {
        commands.entity(e).despawn_recursive();
    }
}

fn open_menu(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    brand: Option<Res<branding::BrandAssets>>,
    // ONE window query. Two - even `&Window` plus `&mut Window` - are a
    // B0001 conflict and panic the state transition on entry.
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let aspect = windows
        .get_single()
        .map(|w| w.resolution.width() / w.resolution.height().max(1.0))
        .unwrap_or(menu_ui::KEY_ART_ASPECT);
    release_cursor(&mut cam, &mut windows);
    cam.ads = false; // no stale scope glass / zoom over the menu
    let brand = brand.as_deref();
    let root = menu_ui::spawn_surface(&mut commands, brand, aspect);
    commands.entity(root).insert(MenuRoot).with_children(|p| {
        menu_ui::plate(p, menu_ui::PLATE_W_PAUSE, |b| {
            menu_ui::title(b, "PAUSED");
            menu_ui::rule_and_boss(b, true);
            for (which, label, hint, kind) in PAUSE_ROWS {
                menu_ui::menu_row(b, which, kind, label, None, hint);
            }
            menu_ui::seal_footer(b, brand, Some(("ESC", "resume")));
        });
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
    brand: Option<Res<branding::BrandAssets>>,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let aspect = windows
        .get_single()
        .map(|w| w.resolution.width() / w.resolution.height().max(1.0))
        .unwrap_or(menu_ui::KEY_ART_ASPECT);
    release_cursor(&mut cam, &mut windows);
    cam.ads = false; // no stale scope glass over the menu

    // Left column carries three whole sections, right carries the
    // crosshair's nine rows. Columns break BETWEEN groups and never
    // inside one - that is what makes the grouping survive the split.
    // The old layout wrapped ROW-MAJOR through a fixed 1010px grid, which
    // put the minimap on/off and its own rotate toggle on opposite sides
    // of the same visual line.
    const LEFT: [SettingsGroup; 3] =
        [SettingsGroup::Aim, SettingsGroup::Minimap, SettingsGroup::Hud];
    const RIGHT: [SettingsGroup; 1] = [SettingsGroup::Crosshair];

    let brand = brand.as_deref();
    let root = menu_ui::spawn_surface(&mut commands, brand, aspect);
    commands.entity(root).insert(SettingsRoot).with_children(|p| {
        menu_ui::plate(p, menu_ui::PLATE_W_SETTINGS, |b| {
            menu_ui::title(b, "SETTINGS");
            menu_ui::rule_and_boss(b, true);
            // ONE line, warm. The old subtitle ran to two full lines in a
            // cool lavender - the only cool colour on a warm page - and
            // its second sentence duplicated a pause row one keypress away.
            b.spawn((
                Text::new("CHANGES APPLY IMMEDIATELY"),
                TextFont { font_size: menu_ui::T_MICRO, ..default() },
                TextColor(branding::palette::PARCHMENT_DIM),
                Node { margin: UiRect::bottom(Val::Px(menu_ui::U2)), ..default() },
            ));
            b.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(menu_ui::U6),
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|cols| {
                for groups in [&LEFT[..], &RIGHT[..]] {
                    cols.spawn(Node {
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|col| {
                        for g in groups {
                            menu_ui::eyebrow(col, g.title());
                            for (which, kind, group) in SETTINGS_ROWS {
                                if group != *g {
                                    continue;
                                }
                                let full = settings_label_text(kind, &settings);
                                let (name, value) = split_label(&full);
                                menu_ui::menu_row_at(
                                    col,
                                    (which, SettingsLabel(kind)),
                                    menu_ui::RowKind::Normal,
                                    name,
                                    Some(value),
                                    None,
                                    menu_ui::ROW_H_DENSE,
                                );
                            }
                        }
                    });
                }
            });
            // Back is NOT an 18th setting. Full width, below both columns,
            // carrying the key that already does it.
            menu_ui::menu_row_at(
                b,
                SettingsButton::Back,
                menu_ui::RowKind::Normal,
                "BACK",
                None,
                Some("ESC"),
                menu_ui::ROW_H_DENSE,
            );
            menu_ui::seal_footer(b, brand, None);
        });
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
        // §4.3
        SettingsButtonKind::MinimapRotate => format!(
            "Minimap rotates with facing: {}",
            if s.minimap_rotate { "ON" } else { "OFF  (north-up)" }
        ),
        SettingsButtonKind::MinimapScale => {
            format!("Minimap size: {}%", s.minimap_scale)
        }
        // §4.1
        SettingsButtonKind::VitalsStyle => format!(
            "Vitals readout: {}",
            if s.hud_vitals_style == 0 {
                "numbers + bars"
            } else {
                "numbers only"
            }
        ),
        SettingsButtonKind::Sens => {
            format!("Mouse sensitivity: {}", SENS_CHOICES[s.sens_idx].0)
        }
        SettingsButtonKind::Fov => {
            format!("Field of view: {} deg", FOV_CHOICES[s.fov_idx].0)
        }
        SettingsButtonKind::InvertY => {
            format!("Invert look Y: {}", if s.invert_y { "ON" } else { "OFF" })
        }
        // §4.6 - every row prints its LIVE value, so the settings page
        // and the drawn crosshair read off the same field
        SettingsButtonKind::CrossSize => format!("Crosshair size: {}", s.cross_size),
        SettingsButtonKind::CrossGap => format!("Crosshair gap: {}", s.cross_gap),
        SettingsButtonKind::CrossThickness => {
            format!("Crosshair thickness: {}", s.cross_thickness)
        }
        SettingsButtonKind::CrossDot => {
            format!("Crosshair dot: {}", if s.cross_dot { "ON" } else { "OFF" })
        }
        SettingsButtonKind::CrossOutline => {
            if s.cross_outline {
                format!("Crosshair outline: ON  ({} px)", s.cross_outline_px)
            } else {
                "Crosshair outline: OFF".to_string()
            }
        }
        SettingsButtonKind::CrossColor => {
            let idx = s.cross_color_idx.min(CROSS_COLOR_CUSTOM_IDX);
            let (r, g, b) = crosshair_rgb(s);
            format!(
                "Crosshair colour: {}  ({r},{g},{b})",
                CROSS_COLOR_CHOICES[idx].0
            )
        }
        SettingsButtonKind::CrossAlpha => format!("Crosshair alpha: {}", s.cross_alpha),
        SettingsButtonKind::CrossTShape => format!(
            "Crosshair T-shape: {}",
            if s.cross_t_shape { "ON" } else { "OFF" }
        ),
        SettingsButtonKind::CrossDynamic => format!(
            "Crosshair style: {}",
            if s.cross_dynamic { "DYNAMIC" } else { "STATIC" }
        ),
    }
}

fn close_settings(mut commands: Commands, q: Query<Entity, With<SettingsRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn settings_buttons(
    mut q: Query<
        (
            &Interaction,
            &SettingsButton,
            &menu_ui::PlateRow,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut bosses: Query<
        &mut BackgroundColor,
        (With<menu_ui::RowBoss>, Without<menu_ui::PlateRow>),
    >,
    mut settings: ResMut<GameSettings>,
    mut labels: Query<(&SettingsLabel, &mut Text)>,
    mut next: ResMut<NextState<GameState>>,
) {
    let mut dirty = false;
    for (interaction, which, row, kids, mut bg, mut border) in &mut q {
        menu_ui::paint_row(
            row.kind,
            false,
            *interaction,
            &mut bg,
            &mut border,
            Some(kids),
            &mut bosses,
        );
        if *interaction == Interaction::Pressed {
            match which {
                SettingsButton::SwapMouse => {
                    settings.swap_mouse = !settings.swap_mouse;
                    dirty = true;
                }
                SettingsButton::Minimap => {
                    settings.minimap = !settings.minimap;
                    dirty = true;
                }
                // §4.3
                SettingsButton::MinimapRotate => {
                    settings.minimap_rotate = !settings.minimap_rotate;
                    dirty = true;
                }
                SettingsButton::MinimapScale => {
                    // steps of 5%, wrapping at the brief's 25..100 range
                    settings.minimap_scale += 5;
                    if settings.minimap_scale > MINIMAP_SCALE_RANGE.1 {
                        settings.minimap_scale = MINIMAP_SCALE_RANGE.0;
                    }
                    dirty = true;
                }
                // §4.1
                SettingsButton::VitalsStyle => {
                    settings.hud_vitals_style = 1 - settings.hud_vitals_style;
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
                // §4.6: numeric rows step forward and wrap through the
                // SAME range constants `parse_settings` clamps to, so a
                // value produced by clicking can never be a value the
                // file loader would reject.
                SettingsButton::CrossSize => {
                    settings.cross_size = cycle_i32(settings.cross_size, CROSS_SIZE_RANGE);
                    dirty = true;
                }
                SettingsButton::CrossGap => {
                    settings.cross_gap = cycle_i32(settings.cross_gap, CROSS_GAP_RANGE);
                    dirty = true;
                }
                SettingsButton::CrossThickness => {
                    settings.cross_thickness =
                        cycle_i32(settings.cross_thickness, CROSS_THICK_RANGE);
                    dirty = true;
                }
                SettingsButton::CrossDot => {
                    settings.cross_dot = !settings.cross_dot;
                    dirty = true;
                }
                // one row for both outline fields: it cycles the width
                // up and rolls OFF past the top, so the toggle and the
                // width never disagree about whether an outline is drawn
                SettingsButton::CrossOutline => {
                    if !settings.cross_outline {
                        settings.cross_outline = true;
                        settings.cross_outline_px = CROSS_OUTLINE_RANGE.0.max(1);
                    } else if settings.cross_outline_px >= CROSS_OUTLINE_RANGE.1 {
                        settings.cross_outline = false;
                    } else {
                        settings.cross_outline_px += 1;
                    }
                    dirty = true;
                }
                SettingsButton::CrossColor => {
                    settings.cross_color_idx =
                        (settings.cross_color_idx + 1) % CROSS_COLOR_CHOICES.len();
                    dirty = true;
                }
                SettingsButton::CrossAlpha => {
                    settings.cross_alpha = cycle_alpha(settings.cross_alpha);
                    dirty = true;
                }
                SettingsButton::CrossTShape => {
                    settings.cross_t_shape = !settings.cross_t_shape;
                    dirty = true;
                }
                SettingsButton::CrossDynamic => {
                    settings.cross_dynamic = !settings.cross_dynamic;
                    dirty = true;
                }
                SettingsButton::Back => next.set(GameState::Paused),
            }
        }
    }
    if dirty {
        for (l, mut t) in &mut labels {
            // the VALUE half only - the name node is static. `labels` is
            // an UNSCOPED query, so one SettingsLabel per row is a
            // correctness requirement, not a style preference: a second
            // one would have this write the same string into both nodes.
            **t = split_label(&settings_label_text(l.0, &settings)).1.to_string();
        }
    }
}

// ---- rules & manual (§14): generated from the live weapon table ----------

fn open_manual(
    mut commands: Commands,
    settings: Res<GameSettings>,
    brand: Option<Res<branding::BrandAssets>>,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let aspect = windows
        .get_single()
        .map(|w| w.resolution.width() / w.resolution.height().max(1.0))
        .unwrap_or(menu_ui::KEY_ART_ASPECT);
    release_cursor(&mut cam, &mut windows);
    cam.ads = false; // no stale scope glass over the menu

    // The ONLY sanctioned way to name the mouse buttons. The settings
    // label and this manual once derived the mapping independently and
    // BOTH had it backwards - on the very control that changes it.
    let (aim_b, fire_b) = mouse_map_names(settings.swap_mouse);

    // Every number below is the LIVE constant, never a retyped copy. The
    // old prose hardcoded all of them, and the weapon table twelve lines
    // under it was already derived correctly - one screen, two policies.
    let shield = format!(
        "Always carried. Blocks the FRONT ARC only (+/-{:.0}deg): standing\n\
         cuts damage {:.0}%, crouched {:.0}%. Sides and rear ignore it - FLANK.\n\
         Shield up = no shooting, slow walk.",
        SHIELD_ARC_COS.acos().to_degrees(),
        SHIELD_BLOCK_STAND * 100.0,
        SHIELD_BLOCK_CROUCH * 100.0,
    );
    let damage = format!(
        "{:.0} HP. Zones: head x{HEAD_MULT}, torso x1, arms x{ARM_MULT},\n\
         legs x{LEG_MULT}. Baseline M4A1: 2 headshots / 8 body shots.\n\
         {fire_b} fires; {aim_b} focuses every weapon.",
        MAX_HEALTH,
    );
    let checkpoints = "Stand in a white ring uncontested to flip it; your team then\n\
         respawns AT the ring. Contested rings freeze."
        .to_string();
    let modes = format!(
        "TDM first to {:.0} - KOTH hold the center {:.0} s -\n\
         {:.0}-min clock, {:.0} s sudden-death overtime.",
        TDM_TARGET,
        KOTH_TARGET_S,
        MATCH_LEN_S / 60.0,
        OVERTIME_S,
    );

    let brand = brand.as_deref();
    let root = menu_ui::spawn_surface(&mut commands, brand, aspect);
    commands.entity(root).insert(ManualRoot).with_children(|p| {
        menu_ui::plate(p, menu_ui::PLATE_W_MANUAL, |b| {
            menu_ui::title(b, "FIELD MANUAL");
            menu_ui::rule_and_boss(b, true);
            b.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(menu_ui::U8),
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|cols| {
                // LEFT: the rules prose
                cols.spawn(Node {
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|col| {
                    for (head, body) in [
                        ("THE SHIELD", shield.as_str()),
                        ("DAMAGE MODEL", damage.as_str()),
                        ("CHECKPOINTS", checkpoints.as_str()),
                        ("MODES", modes.as_str()),
                    ] {
                        menu_ui::eyebrow(col, head);
                        col.spawn((
                            Text::new(body.to_string()),
                            TextFont { font_size: menu_ui::T_DATA, ..default() },
                            TextColor(branding::palette::PARCHMENT),
                        ));
                    }
                    menu_ui::eyebrow(col, "CONTROLS");
                    // The hand-written control list is GONE, not fixed. It
                    // duplicated BIND_REGISTRY, omitted eight binds, and
                    // described keys this session had already remapped -
                    // two screens on the same pause menu disagreeing about
                    // the controls. One pointer now.
                    menu_ui::bind_row(col, "ESC", "MENU > CONTROLS for the full bind list", false);
                });
                // RIGHT: the weapon table, still derived from live specs
                cols.spawn(Node {
                    flex_grow: 1.0,
                    flex_basis: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|col| {
                    menu_ui::eyebrow(col, "WEAPONS");
                    col.spawn((
                        Text::new("torso dmg / shots to kill body / head / mag"),
                        TextFont { font_size: menu_ui::T_MICRO, ..default() },
                        TextColor(branding::palette::PARCHMENT_DIM),
                        Node { margin: UiRect::bottom(Val::Px(menu_ui::U2)), ..default() },
                    ));
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
                            (MAX_HEALTH / (s.damage * HEAD_MULT * s.pellets.max(1) as f32)).ceil()
                                as u32
                        };
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(menu_ui::U3),
                            ..default()
                        })
                        .with_children(|r| {
                            // real fixed-width name cell - the old screen
                            // faked columns with {:<14} format padding
                            r.spawn((
                                Text::new(s.name.to_string()),
                                TextFont { font_size: menu_ui::T_DATA, ..default() },
                                TextColor(branding::palette::PARCHMENT),
                                Node {
                                    width: Val::Px(menu_ui::ROW_LABEL_W),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                            ));
                            r.spawn((
                                Text::new(format!(
                                    "{:>5.1}{}  body x{}  head x{}{}  mag {}",
                                    s.damage,
                                    if s.pellets > 1 {
                                        format!(" x{} pellets", s.pellets)
                                    } else {
                                        String::new()
                                    },
                                    body_stk,
                                    head_stk,
                                    // honesty clause: pellet numbers assume
                                    // the WHOLE spread lands
                                    if s.pellets > 1 { " (full spread)" } else { "" },
                                    s.mag
                                )),
                                TextFont { font_size: menu_ui::T_DATA, ..default() },
                                TextColor(branding::palette::PARCHMENT),
                            ));
                        });
                    }
                });
            });
            menu_ui::seal_footer(b, brand, Some(("ESC", "BACK")));
        });
    });
}

fn close_manual(mut commands: Commands, q: Query<Entity, With<ManualRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn menu_buttons(
    mut q: Query<
        (
            &Interaction,
            &MenuButton,
            &menu_ui::PlateRow,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut bosses: Query<
        &mut BackgroundColor,
        (With<menu_ui::RowBoss>, Without<menu_ui::PlateRow>),
    >,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, which, row, kids, mut bg, mut border) in &mut q {
        // ONE painter for every interactive row in the menus, so five
        // handlers cannot drift into five different hover treatments.
        // Nothing in the pause menu is "selected" - it is a list of
        // actions, not a set of choices.
        menu_ui::paint_row(
            row.kind,
            false,
            *interaction,
            &mut bg,
            &mut border,
            Some(kids),
            &mut bosses,
        );
        if *interaction == Interaction::Pressed {
            match which {
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
            }
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

    /// §1.4a screen-intrusion, Brief VII §3.3/§4.3: EVERY weapon, at ITS
    /// OWN carry, with ITS OWN measured geometry, under ITS OWN profile,
    /// across every pose it can be held in indefinitely.
    ///
    /// This replaces a sweep that could see almost none of that. The old
    /// one ran a single hard-coded root, `(0.11, -0.13, 0.32)` - the
    /// GENERIC carry - against a pair of DECLARED envelope boxes. So the
    /// pistol's, the M249's, the spear's and the bow's own placements
    /// were never checked; every pose shift was unbounded (full draw
    /// pulls the bow 7.5 cm toward the midline and nothing looked); and
    /// the envelope it did check was a transcription that had drifted
    /// 2.3x off the real geometry. It passed the whole time.
    ///
    /// Nothing it covered is lost: stance, bob phase, fire kick, sprint
    /// and air are all still swept, now per weapon.
    ///
    /// SUSTAINED poses only, and that is a real distinction rather than a
    /// convenience. A drawn bow, a cooked grenade, a sprint and a
    /// suppression shake are states you can sit in for as long as you
    /// like, so an intrusion there is permanent. A reload, an inspect and
    /// a melee swing are committed actions that run to completion on
    /// their own clocks - a swing that crosses the whole frame is the
    /// READ the defender is being given, and clamping it would be
    /// removing the tell. `transient_poses_cannot_be_held_open` keeps
    /// that from becoming a loophole.
    #[test]
    fn every_weapon_holds_its_own_screen_profile() {
        let r_at = |z: f32| 0.24 * z * (VM_FOV_DEG.to_radians() * 0.5).tan();
        for kind in ALL_WEAPONS {
            let prof = screen_profile(kind);
            // MEASURED off the model, not transcribed from it. The
            // aggregate answers the MIDLINE question (how far left does
            // anything reach); the per-part corners answer the CIRCLE
            // one, because that needs a point the weapon really occupies.
            let (bl, bu) = weapon_bounded_extent(kind);
            let corners = weapon_bounded_corners(kind);
            let (carry, _) = vm_carry(kind);
            // the root, in the same frame the sweep above uses: vm_carry
            // stores forward as NEGATIVE z, the camera frame as positive
            let base = Vec3::new(carry.x, carry.y, -carry.z);
            for sf in [0.0_f32, 0.3, 0.6, 1.0] {
                for th in 0..80 {
                    for grounded in [true, false] {
                        for (kick, sp) in
                            [(0.0_f32, 0.0_f32), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
                        {
                            for pull in [0.0_f32, 1.0] {
                                for cook in [0.0_f32, 1.0] {
                                    // worst case: every sustained shift at
                                    // its most intrusive, all at once.
                                    // They cannot all peak together in
                                    // play, which is the point - the
                                    // bound has to hold even then.
                                    //
                                    // The draw shift is the BOW's -
                                    // applying it to a pistol is how the
                                    // first run of this test "found" a
                                    // midline crossing on the Glock that
                                    // no player could ever see.
                                    // the draw shift is the BOW's -
                                    // applying it to a pistol is how the
                                    // first run of this test "found" a
                                    // midline crossing on the Glock that
                                    // no player could ever see - and it
                                    // yields to the coil exactly as the
                                    // render path does
                                    let draw = if kind == GunKind::Bow {
                                        pull * (1.0 - cook)
                                    } else {
                                        0.0
                                    };
                                    let pose = base
                                        + carry_offset(
                                            sf,
                                            th as f32 * 0.173,
                                            grounded,
                                            kick,
                                            sp,
                                            0.04,
                                            0.0,
                                        )
                                        + VM_BOW_DRAW_SHIFT * draw
                                        + VM_GRENADE_SHIFT * cook
                                        - VM_SUPPRESS_SHAKE;
                                    let sway =
                                        pose.z.abs() * VM_SWAY_CAP_DEG.to_radians().tan();
                                    let r = r_at(pose.z.abs());
                                    match prof {
                                        // the bow is symmetric: no midline
                                        // test, a VERTICAL one instead
                                        ScreenProfile::BowDrawn => {
                                            assert!(
                                                pose.y + bu < -r,
                                                "{kind:?}: the bow reaches the crosshair \
                                                 - top {:.3} vs circle {:.3} \
                                                 (sf {sf} th {th} pull {pull})",
                                                pose.y + bu,
                                                -r
                                            );
                                        }
                                        _ => {
                                            // midline: the widest thing on
                                            // the weapon, wherever it is
                                            assert!(
                                                pose.x - bl - sway > 0.0,
                                                "{kind:?} ({prof:?}): the bounded part \
                                                 crosses the midline at {:.3} \
                                                 (sf {sf} th {th} cook {cook})",
                                                pose.x - bl - sway
                                            );
                                            // circle: every REAL corner
                                            for (pl, pu) in &corners {
                                                let dx = pose.x - pl - sway;
                                                let dy = pose.y + pu;
                                                let d = (dx * dx + dy * dy).sqrt();
                                                assert!(
                                                    d > r,
                                                    "{kind:?} ({prof:?}): a part corner \
                                                     is inside the centre circle - \
                                                     d {d:.3} <= r {r:.3} \
                                                     (sf {sf} th {th} kick {kick} sp {sp})"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// What the arsenal's extremes actually ARE, pinned - so a model that
    /// grows past them names itself instead of drifting.
    ///
    /// This is the test that retired the audited budgets, and it retired
    /// them by disagreeing with them. `VM_RECEIVER_LEFT`/`VM_MAST_UP`
    /// carried the note "current widest receiver = minigun cluster (0.069
    /// left); tallest mast = AWM scope (0.085 up)". Measured:
    ///
    ///   widest   MINIGUN  0.065 up-left   (the note was close, and
    ///                                      conservative, which is fine)
    ///   tallest  M249     0.192 up        (the note said 0.085, and
    ///                                      named the wrong weapon)
    ///
    /// The M249 is 2.3x the claimed ceiling, and the reason is ordinary:
    /// its arched carry handle was raised clear of the sight line in a
    /// later change, and the comment describing the arsenal was not part
    /// of that change. Nothing could catch it, because the geometry could
    /// not be read - which is the whole argument for `weapon_parts`.
    ///
    /// It is NOT a screen intrusion. The M249 carries lower and further
    /// out than the generic placement (`vm_carry`), and
    /// `every_weapon_holds_its_own_screen_profile` clears it at its own
    /// carry. The defect was the claim, not the gun.
    #[test]
    fn the_arsenals_extremes_are_what_we_think_they_are() {
        let mut widest = (GunKind::Fists, 0.0_f32);
        let mut tallest = (GunKind::Fists, 0.0_f32);
        for kind in ALL_WEAPONS {
            if screen_profile(kind) != ScreenProfile::Strict {
                continue; // the polearm and the bow have their own rules
            }
            let (l, u) = weapon_bounded_extent(kind);
            if l > widest.1 {
                widest = (kind, l);
            }
            if u > tallest.1 {
                tallest = (kind, u);
            }
        }
        assert_eq!(
            widest.0,
            GunKind::Minigun,
            "the widest gun is the minigun's barrel cluster, not {:?} at {:.3}",
            widest.0,
            widest.1
        );
        assert!(
            (widest.1 - 0.065).abs() < 0.002,
            "the minigun now reaches {:.4} left, not 0.065 - if that is \
             deliberate, move this number and re-read the sweep",
            widest.1
        );
        assert_eq!(
            tallest.0,
            GunKind::M249,
            "the tallest gun is the M249's carry handle, not {:?} at {:.3}",
            tallest.0,
            tallest.1
        );
        assert!(
            (tallest.1 - 0.192).abs() < 0.002,
            "the M249 now reaches {:.4} up, not 0.192",
            tallest.1
        );
    }

    /// The profiles exempt what they say they exempt, and nothing else.
    ///
    /// The spear's is the one worth pinning: bounding its 1.85 m SHAFT
    /// would be bounding the raised javelin's whole silhouette, which is
    /// the thing the profile exists to permit.
    #[test]
    fn each_profile_exempts_exactly_what_it_claims() {
        let spear = weapon_parts(GunKind::Spear);
        let bounded: Vec<&WPart> = spear
            .iter()
            .filter(|w| profile_bounds_part(ScreenProfile::SpearRaised, w))
            .collect();
        assert!(!bounded.is_empty(), "the grip cannot be empty");
        assert!(
            bounded.len() < spear.len(),
            "SpearRaised must exempt SOMETHING, or it is just Strict"
        );
        // the shaft - the longest part - must be among the exempt
        let longest = spear
            .iter()
            .max_by(|a, b| a.half().z.partial_cmp(&b.half().z).unwrap())
            .unwrap();
        assert!(
            !profile_bounds_part(ScreenProfile::SpearRaised, longest),
            "the shaft is bounded - that is the javelin's silhouette, not an \
             intrusion"
        );
        // and every bounded part really is at the hand
        for w in bounded {
            assert!(
                w.pos.z.abs() <= GRIP_WINDOW_M,
                "a part at z {} is not the grip",
                w.pos.z
            );
        }
        // a gun exempts nothing
        for w in &weapon_parts(GunKind::Ak47) {
            assert!(profile_bounds_part(ScreenProfile::Strict, w));
        }
    }

    /// The lift out of `spawn_weapon_model` was a pure MOVE.
    ///
    /// Every weapon still has parts, the minigun still has exactly the
    /// nine that spin, and no other weapon has any - if the `spin` flag
    /// had leaked onto a shared helper this is where it would show.
    #[test]
    fn the_part_tables_survived_being_lifted_out() {
        for kind in ALL_WEAPONS {
            let parts = weapon_parts(kind);
            assert!(!parts.is_empty(), "{kind:?} has no geometry at all");
            for w in &parts {
                assert!(w.size.min_element() > 0.0, "{kind:?}: a zero-sized part");
                assert!(w.pos.is_finite() && w.size.is_finite(), "{kind:?}: NaN part");
            }
            let spinning = parts.iter().filter(|w| w.spin).count();
            let expect = if kind == GunKind::Minigun { 9 } else { 0 };
            assert_eq!(
                spinning, expect,
                "{kind:?} has {spinning} spinning parts, expected {expect} - \
                 the barrel cluster is a spine, two caps and six barrels"
            );
        }
        // `Fists` is the one weapon that legitimately has no model, and
        // it is not in ALL_WEAPONS - assert that stays true, since the
        // loop above would fail loudly if it were ever added
        assert!(!ALL_WEAPONS.contains(&GunKind::Fists));
    }

    /// The directional read: the strip that lights is the one facing the
    /// fire, on all four bearings.
    ///
    /// This is the whole value of the suppression HUD - without a correct
    /// bearing it is a screen flash that tells a player to panic in an
    /// unspecified direction. The right/left convention is the easy one
    /// to get backwards, and getting it backwards would send someone
    /// turning INTO the gun, so both are pinned explicitly.
    #[test]
    fn the_lit_edge_faces_the_fire_on_every_bearing() {
        let me = [0.0_f32, 0.0, 0.0];
        // camera looking down +Z (yaw 0)
        assert_eq!(edge_toward([0.0, 0.0, 10.0], me, 0.0), Some(0), "ahead");
        assert_eq!(edge_toward([0.0, 0.0, -10.0], me, 0.0), Some(2), "behind");
        assert_eq!(edge_toward([-10.0, 0.0, 0.0], me, 0.0), Some(1), "screen-right");
        assert_eq!(edge_toward([10.0, 0.0, 0.0], me, 0.0), Some(3), "screen-left");
        // turning the camera turns the read with it: face the man who was
        // behind you and he is now ahead
        assert_eq!(edge_toward([0.0, 0.0, -10.0], me, PI), Some(0));
        // a shooter standing exactly on you has no bearing to report
        assert_eq!(edge_toward(me, me, 0.0), None);
        // and height never decides a compass bearing
        assert_eq!(
            edge_toward([0.0, 40.0, 10.0], me, 0.0),
            Some(0),
            "a mech firing down from a hull is still AHEAD"
        );
    }

    /// A hit always outranks being shot at, and it does so by
    /// construction rather than by a priority rule.
    #[test]
    fn being_hit_outshouts_being_shot_at() {
        assert!(
            SUPPRESS_EDGE_ALPHA * 2.0 <= 0.55 + 1e-6,
            "suppression's ceiling ({SUPPRESS_EDGE_ALPHA}) must stay far \
             enough under the damage flash's 0.55 that a hit wins the strip \
             on alpha alone - the moment they overlap, the two messages stop \
             being distinguishable and the shared widget stops being honest"
        );
        assert!(SUPPRESS_EDGE_ALPHA > 0.1, "and it still has to be visible");
    }

    /// The exemption above is only honest if the exempt poses are bounded
    /// in BOTH senses: they displace the weapon by a finite amount, and
    /// they run on clocks that finish.
    ///
    /// The first draft of this asserted something stronger and wrong -
    /// that a reload pose is the identity at r = 0 and r = 1, so it begins
    /// and ends exactly at the carry. The SHOTGUN disproves it: its
    /// shell-by-shell feed holds a constant 6 cm dip for the whole reload
    /// because the gun is held low at the loading gate throughout, so it
    /// steps to the dip the instant the reload starts. That is the pose
    /// the weapon is supposed to have, not a defect, and a test that
    /// demanded otherwise would have been demanding a worse animation.
    #[test]
    fn transient_poses_are_bounded_in_travel_and_in_time() {
        // No reload displaces the weapon further than this at any point
        // in its run. Generous - it is a cap on runaway, not a style
        // note - but finite, which is the whole claim.
        const RELOAD_TRAVEL_CAP_M: f32 = 0.30;
        for kind in ALL_WEAPONS {
            for step in 0..=100 {
                let r = step as f32 / 100.0;
                let (t, _) = reload_pose(kind, r);
                assert!(
                    t.length() < RELOAD_TRAVEL_CAP_M,
                    "{kind:?} at r {r}: the reload throws the weapon {:.3} m - \
                     an exempt pose still has to come back",
                    t.length()
                );
                assert!(t.is_finite(), "{kind:?} at r {r}: non-finite pose {t:?}");
            }
        }
        // and the melee windows are finite and positive - a swing that
        // never ended would hold the frame open forever, which is exactly
        // the loophole the exemption must not have
        for w in [
            KNIFE_QUICK_WIND_S,
            KNIFE_QUICK_ACTIVE_S,
            KNIFE_QUICK_RECOVER_S,
            AXE_QUICK_WIND_S,
            AXE_QUICK_ACTIVE_S,
            AXE_QUICK_RECOVER_S,
        ] {
            assert!(w > 0.0 && w < 2.0, "a melee window of {w}s is not a swing");
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
        // §4.5 killfeed glyphs from a scripted stream. Each modifier has
        // its own mark, they COMPOSE (a blind noscope headshot earns all
        // three), and a plain kill earns none - an empty string, not a
        // pad, so the row does not reserve space for a badge it lacks.
        let stream = [
            ((false, false, false, false, false), ""),
            ((true, false, false, false, false), "*"),
            ((false, true, false, false, false), "o"),
            ((false, false, true, false, false), "?"),
            ((false, false, false, true, false), "~"),
            ((false, false, false, false, true), "#"),
            ((true, true, false, false, false), "*o"),
            ((true, true, true, true, true), "*o?~#"),
        ];
        for ((hs, ns, bl, sm, wb), want) in stream {
            assert_eq!(
                feed_glyphs(hs, ns, bl, sm, wb),
                want,
                "glyphs for headshot={hs} noscope={ns} blind={bl} smoke={sm} wallbang={wb}"
            );
        }
    }

    /// §owner: every firearm carries a 1x red-dot optic, and the
    /// reticle sits at EXACTLY the height focus aligns to the eye.
    ///
    /// This is the test the M249 needed and did not have. It shipped
    /// with a sight line 2 mm above a flat feed cover and no rear
    /// aperture at all, so aiming laid a 30 cm plate across the view -
    /// a defect no unit test could see, because nothing tied the
    /// declared number to the geometry that number is about.
    ///
    /// `push_red_dot(y, _)` builds its cross at `y`; `sight_line_y`
    /// says where the eye goes. If those two ever disagree the optic
    /// becomes decoration and the player aims with a cross that is not
    /// on the target.
    #[test]
    fn every_firearm_carries_an_aligned_optic() {
        // (kind, the y passed to push_red_dot in spawn_weapon_model)
        let optics = [
            (sim::GunKind::Glock, 0.1075_f32),
            (sim::GunKind::Deagle, 0.1300),
            (sim::GunKind::Mp5, 0.1160),
            (sim::GunKind::Shotgun, 0.0950),
            (sim::GunKind::Ak47, 0.1060),
            (sim::GunKind::M4, 0.1120),
            (sim::GunKind::M249, 0.1265),
            (sim::GunKind::Minigun, 0.1120),
        ];
        for (kind, reticle_y) in optics {
            let declared = sight_line_y(kind).unwrap_or_else(|| {
                panic!("{kind:?} carries an optic but declares no sight line")
            });
            assert!(
                (declared - reticle_y).abs() < 1e-6,
                "{kind:?}: the reticle sits at {reticle_y} but focus aligns                  {declared} to the eye - the cross would not be on the                  crosshair"
            );
        }
        // The AWM is the deliberate exception: `vm_hidden_while_scoped`
        // deletes its viewmodel while zoomed, so a modelled optic could
        // never be seen. Its illuminated cross lives in the scope
        // overlay instead. Any sight line here would be a lie.
        assert!(
            sight_line_y(sim::GunKind::Awm).is_none(),
            "the scoped rifle must not declare a viewmodel sight line"
        );
        // Fists, bow and spear have no sights to align.
        for kind in [sim::GunKind::Fists, sim::GunKind::Bow, sim::GunKind::Spear] {
            assert!(sight_line_y(kind).is_none(), "{kind:?} has no sights");
        }
    }

    /// The optic must be a WINDOW, not a block: the frame bars sit
    /// outside the clear aperture, and the cross arms stop short of the
    /// frame. A reticle that touched the frame would read as painted-on
    /// rather than projected, and a frame that overlapped the window
    /// would be the grey wall all over again.
    #[test]
    fn the_optic_window_stays_clear() {
        let mut parts = Vec::new();
        push_red_dot(&mut parts, 0.10, 0.0, 0.06);
        let reticles: Vec<&WPart> =
            parts.iter().filter(|p| p.tone == Tone::Reticle).collect();
        assert_eq!(reticles.len(), 1, "the reticle is ONE dot");
        let dot = reticles[0];
        assert!(
            (dot.pos.y - 0.10).abs() < 1e-6 && dot.pos.x.abs() < 1e-6,
            "the dot must be centred on the sight line"
        );
        // and it must stay inside the glass even at FULL recoil drift -
        // a dot that slid behind the housing would read as a rendering
        // bug rather than as recoil
        let farthest = dot.size.y * 0.5 + RETICLE_DRIFT_M;
        assert!(
            farthest < OPTIC_HALF,
            "the dot leaves the window at full drift: {farthest} vs the              {OPTIC_HALF} aperture"
        );
        // every frame bar clears the aperture
        for b in parts.iter().filter(|p| p.tone == Tone::Black) {
            let dx = b.pos.x.abs() - b.size.x * 0.5;
            let dy = (b.pos.y - 0.10).abs() - b.size.y * 0.5;
            assert!(
                dx >= OPTIC_HALF - 1e-6 || dy >= OPTIC_HALF - 1e-6,
                "a frame bar intrudes into the clear window"
            );
        }
    }

    /// §1.4 Rule-2 gate: scoped + zoomed = the viewmodel is not rendered.
    #[test]
    /// §owner TEXTURE PIPELINE: the generators are pure functions of a
    /// seed, they TILE seamlessly, and they only ever darken.
    ///
    /// All three properties are load-bearing. Purity keeps every machine
    /// (and every capture) identical. Seamlessness is what makes a 128px
    /// tile usable across a 60 m map - a visible seam is worse than no
    /// texture. And "never brighter than white" is the one that already
    /// bit once: the first encoding centred the multiplier on mid-grey,
    /// which silently halved the brightness of every surface it touched.
    #[test]
    fn generated_textures_tile_and_only_darken() {
        // pure: same input, same output, every time
        for (x, y, seed) in [(0u32, 0u32, 7u32), (63, 17, 11), (127, 127, 29)] {
            assert_eq!(tex_hash(x, y, seed), tex_hash(x, y, seed));
            assert_eq!(tex_fbm(x, y, seed), tex_fbm(x, y, seed));
        }
        // seamless: the noise wraps, so the left edge matches where the
        // right edge would continue to
        for y in (0..TEX_SIZE).step_by(7) {
            let left = tex_noise(0, y, 8, 5);
            let wrapped = tex_noise(TEX_SIZE, y, 8, 5);
            assert!(
                (left - wrapped).abs() < 1e-6,
                "the tile must wrap at x: {left} vs {wrapped} at y={y}"
            );
        }
        for x in (0..TEX_SIZE).step_by(7) {
            let top = tex_noise(x, 0, 8, 5);
            let wrapped = tex_noise(x, TEX_SIZE, 8, 5);
            assert!(
                (top - wrapped).abs() < 1e-6,
                "the tile must wrap at y: {top} vs {wrapped} at x={x}"
            );
        }
        // in range: fbm never leaves 0..1, so no generator can be pushed
        // past white by its own noise term
        for x in (0..TEX_SIZE).step_by(11) {
            for y in (0..TEX_SIZE).step_by(11) {
                let v = tex_fbm(x, y, 3);
                assert!((0.0..=1.0).contains(&v), "fbm out of range: {v}");
            }
        }
    }

    /// §2.5: the Vec3 spring is the SAME math as the 2D one, not a
    /// second solver - it must critically damp on every axis, reach its
    /// target, and never overshoot.
    #[test]
    fn the_three_axis_spring_settles_without_overshoot() {
        let target = Vec3::new(0.4, -0.2, 0.7);
        let mut x = Vec3::ZERO;
        let mut v = Vec3::ZERO;
        let mut max_over = 0.0_f32;
        for _ in 0..400 {
            let (nx, nv) = damped_spring3(x, v, target, SPRING_K_HAND_FOLLOW, 1.0 / 120.0);
            x = nx;
            v = nv;
            // critical damping never crosses the target
            for a in 0..3 {
                let over = (x[a] - target[a]) * target[a].signum();
                max_over = max_over.max(over);
            }
        }
        assert!(
            max_over < 1e-3,
            "a critically damped spring must not overshoot, got {max_over}"
        );
        assert!(
            (x - target).length() < 1e-3,
            "the spring must actually arrive: {x:?} vs {target:?}"
        );
        // a stiffer spring must arrive sooner - this is what makes the
        // named k values mean something rather than being decoration
        let settle = |k: f32| -> usize {
            let (mut x, mut v) = (Vec3::ZERO, Vec3::ZERO);
            for i in 0..2000 {
                let (nx, nv) = damped_spring3(x, v, target, k, 1.0 / 120.0);
                x = nx;
                v = nv;
                if (x - target).length() < 0.01 {
                    return i;
                }
            }
            usize::MAX
        };
        assert!(
            settle(SPRING_K_FINGER_SETTLE) < settle(SPRING_K_HAND_FOLLOW),
            "fingers (k=220) must settle faster than the hand (k=120)"
        );
        assert!(
            settle(SPRING_K_HAND_FOLLOW) < settle(SPRING_K_ELBOW_POLE),
            "the hand (k=120) must arrive before the elbow pole (k=60) - \
             that lag IS the secondary motion"
        );
        assert!(
            settle(SPRING_K_ELBOW_POLE) < settle(SPRING_K_SHOULDER),
            "the clavicle (k=45) must be the slowest link in the chain"
        );
    }

    #[test]
    fn vm_hides_while_scoped() {
        assert!(vm_hidden_while_scoped(true, true));
        assert!(!vm_hidden_while_scoped(true, false));
        assert!(!vm_hidden_while_scoped(false, true));
        assert!(!vm_hidden_while_scoped(false, false));
    }

    /// The vm camera draws AFTER the UI camera, so a rendered gun would
    /// composite over every menu plate. The gate must close in every
    /// non-Playing state and open again in Playing.
    #[test]
    fn vm_hides_while_menu_open() {
        for s in [
            GameState::Intro,
            GameState::Paused,
            GameState::Settings,
            GameState::Manual,
            GameState::Controls,
        ] {
            assert!(!vm_rendered(&s, 0.0, true, 0.0), "hidden in {s:?}");
        }
        assert!(vm_rendered(&GameState::Playing, 0.0, true, 0.0));
        // the pre-existing gates still hold in Playing
        assert!(!vm_rendered(&GameState::Playing, 1.0, true, 0.0), "third person");
        assert!(!vm_rendered(&GameState::Playing, 0.0, false, 0.0), "dead");
        assert!(!vm_rendered(&GameState::Playing, 0.0, true, 0.5), "mid-roll");
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

    /// §C regression guard: client FX must follow the weapon that
    /// ACTUALLY fired.
    ///
    /// Every shot effect in this file — casings, muzzle flash, shot
    /// audio, camera kick — detects a fresh shot by a cooldown jumping
    /// UP. That worked while every weapon shared `fire_cd`. When the
    /// hull mounts got their own clocks (correctly: sharing `fire_cd`
    /// was throttling the pilot's carried gun on dismount) they silently
    /// stopped feeding those sites, and a firing hull gatling went
    /// flashless and silent.
    ///
    /// This pins the fix so a future clock change cannot un-wire it
    /// again without a red test.
    #[test]
    fn shot_clock_follows_the_weapon_that_actually_fired() {
        // ..Default::default() rather than listing every field: this
        // struct has grown twice already, and a test that only cares
        // about two fields should not break when a third is added.
        let mut s = sim::TdmSim::new(sim::MatchConfig {
            seed: 0xFEED,
            per_team: 1,
            ..Default::default()
        });

        // On foot, the carried gun's clock is the shot clock.
        {
            let f = &mut s.fighters[0];
            f.armor_set = sim::ArmorSet::None;
            f.fire_cd = 0.11;
            f.gatling_cd = 0.0;
            f.autocannon_cd = 0.0;
        }
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.11,
            "an infantry fighter's shots run on fire_cd"
        );

        // In a mech the HULL MOUNT's clock is the shot clock - and
        // crucially it must NOT read fire_cd, or the FX fire on the
        // pilot's carried gun instead of the weapon that shot.
        {
            let f = &mut s.fighters[0];
            f.armor_set = sim::ArmorSet::RobotSuit;
            f.hull = sim::MECH_HULL;
            f.fire_cd = 0.11; // the carried rifle, NOT firing
            f.mech_weapon = sim::MechWeapon::Gatling;
            f.gatling_cd = 0.07;
            f.autocannon_cd = 0.0;
        }
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.07,
            "a piloted gatling's shots must run on gatling_cd, not the \
             carried gun's fire_cd"
        );

        // ...and switching mounts follows the selection.
        s.fighters[0].mech_weapon = sim::MechWeapon::Autocannon;
        s.fighters[0].autocannon_cd = 1.35;
        assert_eq!(
            shot_clock(&s.fighters[0]),
            1.35,
            "selecting the autocannon must move the shot clock with it"
        );
        // §C.7: the pod rides its relaunch cooldown
        s.fighters[0].mech_weapon = sim::MechWeapon::Rockets;
        s.fighters[0].pod_cd = 0.9;
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.9,
            "the ROCKETS mount's shot clock is pod_cd"
        );
        s.fighters[0].pod_cd = 0.0;
        s.fighters[0].mech_weapon = sim::MechWeapon::Autocannon;

        // The regression this guards: with the mount idle, the shot
        // clock must be ZERO even though the carried gun is hot. If this
        // ever reads 0.11 again, every mech FX site is firing off the
        // pilot's rifle.
        s.fighters[0].autocannon_cd = 0.0;
        assert_eq!(
            shot_clock(&s.fighters[0]),
            0.0,
            "an idle hull mount must read idle - a hot carried gun must \
             not make the mech look like it is shooting"
        );
    }

    /// §4.7: the death camera's phase logic. Pure, and shared by the
    /// camera and the HUD - this is the function that decides whether
    /// you are watching your killer or following them, so its edges are
    /// exactly the cases that would put a live player in a spectate cam
    /// or point a corpse camera at nobody.
    #[test]
    fn death_phase_knows_when_to_watch_and_when_to_follow() {
        let mut s = sim::TdmSim::new(sim::MatchConfig {
            seed: 0xDEAD,
            per_team: 1,
            ..Default::default()
        });

        // alive: no death camera, full stop
        assert_eq!(death_phase(&s, 0), None, "a live fighter has no death cam");

        // freshly killed by the enemy: killer-cam first
        s.fighters[0].health = 0.0;
        s.fighters[0].respawn_t = sim::RESPAWN_S;
        s.fighters[0].last_hit_by = Some((1, 0.0));
        assert_eq!(
            death_phase(&s, 0),
            Some((1, false)),
            "a fresh death looks AT the killer before following them"
        );

        // past the killer-cam window: spectate
        s.fighters[0].respawn_t = sim::RESPAWN_S - KILLER_CAM_S - 0.1;
        assert_eq!(
            death_phase(&s, 0),
            Some((1, true)),
            "after the killer-cam beat the camera follows the killer"
        );

        // suicide: nobody worth watching - ordinary corpse view
        s.fighters[0].last_hit_by = Some((0, 0.0));
        assert_eq!(
            death_phase(&s, 0),
            None,
            "a cooked frag has no killer-cam - you did this to yourself"
        );

        // no recorded attacker at all (fell, or state cleared)
        s.fighters[0].last_hit_by = None;
        assert_eq!(death_phase(&s, 0), None, "no killer, no spectate target");
    }

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

/// The look-pitch clamp: recoil and the mouse write the same state, so
/// they must agree about how far it may go.
#[cfg(test)]
mod recoil_pitch_tests {
    use super::*;

    /// The bug this module exists for.
    ///
    /// Recoil clamps the ACCUMULATED pitch, not its own delta. While its
    /// limit was (-0.7, 0.8) and the mouse's was ±1.53, a player holding
    /// a steep aim had it yanked back to the recoil limit the instant
    /// they fired — up to 0.73 rad in one frame, from one bullet, with no
    /// input. Firing must never move the aim further than the kick.
    #[test]
    fn firing_at_a_steep_aim_does_not_teleport_the_view() {
        let kick = gun(sim::GunKind::Ak47).kick;
        for &pitch in &[
            -LOOK_PITCH_LIMIT,
            -1.2,
            -0.71, // just outside the OLD lower clamp
            0.0,
            0.81, // just outside the OLD upper clamp
            1.2,
            LOOK_PITCH_LIMIT,
        ] {
            let after = recoil_kicked_pitch(pitch, kick, 0.0, 1.0);
            let moved = (after - pitch).abs();
            let most = kick * 6.0 + 1e-6;
            assert!(
                moved <= most,
                "pitch {pitch} moved {moved} rad on one shot; the kick is only {most}"
            );
        }
    }

    /// ...and the guard above must not be satisfied by doing nothing.
    #[test]
    fn the_kick_still_kicks() {
        let kick = gun(sim::GunKind::Ak47).kick;
        let after = recoil_kicked_pitch(0.0, kick, 0.0, 1.0);
        assert!(after < 0.0, "recoil must raise the muzzle (lower pitch)");
        assert!(
            (after.abs() - kick * 6.0).abs() < 1e-6,
            "one shot should move exactly kick*6, got {after}"
        );
    }

    /// Recoil may push the aim TO the ceiling, never through it.
    #[test]
    fn sustained_fire_stops_at_the_same_limit_the_mouse_obeys() {
        let kick = gun(sim::GunKind::M249).kick;
        let mut pitch = 0.0;
        for _ in 0..500 {
            pitch = recoil_kicked_pitch(pitch, kick, 0.05, 1.0);
        }
        assert!(
            pitch >= -LOOK_PITCH_LIMIT,
            "ran past the look limit: {pitch}"
        );
        assert!(
            (pitch + LOOK_PITCH_LIMIT).abs() < 1e-3,
            "500 rounds should pin the aim at the ceiling, got {pitch}"
        );
    }

    /// A brace is a discount on the kick, in both directions of the
    /// ladder — the sim damps a braced mech's punch, so the view must
    /// damp too or the stance does nothing the player can feel.
    #[test]
    fn bracing_reduces_the_kick_by_the_sims_own_factor() {
        let kick = 0.018; // the autocannon's camera kick
        let unbraced = recoil_kicked_pitch(0.0, kick, 0.0, 1.0).abs();
        let braced =
            recoil_kicked_pitch(0.0, kick, 0.0, sim::MECH_BRACE_RECOIL_DAMP).abs();
        assert!(braced < unbraced, "bracing must help: {braced} vs {unbraced}");
        assert!(
            (braced - unbraced * sim::MECH_BRACE_RECOIL_DAMP).abs() < 1e-6,
            "the view should scale by the sim's own damp factor"
        );
        // and leaning on foot is the milder discount
        let leaned = recoil_kicked_pitch(0.0, kick, 0.0, sim::LEAN_RECOIL_MULT).abs();
        assert!(
            leaned > braced && leaned < unbraced,
            "lean sits between braced and unbraced: {braced} < {leaned} < {unbraced}"
        );
    }

    /// Bloom widens the kick, so a hot gun climbs faster than a cold one.
    #[test]
    fn a_hot_gun_climbs_faster_than_a_cold_one() {
        let kick = gun(sim::GunKind::Ak47).kick;
        let cold = recoil_kicked_pitch(0.0, kick, 0.0, 1.0).abs();
        let hot = recoil_kicked_pitch(0.0, kick, 0.05, 1.0).abs();
        assert!(hot > cold, "bloom should add climb: {hot} vs {cold}");
    }
}

/// The bowstring must report the draw the SIM is running, not a guess
/// assembled from the ADS toggle.
#[cfg(test)]
mod bow_draw_visual_tests {
    use super::*;

    const PERIOD: f32 = 0.95; // gun(Bow).fire_period

    /// The split brain this replaces: the pull was `1.0` while aiming and
    /// `0.25` otherwise, so the whole 0.15s..0.7s curve the sim runs was
    /// invisible - two positions standing in for a continuous draw.
    #[test]
    fn the_players_pull_tracks_the_sims_clock() {
        let at = |t: f32| bow_draw_visual(t, 0.0, PERIOD, true);
        assert_eq!(at(0.0), 0.0, "an untouched bow is slack");
        assert!(at(sim::BOW_DRAW_FULL_S) >= 1.0 - 1e-6, "0.7s is full draw");
        assert_eq!(at(5.0), 1.0, "holding past full stays full, never past it");

        // strictly increasing across the whole draw - a continuous pull,
        // which is exactly what the two-position version could not show
        let mut prev = -1.0;
        for i in 0..=70 {
            let v = at(i as f32 * 0.01);
            assert!(v >= prev, "pull went backwards at t={}", i as f32 * 0.01);
            prev = v;
        }
        // and it is genuinely partway at the halfway mark, not snapped
        let mid = at(sim::BOW_DRAW_FULL_S * 0.5);
        assert!(
            (0.4..0.6).contains(&mid),
            "half a draw should look half drawn, got {mid}"
        );
    }

    /// Bots never run `step_bow_draw`, so their clock is pinned at 0.
    /// Reading it directly would leave every bot bow permanently slack -
    /// a regression from the fixed 0.6 this replaced.
    #[test]
    fn a_bot_bow_still_draws_even_though_its_clock_never_runs() {
        let bot = |cd: f32| bow_draw_visual(0.0, cd, PERIOD, true.eq(&false));
        assert_eq!(bot(PERIOD), 0.0, "just loosed: string forward");
        assert!(bot(PERIOD * 0.5) > 0.4, "mid-cadence: drawing");
        assert_eq!(bot(0.0), 1.0, "about to loose: fully drawn");

        // the naive version of this fix - reading bow_draw_t for everyone
        let naive = bow_draw_visual(0.0, PERIOD * 0.5, PERIOD, true);
        assert_eq!(naive, 0.0, "which is why bots must not use that path");
    }

    /// A cadence-derived pull is meaningless without a period.
    #[test]
    fn a_zero_period_cannot_divide_by_zero() {
        assert_eq!(bow_draw_visual(0.0, 0.0, 0.0, false), 0.0);
    }

    /// Whatever the input, the string is a 0..1 quantity - the renderer
    /// multiplies anchor offsets by it.
    #[test]
    fn the_pull_is_always_a_unit_fraction() {
        for &(t, cd, player) in &[
            (-1.0, -1.0, true),
            (99.0, 99.0, true),
            (-1.0, -1.0, false),
            (99.0, 99.0, false),
        ] {
            let v = bow_draw_visual(t, cd, PERIOD, player);
            assert!((0.0..=1.0).contains(&v), "out of range: {v}");
        }
    }
}

/// The bow's string, arrow and draw hand, which must agree by
/// construction because for a long time they did not.
#[cfg(test)]
mod bow_string_tests {
    use super::*;

    /// Sample draws across the full range, ends included.
    const DRAWS: [f32; 6] = [0.0, 0.2, 0.45, 0.7, 0.9, 1.0];

    /// The two halves meet AT the nock, and their far ends stay pinned to
    /// the limb tips.
    ///
    /// This is the whole geometric claim of a bowstring and it is the one
    /// the single tip-to-tip box could not make: it had no vertex at the
    /// nock, so there was nothing to pull and nothing to check.
    #[test]
    fn both_string_halves_run_from_a_limb_tip_to_the_nock() {
        for d in DRAWS {
            let nock = bow_nock_local(d);
            for side in [-1.0_f32, 1.0] {
                let t = bow_string_half(side, d);
                // reconstruct the segment from the transform alone - if
                // the rotation and the length disagree this fails
                let half = t.rotation * Vec3::X * (t.scale.x * 0.5);
                let (a, b) = (t.translation - half, t.translation + half);
                let tip = Vec3::new(side * BOW_TIP_X, 0.0, BOW_TIP_Z);
                // whichever end is nearer the tip must BE the tip, and the
                // other must be the nock
                let (near_tip, near_nock) =
                    if (a - tip).length() < (b - tip).length() { (a, b) } else { (b, a) };
                assert!(
                    (near_tip - tip).length() < 1e-4,
                    "side {side} draw {d}: string leaves {near_tip:?}, tip is {tip:?}"
                );
                assert!(
                    (near_nock - nock).length() < 1e-4,
                    "side {side} draw {d}: string ends {near_nock:?}, nock is {nock:?}"
                );
            }
        }
    }

    /// Drawing makes the string LONGER, monotonically. A V is two sides of
    /// a triangle and both grow with the pull; a version that shortened
    /// would mean the halves were being rotated without being re-measured.
    #[test]
    fn the_string_lengthens_as_it_is_drawn() {
        let len = |d: f32| bow_string_half(1.0, d).scale.x;
        let rest = len(0.0);
        assert!(rest > BOW_TIP_X, "a slack string still spans the limb");
        let mut prev = rest;
        for d in [0.2_f32, 0.45, 0.7, 0.9, 1.0] {
            let l = len(d);
            assert!(l > prev, "draw {d}: {l} did not exceed {prev}");
            prev = l;
        }
        // and the nock really travels the full pull
        assert!(
            (bow_nock_local(1.0).z - (BOW_STRING_Z - BOW_DRAW_PULL)).abs() < 1e-6,
            "full draw must be exactly BOW_DRAW_PULL back"
        );
    }

    /// The arrow's NOCK sits on the string at every draw.
    ///
    /// The failure this pins is the one that shipped: the arrow hung off
    /// the bow HAND at a fixed offset, so it tracked a hand rather than a
    /// bow and never moved with the draw at all. Here the tail is computed
    /// from the arrow transform the same way the renderer will, so an
    /// arrow that floats off the cord fails.
    #[test]
    fn the_nocked_arrow_keeps_its_tail_on_the_string() {
        for d in DRAWS {
            let t = bow_nocked_arrow(d);
            let tail_z = t.translation.z + ARROW_NOCK_Z * t.scale.z;
            let nock = bow_nock_local(d);
            assert!(
                (tail_z - nock.z).abs() < 1e-5,
                "draw {d}: tail at z {tail_z}, string at z {nock:?}"
            );
            // and it points DOWNRANGE - +Z, never reversed
            assert!(
                t.translation.z > tail_z,
                "draw {d}: the arrow is facing backwards"
            );
        }
    }

    /// The shaft clears the riser it runs beside.
    ///
    /// Held horizontal there is no "on top of the riser" for an arrow to
    /// rest on, which is exactly what the old vertical-bow shelf assumed.
    /// The riser is 0.052 wide, so its face is at x 0.026.
    #[test]
    fn the_arrow_runs_clear_of_the_riser() {
        const RISER_HALF_W: f32 = 0.026;
        // the shaft is 0.020 across in the arrow's unit envelope, so it
        // scales with the nocked length like every other part of it
        let shaft_r = 0.020 * 0.5 * BOW_ARROW_LEN;
        assert!(
            BOW_ARROW_X - shaft_r > RISER_HALF_W,
            "the shaft at x {BOW_ARROW_X} (r {shaft_r}) cuts through a riser \
             half-width {RISER_HALF_W}"
        );
        // and the ADS arrow rest must be UNDER it, not above: the rest is
        // a side bracket at y -0.016 with a 0.018 box, top face -0.007
        assert!(
            -0.007 < 0.0 && -0.016 + 0.018 * 0.5 > -shaft_r - 0.004,
            "the rest has to actually meet the shaft it supports"
        );
    }

    /// The draw HAND is on the string - the anchor restage itself.
    ///
    /// What this replaces was a y lift of 0.14·draw, which was correct for
    /// a VERTICAL bow (whose string spans Y, so any height is still on the
    /// cord) and wrong the moment the limbs turned sideways: the hand rode
    /// a palm's width above a horizontal string, pulling nothing. The
    /// hand's offset from the nock must therefore be CONSTANT - it cannot
    /// grow with the draw, because fingers do not drift off a string they
    /// are holding.
    #[test]
    fn the_draw_hand_holds_the_string_at_every_draw() {
        for d in DRAWS {
            let hand = bow_nock_local(d) + BOW_HAND_OFF;
            let off = hand - bow_nock_local(d);
            assert!(
                (off - BOW_HAND_OFF).length() < 1e-6,
                "draw {d}: the hand drifted to {off:?}"
            );
            // within a hand's reach of the cord, in the cord's own plane
            assert!(off.length() < 0.06, "draw {d}: {off:?} is not a grip");
            assert!(
                off.y.abs() < 1e-6,
                "draw {d}: the hand left the string's plane by {}",
                off.y
            );
        }
    }
}

/// §B.6 (Brief VIII-B): the 20-segment body's own completion gate.
///
/// The data half. Three of the brief's five tests need no new bone -
/// segment count, mass closure, and proportions - which is exactly why
/// the table lands before the rig surgery does.
#[cfg(test)]
mod segment_tests {
    use super::*;

    /// §B.6 segment-count test: all 20 named segments, each once.
    #[test]
    fn the_body_exposes_all_twenty_segments() {
        assert_eq!(SEGMENTS.len(), N_SEGMENTS);
        assert_eq!(N_SEGMENTS, 20);
        for s in SEGMENTS {
            assert_eq!(
                SEGMENTS.iter().filter(|q| **q == s).count(),
                1,
                "{s:?} is listed twice"
            );
        }
        // and the composition is the brief's: one head, three trunk, and
        // eight mirrored pairs
        let singles = SEGMENTS
            .iter()
            .filter(|s| {
                matches!(
                    s,
                    Segment::HeadNeck | Segment::Thorax | Segment::Lumbar | Segment::Pelvis
                )
            })
            .count();
        assert_eq!(singles, 4, "head plus a THREE-part trunk");
        assert_eq!((N_SEGMENTS - singles) % 2, 0, "everything else is a pair");
        assert_eq!((N_SEGMENTS - singles) / 2, 8, "eight mirrored pairs");
    }

    /// §B.6 mass-closure test: the whole body sums to 1.000 ± 0.001.
    ///
    /// This is the test that catches the trap in §B.3's own wording. The
    /// clavicle is "~0.005 (carve from thorax)" - CARVE, not add. Two
    /// clavicles bolted on beside a full-weight thorax put the body at
    /// 1.010, a 1% error that would never show up as anything but a
    /// vaguely wrong-feeling ragdoll.
    #[test]
    fn the_segment_masses_close_to_one_whole_body() {
        let total: f32 = SEGMENTS.into_iter().map(|s| segment_data(s).mass_frac).sum();
        assert!(
            (total - 1.0).abs() < 0.001,
            "body mass sums to {total}, not 1.000 - the clavicles are carved \
             FROM the thorax, not added beside it"
        );
        // the brief's own arithmetic, checked limb group by limb group
        let group = |f: fn(Segment) -> bool| -> f32 {
            SEGMENTS
                .into_iter()
                .filter(|s| f(*s))
                .map(|s| segment_data(s).mass_frac)
                .sum()
        };
        let trunk = group(|s| {
            matches!(
                s,
                Segment::Thorax
                    | Segment::Lumbar
                    | Segment::Pelvis
                    | Segment::ClavicleL
                    | Segment::ClavicleR
            )
        });
        assert!((trunk - 0.497).abs() < 0.001, "trunk is {trunk}, brief says 0.497");
        let one_arm = segment_data(Segment::UpperArmL).mass_frac
            + segment_data(Segment::ForearmL).mass_frac
            + segment_data(Segment::HandL).mass_frac;
        assert!((one_arm - 0.050).abs() < 0.001, "an arm is {one_arm}, brief says 0.050");
        let one_leg = segment_data(Segment::ThighL).mass_frac
            + segment_data(Segment::ShankL).mass_frac
            + segment_data(Segment::FootL).mass_frac
            + segment_data(Segment::ToeL).mass_frac;
        assert!((one_leg - 0.161).abs() < 0.001, "a leg is {one_leg}, brief says 0.161");
        // and no segment is weightless - a zero-mass segment would get a
        // zero-stiffness spring and hang limp
        for s in SEGMENTS {
            assert!(segment_data(s).mass_frac > 0.0, "{s:?} has no mass");
        }
    }

    /// A mirrored pair is the SAME segment on two sides - same mass,
    /// same length, same inertia.
    #[test]
    fn a_left_segment_weighs_what_its_right_twin_does() {
        for (l, r) in [
            (Segment::ClavicleL, Segment::ClavicleR),
            (Segment::UpperArmL, Segment::UpperArmR),
            (Segment::ForearmL, Segment::ForearmR),
            (Segment::HandL, Segment::HandR),
            (Segment::ThighL, Segment::ThighR),
            (Segment::ShankL, Segment::ShankR),
            (Segment::FootL, Segment::FootR),
            (Segment::ToeL, Segment::ToeR),
        ] {
            let (a, b) = (segment_data(l), segment_data(r));
            assert_eq!(a.name, b.name, "{l:?} and {r:?} are the same segment");
            assert_eq!(a.mass_frac, b.mass_frac);
            assert_eq!(a.len_frac, b.len_frac);
            assert_eq!(segment_inertia(l), segment_inertia(r));
        }
    }

    /// §B.6 proportion test: the published lengths, at the brief's own
    /// worked height.
    ///
    /// "At H = 1.8m: upper arm 33cm, forearm 26cm, thigh 44cm, shank
    /// 44cm, foot 27cm." Those five numbers are the brief checking its
    /// own table, so they are the right thing to check ours against - a
    /// fraction transcribed one digit wrong survives inspection and fails
    /// here.
    #[test]
    fn the_published_lengths_land_where_the_brief_says_they_do() {
        let cm = |s: Segment| segment_data(s).len_frac * RIG_HEIGHT_M * 100.0;
        for (s, want) in [
            (Segment::UpperArmL, 33.0_f32),
            (Segment::ForearmL, 26.0),
            (Segment::ThighL, 44.0),
            (Segment::ShankL, 44.0),
        ] {
            let got = cm(s);
            assert!(
                (got - want).abs() < 1.0,
                "{s:?} is {got:.1} cm at H={RIG_HEIGHT_M}, brief says {want}"
            );
        }
        // the foot is split hindfoot/toe, so the brief's 27 cm is the SUM
        let foot = cm(Segment::FootL) + cm(Segment::ToeL);
        assert!(
            (foot - 27.0).abs() < 1.0,
            "hindfoot + toe is {foot:.1} cm, brief says 27"
        );
        // §B.4's other three proportions are stated, not derived
        assert!((SHOULDER_WIDTH_FRAC * RIG_HEIGHT_M - 0.466).abs() < 0.01);
        assert!(SHOULDER_WIDTH_FRAC > HIP_WIDTH_FRAC, "shoulders are wider than hips");
        assert!(
            SHOULDER_HEIGHT_FRAC < 1.0,
            "the shoulders are below the top of the head"
        );
    }

    /// §owner MECH BARRIER: the two halves of "transparent to me, a wall
    /// of light to you" are actually opposed, and the numbers have to
    /// keep them that way.
    ///
    /// A single translucent sheet cannot satisfy both readings, which is
    /// why the field is a near-invisible FILL plus a bright LATTICE. If
    /// the fill ever creeps up toward the lattice's opacity the pilot
    /// starts fighting through frosted glass, and if the lattice ever
    /// dims toward the fill the enemy stops seeing a shield at all.
    #[test]
    fn the_barrier_is_a_window_to_the_pilot_and_a_wall_to_the_enemy() {
        // the fill has to be nearly clear - this is the number the
        // pilot's visibility depends on
        const FILL_A: f32 = 0.085;
        const EDGE_A: f32 = 0.60;
        assert!(FILL_A < 0.12, "a pilot cannot fight through {FILL_A} alpha");
        assert!(
            EDGE_A > FILL_A * 5.0,
            "the lattice must dominate the fill by a wide margin, or the \
             barrier reads as a smudge from both sides"
        );
        // and the deploy has to be combat-fast. A barrier you have to
        // raise half a second early is one you die behind.
        assert!(
            BARRIER_DEPLOY_S <= 0.25,
            "{BARRIER_DEPLOY_S}s is too slow to be a reaction"
        );
        assert!(BARRIER_DEPLOY_S > 0.0, "an instant deploy has no read at all");
        // the petals must actually open far enough to frame a field
        assert!(
            (30.0..90.0).contains(&BARRIER_PETAL_DEG),
            "{BARRIER_PETAL_DEG} deg does not read as a fold"
        );
    }

    /// §B.6 toe-off test: "assert the toe segment rotates through its
    /// plantar-flexion range at contact-exit - no toe rotation means the
    /// run is still a glide."
    ///
    /// It WAS a glide. There was nothing forward of the ankle, so the
    /// foot left the ground as a flat plate and the whole sprint pushed
    /// off nothing. This is the test that proves segments #19-20 landed.
    #[test]
    fn the_sprint_actually_pushes_off_its_toes() {
        // sweep a full cycle at sprint amplitude
        let mut peak = 0.0_f32;
        let mut peak_at = 0.0_f32;
        let n = 720;
        for i in 0..n {
            let ph = i as f32 / n as f32 * std::f32::consts::TAU;
            let a = toe_off_angle(ph, 1.0);
            assert!(a >= 0.0, "a toe pushes, it never pulls: {a} at {ph}");
            assert!(a <= TOE_OFF_MAX + 1e-6, "hyperextended to {a}");
            if a > peak {
                peak = a;
                peak_at = ph;
            }
        }
        assert!(
            (peak - TOE_OFF_MAX).abs() < 1e-3,
            "the toe must reach its full range somewhere in the cycle, got \
             {:.1} of {:.1} degrees",
            peak.to_degrees(),
            TOE_OFF_MAX.to_degrees()
        );
        // ...and reach it at CONTACT EXIT - the back of the stance, a
        // quarter-cycle after the leg's rearmost point, not at mid-swing
        assert!(
            (peak_at - PI).abs() < 0.2,
            "toe-off peaks at phase {peak_at:.2}, expected the back of \
             stance near {PI:.2}"
        );
        // a STANDING fighter has flat feet. A toe that stayed cocked at a
        // standstill would be the no-bounce contract broken in a new place
        for i in 0..64 {
            let ph = i as f32 * 0.31;
            assert_eq!(toe_off_angle(ph, 0.0), 0.0, "a standing toe must be flat");
        }
        // and a WALK gets a roll where a sprint gets a snap
        let walk = (0..n)
            .map(|i| toe_off_angle(i as f32 / n as f32 * std::f32::consts::TAU, 0.25))
            .fold(0.0_f32, f32::max);
        assert!(
            walk < peak * 0.5,
            "a walk must not toe off like a sprint: {walk} vs {peak}"
        );
    }

    /// §B.2: inserting the lumbar must not MOVE anything.
    ///
    /// This test exists because its absence cost a broken build. The
    /// first trunk split left `WAIST_Y` on the thorax while its new
    /// parent, the lumbar, also carried it - so the upper body sat a full
    /// waist above the legs and the soldier came apart in mid-air. Every
    /// rig test in this file passed: they measure ANGLES (separation, the
    /// kinetic chain, the trunk twist above) or the head BAND, which is
    /// derived from `gait_pose` rather than read back off the transform
    /// hierarchy. Nothing was watching where the torso actually WAS.
    ///
    /// The claim is composition: lumbar + thorax must land exactly where
    /// the single trunk segment used to.
    #[test]
    fn thorax_height_is_conserved_across_the_trunk_split() {
        for hip_y in [0.50_f32, 0.63, 0.71] {
            for crouch in [0.0_f32, 0.12] {
                for breath in [-0.004_f32, 0.0, 0.004] {
                    // what the SINGLE trunk segment used to be set to, in
                    // root space - the expression this replaced, verbatim
                    let before = hip_y - crouch + breath;
                    // and what the two segments now compose to
                    let after = WAIST_Y + thorax_local_y(hip_y, crouch, breath);
                    assert!(
                        (after - before).abs() < 1e-6,
                        "the trunk moved: {before} -> {after} \
                         (hip {hip_y} crouch {crouch} breath {breath})"
                    );
                }
            }
        }
        // and the waist really is where the legs are hung from, or the
        // subtraction is against the wrong number
        assert!(
            (WAIST_Y - 0.63).abs() < 1e-6,
            "WAIST_Y must match the height the thighs spawn at"
        );
        // a standing fighter's thorax sits AT the waist - local zero
        assert!(
            thorax_local_y(WAIST_Y, 0.0, 0.0).abs() < 1e-6,
            "an unposed thorax must sit exactly on its own parent"
        );
    }

    /// §B.2: the trunk twist is SHARED between lumbar and thorax, and the
    /// two still sum to exactly what one joint used to carry.
    ///
    /// The sum is the load-bearing half. Hip-shoulder separation and the
    /// §6.2 ±60° additive-aim contract are both stated against the TOTAL
    /// trunk yaw, so splitting it must not change that total - only where
    /// along the spine it happens.
    #[test]
    fn the_trunk_twist_is_shared_but_conserved() {
        assert!(
            LUMBAR_TWIST_SHARE > 0.0,
            "a lumbar with no share is the hinge this replaced"
        );
        assert!(
            LUMBAR_TWIST_SHARE < 0.5,
            "the thoracic spine out-rotates the lumbar - an even split \
             reads as a body hinged at the belt"
        );
        for yaw in [-1.2_f32, -0.4, 0.0, 0.3, 0.9] {
            let lumbar = yaw * LUMBAR_TWIST_SHARE;
            let thorax = yaw * (1.0 - LUMBAR_TWIST_SHARE);
            assert!(
                (lumbar + thorax - yaw).abs() < 1e-6,
                "the split changed the total at {yaw}"
            );
            // and they twist the SAME way - opposed shares would be a
            // counter-rotation nobody asked for
            if yaw != 0.0 {
                assert_eq!(lumbar.signum(), thorax.signum());
            }
        }
    }

    /// §B.5: stiffness comes OUT of the mass model, and it orders the
    /// segments the way physical intuition does.
    ///
    /// The claim being made is not that any particular number is right -
    /// it is that the numbers are no longer independent. A thigh is
    /// heavier and longer than a forearm, so at one shared frequency it
    /// must come back stiffer, and nobody has to decide that by feel.
    #[test]
    fn spring_stiffness_is_derived_from_mass_not_guessed() {
        const W: f32 = 14.0; // the ω the viewmodel sway already runs at
        let k = |s: Segment| derived_spring_k(s, W);
        assert!(
            k(Segment::ThighL) > k(Segment::ForearmL),
            "a thigh must be stiffer to drive than a forearm: {} vs {}",
            k(Segment::ThighL),
            k(Segment::ForearmL)
        );
        assert!(
            k(Segment::UpperArmL) > k(Segment::HandL),
            "an upper arm must be stiffer than a hand"
        );
        assert!(
            k(Segment::ShankL) > k(Segment::ToeL),
            "a shank must be stiffer than a toe"
        );
        // it scales as ω², the standard relation - so the ONE knob left
        // is a frequency, which is a thing a person can reason about
        let a = derived_spring_k(Segment::ThighL, W);
        let b = derived_spring_k(Segment::ThighL, W * 2.0);
        assert!((b / a - 4.0).abs() < 1e-3, "k must go as omega squared");
        // and every segment with a real length gets a real stiffness -
        // a zero would be a limb that never comes back
        for s in SEGMENTS {
            if segment_data(s).len_frac > 0.0 {
                assert!(k(s) > 0.0, "{s:?} has no stiffness");
            }
        }
    }
}

/// R4 - config externalization's completion gate (camera-tuning slice).
#[cfg(test)]
mod capture_path_tests {
    use super::*;

    /// A capture that writes its frames somewhere nobody looks is worse
    /// than one that fails: it reports success. This is the regression
    /// guard for exactly that - two frames of the menus capture were
    /// silently written outside the tracked tree because the path was
    /// relative to the working directory.
    #[test]
    fn capture_frames_land_in_the_tracked_tree_not_the_working_directory() {
        let dir = capture_dir("menus");
        assert!(
            std::path::Path::new(&dir).is_absolute(),
            "capture dir must be absolute so the launch directory cannot move it, got {dir:?}"
        );
        assert!(
            dir.ends_with("/handback/brief-vii/menus"),
            "must land in the handback tree, got {dir:?}"
        );
        assert!(
            dir.contains("jk_tdm"),
            "must be anchored inside this crate, got {dir:?}"
        );
        assert!(
            !dir.contains('\\'),
            "separators must be normalised - a mixed path breaks the ends_with checks \
             callers and tests do, got {dir:?}"
        );
        // the crate root it anchors to must actually be the crate root
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(
            root.join("Cargo.toml").exists(),
            "CARGO_MANIFEST_DIR must point at this crate's root"
        );
        // and every script name gets its own directory, never a shared one
        assert_ne!(capture_dir("menus"), capture_dir("baseline"));
    }
}

#[cfg(test)]
mod lowready_tests {
    use super::*;

    /// Drive the spring the way the frame loop does: 60 fps, fixed step.
    fn run(target: f32, frames: usize, from: (f32, f32)) -> Vec<f32> {
        let (mut x, mut v) = from;
        (0..frames)
            .map(|_| {
                ready_up_step(&mut x, &mut v, target, 1.0 / 60.0);
                x
            })
            .collect()
    }

    /// §3.4: "returns over 0.15s with ONE SMALL OVERSHOOT (ζ ≈ 0.7)".
    ///
    /// The overshoot is the whole point of the spec line - a lerp would
    /// satisfy "returns over 0.15s" and silently drop the character of
    /// the motion. So assert the overshoot EXISTS, that it is small, and
    /// that there is only one.
    #[test]
    fn ready_up_overshoots_once_and_settles_on_the_brief_s_clock() {
        // hold at full low-ready, then clear the wall
        let xs = run(0.0, 60, (1.0, 0.0));

        let min = xs.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(
            min < -0.001,
            "ζ=0.7 must overshoot PAST zero - got a minimum of {min}, \
             which is a lerp wearing a spring's name"
        );
        assert!(
            min > -0.15,
            "the overshoot must be SMALL (brief: 'one small overshoot'), got {min}"
        );

        // Exactly one overshoot the eye can SEE. A ζ=0.7 step response
        // rings analytically at 4.6%, then 0.2%, then 0.01% - counting
        // raw zero-crossings would count that decay tail as a wobble and
        // fail a spring that is behaving exactly as specified. So count
        // excursions that clear 1% of the travel, which is the threshold
        // below which nothing is visible on a 22 deg rotation (0.2 deg).
        let visible = {
            let mut n = 0;
            let mut below = false;
            for &x in &xs {
                if x < -0.01 && !below {
                    n += 1;
                    below = true;
                } else if x >= 0.0 {
                    below = false;
                }
            }
            n
        };
        assert_eq!(visible, 1, "one VISIBLE overshoot, not a wobble: {visible}");

        // settled inside the 2% band by the spec's 0.15 s (9 frames at
        // 60 fps), which is what READY_UP_OMEGA was derived to deliver
        let at_spec = xs[(0.15 * 60.0) as usize];
        assert!(
            at_spec.abs() < 0.02,
            "must be within the 2% band at 0.15s, got {at_spec}"
        );
    }

    /// The dip must actually ARRIVE when a wall is close - a spring that
    /// never reaches its target is a slow bug, not a stance.
    #[test]
    fn low_ready_reaches_full_dip_and_the_angles_match_the_brief() {
        let xs = run(1.0, 60, (0.0, 0.0));
        let settled = *xs.last().unwrap();
        assert!(
            (settled - 1.0).abs() < 0.02,
            "the dip must arrive at full, got {settled}"
        );
        // the brief's numbers, not a re-typed approximation of them
        assert!(
            (LOWREADY_PITCH.to_degrees() - 22.0).abs() < 1e-3,
            "§3.4 specifies 22 degrees"
        );
        assert!(
            (LOWREADY_RANGE_M - 0.6).abs() < 1e-6,
            "§3.4 specifies 0.6 m"
        );
        assert!(
            LOWREADY_YAW > 0.0 && LOWREADY_PITCH > 0.0,
            "'up-and-in' is two rotations, not one"
        );
    }

    /// The spring must not explode on a slow frame. An unstable spring
    /// here throws the weapon off screen rather than degrading, so the
    /// sub-stepping is load-bearing and gets its own proof.
    #[test]
    fn the_spring_stays_bounded_at_terrible_framerates() {
        for fps in [10.0_f32, 15.0, 20.0, 30.0, 144.0] {
            let (mut x, mut v) = (1.0_f32, 0.0_f32);
            for _ in 0..200 {
                ready_up_step(&mut x, &mut v, 0.0, 1.0 / fps);
                assert!(
                    x.is_finite() && x.abs() < 2.0,
                    "spring diverged at {fps} fps: x={x}"
                );
            }
            assert!(
                x.abs() < 0.02,
                "must still settle at {fps} fps, ended at {x}"
            );
        }
    }
}

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
            ForgeProfile { hat: 0, tunic: 0, melee_axe: false, grenade_preset: 0, helmet: 0, armor: 0 },
            ForgeProfile { hat: 3, tunic: 2, melee_axe: true, grenade_preset: 3, helmet: 4, armor: 0x00FF_FFFF },
            ForgeProfile { hat: 1, tunic: 3, melee_axe: false, grenade_preset: 2, helmet: 2, armor: 0x0A5A },
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

    /// §8.1: slot files written before the helmet field existed must
    /// still load. This is the actual upgrade path - anyone who used the
    /// Forge before this build has four-field files sitting on disk, and
    /// the failure mode if `from_line` were strict is that their saved
    /// profiles vanish on first launch without a word.
    #[test]
    fn a_pre_helmet_save_file_still_loads_as_the_field_cap() {
        let old = ForgeProfile::from_line("3,2,1,3")
            .expect("the four-field format that shipped must still parse");
        assert_eq!(old.hat, 3);
        assert_eq!(old.tunic, 2);
        assert!(old.melee_axe);
        assert_eq!(old.grenade_preset, 3);
        assert_eq!(
            old.helmet, 0,
            "a file with no helmet must read as the FIELD CAP - the shape              it was actually wearing when it was written"
        );
        // and a malformed FIFTH field is still an error, not a silent 0:
        // absent and wrong are different, and only absent is forgiven
        assert!(ForgeProfile::from_line("3,2,1,3,x").is_none());
    }

    /// §8.1: index 0 must still be the exact brim-crown-band the body had
    /// before the library existed. Anyone whose saved profile predates the
    /// helmet field loads as 0 (see above), so if these values drifted,
    /// those profiles would quietly change shape.
    #[test]
    fn helmet_zero_is_the_frozen_field_cap() {
        let (name, pieces) = HELMET_CHOICES[0];
        assert_eq!(name, "FIELD CAP");
        assert_eq!(pieces.len(), 3, "brim, crown, band");
        assert_eq!(pieces[0].pos, (0.0, 1.02, 0.0));
        assert_eq!(pieces[0].scale, (0.72, 0.028, 0.72));
        assert_eq!(pieces[1].pos, (0.0, 1.11, 0.0));
        assert_eq!(pieces[1].scale, (0.36, 0.18, 0.36));
        assert_eq!(pieces[2].pos, (0.0, 1.045, 0.0));
        assert_eq!(pieces[2].scale, (0.365, 0.04, 0.365));
        // the antenna is shared, and its two pieces are equally frozen
        assert_eq!(HELMET_ANTENNA[0].pos, (0.13, 1.22, 0.0));
        assert_eq!(HELMET_ANTENNA[1].pos, (0.13, 1.30, 0.0));
        assert_eq!(HELMET_ANTENNA[1].scale, (0.035, 0.035, 0.035));
    }

    /// §8.1: the array type on `ForgePreview::helmets` is `N_HELMETS`, not
    /// `HELMET_CHOICES.len()` (rustc crashes on the latter here - see the
    /// const's doc). That decoupling is exactly the kind that rots, so it
    /// is pinned.
    #[test]
    fn helmet_library_is_the_declared_size() {
        assert_eq!(HELMET_CHOICES.len(), N_HELMETS);
    }

    /// §8.1: every piece of every helmet sits inside the socket envelope.
    ///
    /// This is what makes the library safe to extend. A new entry gets
    /// checked for the three things hand-placed geometry actually gets
    /// wrong - sunk into the head, floating above it, or poking out wide
    /// enough to show through cover - without anyone opening the game.
    ///
    /// A leaning piece is measured by its ROTATED extent, not its resting
    /// one: a tilted box reaches further than its scale suggests, and
    /// checking the unrotated box would pass geometry that visibly clips.
    #[test]
    fn helmet_pieces_stay_in_the_socket_envelope() {
        for (name, pieces) in HELMET_CHOICES {
            assert!(!pieces.is_empty(), "{name} has no geometry");
            for (i, p) in pieces.iter().chain(HELMET_ANTENNA.iter()).enumerate() {
                let (hx, hy, hz) = (p.scale.0 * 0.5, p.scale.1 * 0.5, p.scale.2 * 0.5);
                // rotated half-extents: exact for a box, conservative for
                // the cylinder and sphere, which are inscribed in one
                let (cp, sp) = (p.pitch.cos().abs(), p.pitch.sin().abs());
                let (cr, sr) = (p.roll.cos().abs(), p.roll.sin().abs());
                let ry = hy * cp * cr + hz * sp + hx * sr;
                let rx = hx * cr + hy * sr;
                let rz = hz * cp + hy * sp;
                let (lo, hi) = (p.pos.1 - ry, p.pos.1 + ry);
                assert!(
                    lo >= HELMET_Y_MIN,
                    "{name} piece {i} sinks into the head shell: {lo} < {HELMET_Y_MIN}"
                );
                assert!(
                    hi <= HELMET_Y_MAX,
                    "{name} piece {i} floats above the fighter: {hi} > {HELMET_Y_MAX}"
                );
                let reach = (p.pos.0.abs() + rx).max(p.pos.2.abs() + rz);
                assert!(
                    reach <= HELMET_XZ_MAX,
                    "{name} piece {i} reaches {reach} wide, past {HELMET_XZ_MAX} -                      it would show through cover the player thinks hides them"
                );
            }
        }
    }

    /// §8.1: the five must actually look different. A library whose
    /// entries share a silhouette is four wasted menu rows - and the
    /// stated reason for the feature was that TINT washes out at range
    /// and SHAPE does not.
    ///
    /// Measured as a SAMPLED OUTLINE - how wide the helmet is at each of
    /// sixteen heights, which is what a distant player's eye integrates -
    /// rather than as a bounding box. The difference is not academic: the
    /// first version of this test compared bounding boxes and called VISOR
    /// and CREST identical, when one is a brow-and-cheeks helm and the
    /// other is a bare dome under a tall blade. They occupy a similar box
    /// while looking nothing alike, and a test that measures the box would
    /// have had me distort real geometry to satisfy a bad proxy.
    ///
    /// The shared antenna is excluded: every helmet has it, so it adds the
    /// same value to every profile and can only mask a real difference.
    #[test]
    fn the_five_helmets_have_distinct_silhouettes() {
        const BANDS: usize = 16;
        let outline = |pieces: &[HelmetPiece]| {
            let mut w = [0.0_f32; BANDS];
            for (b, slot) in w.iter_mut().enumerate() {
                let y = HELMET_Y_MIN
                    + (HELMET_Y_MAX - HELMET_Y_MIN) * (b as f32 + 0.5) / BANDS as f32;
                for p in pieces {
                    let hy = p.scale.1 * 0.5;
                    if (p.pos.1 - hy..=p.pos.1 + hy).contains(&y) {
                        *slot = slot.max(p.pos.0.abs() + p.scale.0 * 0.5);
                    }
                }
            }
            w
        };
        let all: Vec<_> = HELMET_CHOICES.iter().map(|(n, p)| (*n, outline(p))).collect();
        for (i, (na, a)) in all.iter().enumerate() {
            for (nb, b) in &all[i + 1..] {
                let d: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
                assert!(
                    d > 0.30,
                    "{na} and {nb} have near-identical outlines ({d}) - at range                      a player could not tell them apart"
                );
            }
        }
    }

    #[test]
    fn save_then_load_round_trips_through_the_real_filesystem() {
        let slot = 99; // a slot no real save will ever use
        let p = ForgeProfile { hat: 2, tunic: 1, melee_axe: true, grenade_preset: 1, helmet: 3, armor: 0x00F0_F0F0 };
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
        // §4.6 (Brief VIII): the crosshair family rides the same file.
        // Every field is set AWAY from its default, so a field the
        // serializer forgot cannot pass by accidentally matching.
        s.cross_size = 9;
        s.cross_gap = -3; // negative is legal (§4.6) and must survive
        s.cross_thickness = 4;
        s.cross_dot = true;
        s.cross_outline = false;
        s.cross_outline_px = 3;
        s.cross_color_idx = CROSS_COLOR_CUSTOM_IDX;
        s.cross_rgb = (17, 200, 240);
        s.cross_alpha = 137;
        s.cross_t_shape = true;
        s.cross_dynamic = true;
        // §4.1/§4.3: the HUD readout options ride the same file, same
        // rule - every one set AWAY from its default.
        s.hud_vitals_style = 1;
        s.minimap_rotate = false;
        s.minimap_scale = 45;
        let back = parse_settings(&settings_to_text(&s));
        assert_eq!(back.swap_mouse, s.swap_mouse);
        assert_eq!(back.minimap, s.minimap);
        assert_eq!(back.sens_idx, s.sens_idx);
        assert_eq!(back.fov_idx, s.fov_idx);
        assert_eq!(back.invert_y, s.invert_y);
        assert_eq!(back.cross_size, s.cross_size, "crosshair size must persist");
        assert_eq!(back.cross_gap, s.cross_gap, "a NEGATIVE gap must persist as-is");
        assert_eq!(back.cross_thickness, s.cross_thickness);
        assert_eq!(back.cross_dot, s.cross_dot);
        assert_eq!(back.cross_outline, s.cross_outline);
        assert_eq!(back.cross_outline_px, s.cross_outline_px);
        assert_eq!(back.cross_color_idx, s.cross_color_idx);
        assert_eq!(back.cross_rgb, s.cross_rgb, "custom RGB must persist per channel");
        assert_eq!(back.cross_alpha, s.cross_alpha);
        assert_eq!(back.cross_t_shape, s.cross_t_shape);
        assert_eq!(back.cross_dynamic, s.cross_dynamic);
        assert_eq!(back.hud_vitals_style, s.hud_vitals_style, "§4.1 vitals style must persist");
        assert_eq!(back.minimap_rotate, s.minimap_rotate, "§4.3 rotate must persist");
        assert_eq!(back.minimap_scale, s.minimap_scale, "§4.3 scale must persist");

        // §4.1/§4.3 hostile: the same clamp rule as everything above.
        let evil_hud = "hud_vitals_style = 88\nminimap_scale = 9999\n";
        let h = parse_settings(evil_hud);
        assert!(h.hud_vitals_style <= 1, "vitals style clamps to a real mode");
        assert_eq!(
            h.minimap_scale, MINIMAP_SCALE_RANGE.1,
            "an oversize minimap clamps to the brief's 1.0 ceiling"
        );
        let tiny = parse_settings("minimap_scale = -400\n");
        assert_eq!(
            tiny.minimap_scale, MINIMAP_SCALE_RANGE.0,
            "a negative minimap clamps to the brief's 0.25 floor, never to nothing"
        );

        // hostile: out-of-range indices clamp instead of panicking later
        let evil = "sens_idx = 999\nfov_idx = -5\nswap_mouse = 7\n";
        let p = parse_settings(evil);
        assert_eq!(p.sens_idx, SENS_CHOICES.len() - 1, "oversize index clamps to last");
        assert_eq!(p.fov_idx, 0, "negative index clamps to first");
        assert!(p.swap_mouse, "any nonzero reads as true");
        // and the clamped values actually index safely
        let _ = p.sens_mult();
        let _ = p.fov_deg();

        // §4.6 hostile: every crosshair number clamps into a DRAWABLE
        // range. A zero-size or negative-thickness crosshair is not a
        // preference, it is an invisible or inverted one.
        let evil_cross = "cross_size = 9999\ncross_gap = -9999\ncross_thickness = 0\n\
                          cross_outline_px = 77\ncross_color_idx = 4000\n\
                          cross_r = 900\ncross_g = -12\ncross_alpha = 4096\n";
        let c = parse_settings(evil_cross);
        assert_eq!(c.cross_size, CROSS_SIZE_RANGE.1, "oversize size clamps to max");
        assert_eq!(c.cross_gap, CROSS_GAP_RANGE.0, "gap clamps to its NEGATIVE floor");
        assert_eq!(c.cross_thickness, CROSS_THICK_RANGE.0, "thickness never below 1");
        assert_eq!(c.cross_outline_px, CROSS_OUTLINE_RANGE.1);
        assert_eq!(
            c.cross_color_idx,
            CROSS_COLOR_CHOICES.len() - 1,
            "an out-of-range colour index must not index off the preset table"
        );
        assert_eq!(c.cross_rgb.0, 255, "channel clamps to 255");
        assert_eq!(c.cross_rgb.1, 0, "a negative channel clamps to 0");
        assert_eq!(c.cross_alpha, 255);
        // the clamped values actually produce drawable geometry
        let _ = crosshair_rgb(&c);
        for r in crosshair_arm_rects(c.cross_size as f32, c.cross_gap as f32, c.cross_thickness as f32) {
            assert!(r.w > 0.0 && r.h > 0.0, "clamped settings must still draw: {r:?}");
        }

        // garbage lines are ignored, defaults survive
        let junk = "!!!\nsens_idx = banana\n= 3\nfov_idx\ncross_size = wide\n\
                    cross_gap = \ncross_alpha = 3.5\n";
        let j = parse_settings(junk);
        assert_eq!(j.sens_idx, GameSettings::default().sens_idx);
        assert_eq!(j.fov_idx, GameSettings::default().fov_idx);
        assert_eq!(j.cross_size, CROSS_SIZE_DEFAULT, "non-numeric size keeps the default");
        assert_eq!(j.cross_gap, CROSS_GAP_DEFAULT);
        assert_eq!(j.cross_alpha, CROSS_ALPHA_DEFAULT, "'3.5' is not an integer");
    }

    /// **§4.9, the test that could not previously be written.**
    /// "Crosshair settings round-trip through the settings file" - and
    /// this one goes through an actual FILE, not just the two pure
    /// functions, so a serializer that emits something `parse_settings`
    /// reads but a disk write mangles (line endings, the comment line,
    /// a trailing newline) is caught too.
    #[test]
    fn crosshair_settings_round_trip_through_the_settings_file() {
        let mut s = GameSettings::default();
        s.cross_size = CROSS_SIZE_RANGE.1;
        s.cross_gap = CROSS_GAP_RANGE.0; // the negative extreme
        s.cross_thickness = CROSS_THICK_RANGE.1;
        s.cross_dot = true;
        s.cross_outline = true;
        s.cross_outline_px = 2;
        s.cross_color_idx = CROSS_COLOR_CUSTOM_IDX;
        s.cross_rgb = (1, 2, 254);
        s.cross_alpha = 200;
        s.cross_t_shape = true;
        s.cross_dynamic = true;

        let path = std::env::temp_dir().join(format!(
            "jk_tdm_crosshair_settings_{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, settings_to_text(&s)).expect("write the settings file");
        let text = std::fs::read_to_string(&path).expect("read it back");
        let _ = std::fs::remove_file(&path);

        let back = parse_settings(&text);
        assert_eq!(back.cross_size, s.cross_size);
        assert_eq!(back.cross_gap, s.cross_gap);
        assert_eq!(back.cross_thickness, s.cross_thickness);
        assert_eq!(back.cross_dot, s.cross_dot);
        assert_eq!(back.cross_outline, s.cross_outline);
        assert_eq!(back.cross_outline_px, s.cross_outline_px);
        assert_eq!(back.cross_color_idx, s.cross_color_idx);
        assert_eq!(back.cross_rgb, s.cross_rgb);
        assert_eq!(back.cross_alpha, s.cross_alpha);
        assert_eq!(back.cross_t_shape, s.cross_t_shape);
        assert_eq!(back.cross_dynamic, s.cross_dynamic);
        // and the colour the renderer would use is the one we saved
        assert_eq!(crosshair_rgb(&back), (1, 2, 254));

        // a DEFAULT settings file round-trips to the spec's own defaults:
        // size 5, gap 0, thickness 1, dot off, outline on 1, green
        // 50/250/50, alpha 200, no T-shape, classic STATIC.
        let d = parse_settings(&settings_to_text(&GameSettings::default()));
        assert_eq!(d.cross_size, 5);
        assert_eq!(d.cross_gap, 0);
        assert_eq!(d.cross_thickness, 1);
        assert!(!d.cross_dot, "the dot is OFF by default");
        assert!(d.cross_outline && d.cross_outline_px == 1);
        assert_eq!(crosshair_rgb(&d), (50, 250, 50), "spec default is green 50,250,50");
        assert_eq!(d.cross_alpha, 200);
        assert!(!d.cross_t_shape);
        assert!(!d.cross_dynamic, "the default is classic STATIC");
    }

    /// The drawn geometry must actually MOVE with the settings - the
    /// whole point of replacing a `+` glyph. Every assertion here is
    /// derived by hand from what "size / gap / thickness" mean, not read
    /// back out of the function.
    #[test]
    fn crosshair_geometry_responds_to_size_gap_and_thickness() {
        let a = crosshair_arm_rects(5.0, 2.0, 1.0);
        let top = a[CROSS_ARM_TOP];
        let right = a[CROSS_ARM_RIGHT];
        let bottom = a[CROSS_ARM_BOTTOM];
        let left = a[CROSS_ARM_LEFT];

        // each arm is `size` long along its own axis and `thickness` across
        assert_eq!((top.w, top.h), (1.0, 5.0));
        assert_eq!((bottom.w, bottom.h), (1.0, 5.0));
        assert_eq!((right.w, right.h), (5.0, 1.0));
        assert_eq!((left.w, left.h), (5.0, 1.0));
        // inner edges sit exactly `gap` from the centre, all four ways
        assert_eq!(right.left, 2.0, "right arm starts at the gap");
        assert_eq!(bottom.top, 2.0, "bottom arm starts at the gap");
        assert_eq!(left.left + left.w, -2.0, "left arm ends at -gap");
        assert_eq!(top.top + top.h, -2.0, "top arm ends at -gap");
        // and each arm is centred on its own axis
        assert_eq!(top.left, -0.5);
        assert_eq!(right.top, -0.5);

        // SIZE lengthens the arms without moving their inner edges
        let bigger = crosshair_arm_rects(9.0, 2.0, 1.0);
        assert_eq!(bigger[CROSS_ARM_RIGHT].w, 9.0, "size is the arm length");
        assert_eq!(
            bigger[CROSS_ARM_RIGHT].left, right.left,
            "size must not move the inner edge - that is the gap's job"
        );
        assert!(
            bigger[CROSS_ARM_TOP].top < top.top,
            "a longer top arm must reach FURTHER up"
        );

        // GAP moves the inner edges outward without changing the length
        let wider = crosshair_arm_rects(5.0, 6.0, 1.0);
        assert_eq!(wider[CROSS_ARM_RIGHT].left, 6.0);
        assert_eq!(wider[CROSS_ARM_RIGHT].w, right.w, "gap must not resize an arm");
        assert!(
            wider[CROSS_ARM_LEFT].left < left.left,
            "a bigger gap pushes the left arm further left"
        );

        // THICKNESS widens the cross-axis and keeps the arm centred
        let fat = crosshair_arm_rects(5.0, 2.0, 4.0);
        assert_eq!(fat[CROSS_ARM_TOP].w, 4.0, "thickness is the arm's width");
        assert_eq!(fat[CROSS_ARM_TOP].h, 5.0, "thickness must not change the length");
        assert_eq!(fat[CROSS_ARM_TOP].left, -2.0, "still centred on the vertical axis");
        assert_eq!(fat[CROSS_ARM_RIGHT].top, -2.0, "still centred on the horizontal axis");
        // the dot follows the thickness, centred
        let dot = crosshair_dot_rect(4.0);
        assert_eq!((dot.w, dot.h), (4.0, 4.0));
        assert_eq!((dot.left, dot.top), (-2.0, -2.0));

        // opposite arms are exact mirrors of each other
        assert_eq!(top.left, bottom.left);
        assert_eq!(top.top + top.h, -(bottom.top), "top/bottom mirror through 0");
        assert_eq!(left.top, right.top);
        assert_eq!(left.left + left.w, -(right.left), "left/right mirror through 0");
    }

    /// §4.6 says gap may go NEGATIVE. That is the case most likely to
    /// produce a zero-size or inverted rect, so it gets its own test: at
    /// every legal gap, all four arms stay positively-sized and full
    /// length, and a negative gap really does cross the centre.
    #[test]
    fn crosshair_negative_gap_is_legal_and_never_inverts_a_rect() {
        for gap in CROSS_GAP_RANGE.0..=CROSS_GAP_RANGE.1 {
            for size in CROSS_SIZE_RANGE.0..=CROSS_SIZE_RANGE.1 {
                for thick in CROSS_THICK_RANGE.0..=CROSS_THICK_RANGE.1 {
                    let arms =
                        crosshair_arm_rects(size as f32, gap as f32, thick as f32);
                    for (i, r) in arms.iter().enumerate() {
                        assert!(
                            r.w > 0.0 && r.h > 0.0,
                            "arm {i} collapsed at size={size} gap={gap} thick={thick}: {r:?}"
                        );
                        // vertical arms run along h, horizontal along w -
                        // asserted per-axis, not by max/min, because at
                        // size 1 / thickness 5 an arm is legitimately
                        // wider than it is long
                        let vertical = i == CROSS_ARM_TOP || i == CROSS_ARM_BOTTOM;
                        let (along, across) = if vertical { (r.h, r.w) } else { (r.w, r.h) };
                        assert_eq!(
                            along, size as f32,
                            "arm {i} lost length at size={size} gap={gap} thick={thick}"
                        );
                        assert_eq!(
                            across, thick as f32,
                            "arm {i} lost thickness at size={size} gap={gap} thick={thick}"
                        );
                    }
                }
            }
        }
        // a negative gap must genuinely cross the centre, not clamp to 0
        let crossed = crosshair_arm_rects(5.0, -3.0, 1.0);
        assert!(
            crossed[CROSS_ARM_TOP].top + crossed[CROSS_ARM_TOP].h > 0.0,
            "at gap -3 the top arm's lower edge must sit BELOW the centre"
        );
        assert!(
            crossed[CROSS_ARM_RIGHT].left < 0.0,
            "at gap -3 the right arm must start LEFT of the centre"
        );
        // and the outline only ever grows the rect it backs
        let base = crossed[CROSS_ARM_RIGHT];
        let o = crosshair_outline_rect(base, 2.0);
        assert_eq!((o.w, o.h), (base.w + 4.0, base.h + 4.0));
        assert_eq!((o.left, o.top), (base.left - 2.0, base.top - 2.0));
        let none = crosshair_outline_rect(base, -5.0);
        assert_eq!(none, base, "a negative outline width can never SHRINK the fill");
    }

    /// The three remaining §4.6 switches: T-shape drops exactly one arm,
    /// static ignores the aim cone while dynamic blooms with it, and the
    /// colour comes from the preset table or the custom triple.
    #[test]
    fn crosshair_t_shape_static_dynamic_and_colour_presets() {
        // T-shape hides the TOP arm and nothing else
        for arm in 0..4 {
            assert!(crosshair_arm_shown(arm, false), "arm {arm} shows without T-shape");
        }
        assert!(!crosshair_arm_shown(CROSS_ARM_TOP, true), "T-shape drops the top arm");
        for arm in [CROSS_ARM_RIGHT, CROSS_ARM_BOTTOM, CROSS_ARM_LEFT] {
            assert!(crosshair_arm_shown(arm, true), "T-shape must keep arm {arm}");
        }

        // classic STATIC ignores spread entirely
        assert_eq!(crosshair_gap_px(2, 0.0, false), 2.0);
        assert_eq!(
            crosshair_gap_px(2, 0.05, false),
            2.0,
            "a static crosshair must not move with the aim cone"
        );
        // DYNAMIC blooms with it, monotonically, from the same base
        assert_eq!(crosshair_gap_px(2, 0.0, true), 2.0, "no spread, no bloom");
        let a = crosshair_gap_px(2, 0.01, true);
        let b = crosshair_gap_px(2, 0.05, true);
        assert!(a > 2.0 && b > a, "more spread must open the gap further: {a} then {b}");
        // a negative base gap still blooms outward from where it started
        assert!(crosshair_gap_px(-4, 0.02, true) > -4.0);

        // colour: presets come from the table, CUSTOM from the settings
        let mut s = GameSettings::default();
        s.cross_color_idx = 0;
        assert_eq!(crosshair_rgb(&s), CROSS_COLOR_CHOICES[0].1);
        s.cross_color_idx = 3;
        assert_eq!(crosshair_rgb(&s), CROSS_COLOR_CHOICES[3].1);
        s.cross_color_idx = CROSS_COLOR_CUSTOM_IDX;
        s.cross_rgb = (11, 22, 33);
        assert_eq!(crosshair_rgb(&s), (11, 22, 33), "CUSTOM reads the stored triple");
        // every preset is distinct, or a cycle click would look dead
        for i in 0..CROSS_COLOR_CUSTOM_IDX {
            for j in (i + 1)..CROSS_COLOR_CUSTOM_IDX {
                assert_ne!(
                    CROSS_COLOR_CHOICES[i].1, CROSS_COLOR_CHOICES[j].1,
                    "presets {i} and {j} are the same colour"
                );
            }
        }
    }

    /// The feedback ladder that used to be an inline `match` on the
    /// glyph's `TextColor`. Hiding must beat everything: a scoped weapon
    /// firing from the hip must not leak an aim point through a
    /// hitmarker, which is exactly what a re-ordered ladder would do.
    #[test]
    fn crosshair_hiding_beats_every_other_feedback_state() {
        // §5.2: scoped + unscoped = nothing drawn, whatever else happened
        for kill in [false, true] {
            for hit in [None, Some(false), Some(true)] {
                for blocked in [false, true] {
                    assert_eq!(
                        crosshair_feedback(true, kill, hit, blocked),
                        CrossFeedback::Hidden,
                        "noscope must hide through kill={kill} hit={hit:?} blocked={blocked}"
                    );
                }
            }
        }
        // and Hidden really is invisible, at ANY settings alpha
        for alpha in [0u8, 137, 255] {
            let c = crosshair_color(CrossFeedback::Hidden, (50, 250, 50), alpha)
                .to_srgba();
            assert_eq!(c.alpha, 0.0, "a hidden crosshair must be fully transparent");
        }

        // the rest of the ladder, in order
        assert_eq!(
            crosshair_feedback(false, true, Some(true), true),
            CrossFeedback::Kill,
            "a kill outranks a headshot marker"
        );
        assert_eq!(
            crosshair_feedback(false, false, Some(true), true),
            CrossFeedback::Headshot
        );
        assert_eq!(
            crosshair_feedback(false, false, Some(false), true),
            CrossFeedback::Hit,
            "a body hit outranks the blocked warning"
        );
        assert_eq!(
            crosshair_feedback(false, false, None, true),
            CrossFeedback::Blocked
        );
        assert_eq!(crosshair_feedback(false, false, None, false), CrossFeedback::Idle);

        // Idle is the ONLY state painted in the player's own colour
        let idle = crosshair_color(CrossFeedback::Idle, (50, 250, 50), 200).to_srgba();
        assert!((idle.red - 50.0 / 255.0).abs() < 1e-6);
        assert!((idle.green - 250.0 / 255.0).abs() < 1e-6);
        assert!((idle.blue - 50.0 / 255.0).abs() < 1e-6);
        assert!(
            (idle.alpha - 200.0 / 255.0).abs() < 1e-6,
            "idle alpha is the settings alpha, got {}",
            idle.alpha
        );
        // a feedback flash keeps its own signal colour - turning the
        // crosshair alpha down must not be able to mute a hitmarker
        let quiet_hit = crosshair_color(CrossFeedback::Hit, (50, 250, 50), 10).to_srgba();
        assert!(
            quiet_hit.alpha > 0.9,
            "a hitmarker must stay legible at alpha 10, got {}",
            quiet_hit.alpha
        );
        assert!(quiet_hit.red > quiet_hit.green, "the hit flash is red, not the fill green");
    }

    /// The settings rows are a real control surface: every row renders a
    /// live label, and every crosshair row's click actually changes the
    /// value it advertises. A persisted field with a no-op row is a dead
    /// control wearing a live one's clothes.
    #[test]
    fn every_crosshair_row_cycles_its_own_value_and_wraps_in_range() {
        // numeric cycles step up and wrap to the FLOOR, not to zero
        assert_eq!(cycle_i32(2, (-5, 12)), 3);
        assert_eq!(cycle_i32(12, (-5, 12)), -5, "wraps to the negative floor");
        assert_eq!(cycle_i32(-5, (-5, 12)), -4, "steps up out of the floor");
        // clicking from ANY start lands inside the range, always
        for range in [CROSS_SIZE_RANGE, CROSS_GAP_RANGE, CROSS_THICK_RANGE] {
            let mut v = range.0;
            for _ in 0..(range.1 - range.0 + 3) {
                v = cycle_i32(v, range);
                assert!(v >= range.0 && v <= range.1, "cycled out of range: {v} in {range:?}");
            }
        }
        // and the cycle visits every value before repeating
        let mut seen = std::collections::BTreeSet::new();
        let mut v = CROSS_GAP_RANGE.0;
        for _ in 0..(CROSS_GAP_RANGE.1 - CROSS_GAP_RANGE.0 + 1) {
            seen.insert(v);
            v = cycle_i32(v, CROSS_GAP_RANGE);
        }
        assert_eq!(
            seen.len() as i32,
            CROSS_GAP_RANGE.1 - CROSS_GAP_RANGE.0 + 1,
            "every gap value must be reachable by clicking"
        );

        // alpha cycles by VALUE, so it recovers from a hand-edited file
        assert_eq!(cycle_alpha(200), 230);
        assert_eq!(cycle_alpha(255), CROSS_ALPHA_CHOICES[0], "wraps at the top");
        assert_eq!(cycle_alpha(137), 160, "an off-preset value steps to the next preset");
        assert_eq!(cycle_alpha(0), CROSS_ALPHA_CHOICES[0]);

        // every row on the page has a live label, and no two rows share
        // one (a duplicated label means two rows edit the same thing)
        let s = GameSettings::default();
        let mut labels = std::collections::BTreeSet::new();
        for (_, kind, _) in SETTINGS_ROWS {
            let l = settings_label_text(kind, &s);
            assert!(!l.is_empty(), "a settings row rendered an empty label");
            assert!(labels.insert(l.clone()), "two settings rows render {l:?}");
        }
        assert_eq!(labels.len(), SETTINGS_ROWS.len());

        // and every CROSSHAIR row's label moves when its value moves
        let mut s = GameSettings::default();
        let before = settings_label_text(SettingsButtonKind::CrossSize, &s);
        s.cross_size = cycle_i32(s.cross_size, CROSS_SIZE_RANGE);
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossSize, &s));
        let before = settings_label_text(SettingsButtonKind::CrossGap, &s);
        s.cross_gap = cycle_i32(s.cross_gap, CROSS_GAP_RANGE);
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossGap, &s));
        let before = settings_label_text(SettingsButtonKind::CrossThickness, &s);
        s.cross_thickness = cycle_i32(s.cross_thickness, CROSS_THICK_RANGE);
        assert_ne!(
            before,
            settings_label_text(SettingsButtonKind::CrossThickness, &s)
        );
        let before = settings_label_text(SettingsButtonKind::CrossDot, &s);
        s.cross_dot = !s.cross_dot;
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossDot, &s));
        let before = settings_label_text(SettingsButtonKind::CrossColor, &s);
        s.cross_color_idx = (s.cross_color_idx + 1) % CROSS_COLOR_CHOICES.len();
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossColor, &s));
        let before = settings_label_text(SettingsButtonKind::CrossAlpha, &s);
        s.cross_alpha = cycle_alpha(s.cross_alpha);
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossAlpha, &s));
        let before = settings_label_text(SettingsButtonKind::CrossTShape, &s);
        s.cross_t_shape = !s.cross_t_shape;
        assert_ne!(before, settings_label_text(SettingsButtonKind::CrossTShape, &s));
        let before = settings_label_text(SettingsButtonKind::CrossDynamic, &s);
        s.cross_dynamic = !s.cross_dynamic;
        assert_ne!(
            before,
            settings_label_text(SettingsButtonKind::CrossDynamic, &s)
        );
        // every value reached by clicking still survives the file
        let back = parse_settings(&settings_to_text(&s));
        assert_eq!(back.cross_size, s.cross_size);
        assert_eq!(back.cross_gap, s.cross_gap);
        assert_eq!(back.cross_alpha, s.cross_alpha);
        assert_eq!(back.cross_color_idx, s.cross_color_idx);
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

    /// §C tier 2: a slot file written before the harness field existed
    /// still loads, and loads as PLATE rather than as a naked man.
    ///
    /// The same upgrade path the helmet field needed, and the same
    /// failure if it were strict - except worse: a harness defaulting to
    /// 0 would silently strip every saved profile, and the player would
    /// discover it by dying faster than they used to.
    #[test]
    fn a_pre_armor_save_file_still_loads_as_plate() {
        let five = "3,2,1,3,4"; // the helmet-era format
        let p = ForgeProfile::from_line(five).expect("a five-field file must still load");
        assert_eq!(p.helmet, 4);
        assert_eq!(
            p.armor,
            sim::default_harness(sim::Class::Line).0,
            "an absent harness must read as the class default, not as nothing"
        );
        assert!(
            sim::ArmorLoadout(p.armor).weight_kg() > 0.0,
            "loading an old profile must not undress the player"
        );
        // and the four-field original still works too
        let four = ForgeProfile::from_line("1,1,0,2").expect("the original format");
        assert_eq!(four.helmet, 0);
        assert_eq!(four.armor, sim::default_harness(sim::Class::Line).0);
        // a PRESENT but garbage harness is still an error - otherwise a
        // corrupt file would be silently accepted as an old one
        assert!(ForgeProfile::from_line("1,1,0,2,0,zzz").is_none());
    }

    /// Every plate in the library reaches the grid exactly once.
    ///
    /// The grid is built by filtering `ArmorPiece::ALL` per zone, so a
    /// piece whose `zone()` fell outside the four rows would vanish from
    /// the UI while still counting toward weight - equippable in a save
    /// file, invisible in the Forge, and impossible to take off.
    #[test]
    fn every_plate_appears_in_exactly_one_forge_row() {
        for p in sim::ArmorPiece::ALL {
            let n: usize = ARMOUR_ROWS
                .iter()
                .map(|(_, row)| row.iter().filter(|q| **q == p).count())
                .sum();
            assert_eq!(n, 1, "{} appears in {n} grid rows, not 1", p.name());
        }
        let shown: usize = ARMOUR_ROWS.iter().map(|(_, r)| r.len()).sum();
        assert_eq!(
            shown,
            sim::ARMOR_PIECES,
            "the grid must show the WHOLE harness - a plate missing from \
             every row is one that still counts toward weight, is \
             equippable from a save file, and cannot be taken off"
        );
        // no two plates share a pill label - they are toggles, and a
        // duplicate label is a toggle nobody can identify
        for p in sim::ArmorPiece::ALL {
            let same = sim::ArmorPiece::ALL
                .iter()
                .filter(|q| q.short_name() == p.short_name())
                .count();
            assert_eq!(same, 1, "{} is not a unique pill label", p.short_name());
        }
    }

    /// The grid stays LEGIBLE: even rows, and labels short enough not to
    /// wrap at the pill widths this layout produces.
    ///
    /// The first version used the four damage ZONES as its rows, which
    /// put ten pills in LEGS against two in HEAD - every label in the
    /// long rows wrapped to two lines and the page read as a wall. The
    /// grouping is anatomical now, and this is what stops it drifting
    /// back.
    #[test]
    fn the_armour_grid_rows_stay_even_and_short() {
        const MAX_PILLS: usize = 6;
        const MAX_LABEL: usize = 11;
        for (name, row) in ARMOUR_ROWS {
            assert!(!row.is_empty(), "{name} is an empty row");
            assert!(
                row.len() <= MAX_PILLS,
                "{name} has {} pills - past {MAX_PILLS} the labels wrap",
                row.len()
            );
            for p in row {
                assert!(
                    p.short_name().len() <= MAX_LABEL,
                    "{} is {} chars; the pill fits about {MAX_LABEL}",
                    p.short_name(),
                    p.short_name().len()
                );
            }
        }
        // and the rows must leave the turntable card its space
        assert!(
            ARMOURY_ROW_W_PCT < 100.0,
            "full-width rows run under the soldier preview"
        );
    }

    #[test]
    fn apply_to_selected_only_touches_the_forges_own_fields() {
        let mut sel = Selected::default();
        sel.map = MapKind::Bailey; // untouched by the Forge - must survive
        let p =
            ForgeProfile { hat: 3, tunic: 0, melee_axe: true, grenade_preset: 3, helmet: 1, armor: 0x00AB_CDEF };
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

        // §C.3: `return_efficiency` was declared and never read, which
        // made "steel springs are worse than tendons, and the mech
        // should feel it" a comment rather than a mechanic. A mech coil
        // must now give back measurably less than a human one from the
        // identical load.
        let mech = ElasticMove {
            load_s: 0.4,
            release_s: 0.18,
            stored_energy: 1.0,
            return_efficiency: MECH_RETURN_EFFICIENCY,
        };
        assert!(
            mech.release_velocity(base) < full.release_velocity(base),
            "steel must give back less than tendon: {} vs {}",
            mech.release_velocity(base),
            full.release_velocity(base)
        );
        // and by the exact ratio of the two efficiencies, so the number
        // traces to the brief rather than to taste
        let want = base * (1.0 + 0.35 * (MECH_RETURN_EFFICIENCY / HUMAN_RETURN_EFFICIENCY));
        assert!(
            (mech.release_velocity(base) - want).abs() < 1e-4,
            "mech return must scale by 0.55/0.92, got {}",
            mech.release_velocity(base)
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
