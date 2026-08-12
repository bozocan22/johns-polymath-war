//! THE THREAT SENSOR — "who can see ME", on a mech's compass ring.
//!
//! ## The mirror of `SpottedEnemies`
//!
//! `main.rs` already carries a client-side-only visibility resource:
//! `SpottedEnemies` (the minimap's red ghost dots), which asks *who can I
//! see*. It is derived fresh from real `los_clear` queries, lives in a
//! fixed slot array, allocates nothing per frame, and is documented as
//! never read by `sim.rs`. This module is the same shape asking the
//! opposite question: **who can see me, and how badly**.
//!
//! That "never read by sim.rs" guarantee is the load-bearing one. The sim
//! is bit-identical-replay critical; a threat model that fed back into it
//! would make every replay depend on what one client happened to draw.
//! Nothing here is ever read by `sim.rs`, so real delta-time, per-client
//! variation and frame-rate-dependent smoothing are all legitimate.
//!
//! ## The ray is the sim's ray
//!
//! `TdmSim::los_clear` is `pub(crate)` with a doc comment saying it was
//! exposed precisely so `main.rs` could reuse the same query instead of
//! writing a divergent one. This module uses it and writes no raycast of
//! its own — a second cover test that disagreed with the first would put
//! an indicator on a wall the bullets cannot cross.
//!
//! **Known gap, deliberately not closed here:** the sim's `sight_clear`
//! (walls AND smoke — what a bot's target selection actually uses) is
//! private, not `pub(crate)`. So a smoke grenade between you and an enemy
//! blinds the BOT but does not clear this indicator. Exposing it is a
//! one-word change in `sim.rs`, which is not this lane's file.
//!
//! ## The vertical axis does not exist on a `Fighter`
//!
//! A `Fighter` has `yaw` and `prev_yaw` and **no pitch**. Reading the bot
//! fire path confirms it: `bot_act` computes `yaw = dx.atan2(dz)` from
//! the XZ offset to its target, and the vertical component of a bot's
//! aim is synthesised inside `try_fire` from the target point, never
//! stored. So the cone tests here are honestly HORIZONTAL — a bearing
//! test in XZ. The vertical half of "can he see me" is carried by
//! `los_clear`, which is a true 3D ray. This is stated rather than
//! papered over: an enemy directly above you on a rampart, facing your
//! bearing, reads as facing you, because in this data model he is.
//!
//! ## Architecture: detection never draws, drawing never raycasts
//!
//! Two systems. `detect_threats` owns the rays and writes `ThreatSensor`.
//! `paint_sensor` reads `ThreatSensor` and moves `Node`s. They share
//! nothing else. That split is what keeps the per-frame cost bounded:
//! detection runs on a fixed cadence and evaluates a ROUND-ROBIN SLICE of
//! the roster per tick, while the painter runs every frame and
//! interpolates, so the ring still moves smoothly at 144 Hz off a 10 Hz
//! sensor.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::frontend::palette;
use crate::{branding, sim, Game, GameState, HudRoot, MainCam};

// ---------------------------------------------------------------------
// THE CONST TABLE. Every threshold, angle, radius, fade and grace period
// the feature has. The spec forbids hard-coding any of these down in the
// HUD code, and nothing below this block contains a bare number that
// means anything.
// ---------------------------------------------------------------------

/// Beyond this, an enemy is not a threat worth a light on the ring.
/// Checked as a SQUARED distance before any ray is cast — the cheap
/// reject has to come first or the stagger is pointless.
pub(crate) const DETECT_RADIUS_M: f32 = 50.0;

/// Half-angle of an enemy's VISION cone, degrees. Inside it, he has
/// visual contact with you.
pub(crate) const VISION_HALF_ANGLE_DEG: f32 = 55.0;
/// Half-angle of the tighter AIM cone. Inside it his weapon is actually
/// pointed at you, not merely his body turned your way.
pub(crate) const AIM_HALF_ANGLE_DEG: f32 = 12.0;

/// How often the detector re-evaluates, in seconds. Deliberately far
/// slower than render.
pub(crate) const DETECT_INTERVAL_S: f32 = 0.10;
/// How many enemies one detection tick is allowed to raycast. The roster
/// is walked round-robin, so with a full 16-fighter match every enemy is
/// re-evaluated every `ceil(n / this) * DETECT_INTERVAL_S`.
pub(crate) const STAGGER_PER_TICK: usize = 4;

/// How long a track holds its last positive reading through a LOS break.
/// MUST exceed the worst-case re-evaluation gap, or a continuously
/// visible enemy would fade between his own detection ticks. With 16
/// tracks at 4 per 0.10 s tick that gap is 0.40 s.
pub(crate) const GRACE_S: f32 = 0.65;
/// Seconds from nothing to full presence.
pub(crate) const FADE_IN_S: f32 = 0.18;
/// Seconds from full presence to nothing, once the grace has run out.
pub(crate) const FADE_OUT_S: f32 = 0.55;
/// Smoothing time constant for the threat VALUE, so a jump from
/// "visible" to "clear shot" ramps instead of snapping.
pub(crate) const VALUE_TAU_S: f32 = 0.12;

/// Ring radius as a fraction of the SHORT screen axis' half-extent.
/// Under 1.0 so the ring stays inside the frame on any aspect; well
/// outside the centre third so it never sits over what you are shooting.
///
/// 0.86 was the first value and the capture rejected it: the "dead
/// ahead" marker landed at y = 50 of 720, straight through the compass
/// strip and the score line. The ring has to clear the HUD's own top
/// band as well as the sight picture.
pub(crate) const RING_FRAC: f32 = 0.72;
/// Innermost pip size in UI px at the lowest threat, and at the highest.
pub(crate) const PIP_MIN_PX: f32 = 8.0;
pub(crate) const PIP_MAX_PX: f32 = 16.0;
/// Radial spacing between pips, in UI px.
pub(crate) const PIP_GAP_PX: f32 = 12.0;
/// Pips per marker. The count actually lit scales with threat.
pub(crate) const MAX_PIPS: usize = 3;
/// Pulse rate at the bottom and the top of the threat scale, in Hz.
pub(crate) const PULSE_MIN_HZ: f32 = 0.7;
pub(crate) const PULSE_MAX_HZ: f32 = 4.2;
/// How deeply the pulse cuts the marker's alpha. Below 1.0 so a marker
/// never blinks fully out — a light that vanishes reads as gone.
pub(crate) const PULSE_DEPTH: f32 = 0.40;
/// Alpha floor and ceiling for a fully faded-in marker. The ceiling is
/// under 1.0 on purpose: the owner asked for a sensor, not a billboard.
///
/// These three are not independent. The ladder is only readable if the
/// DIMMEST moment of a clear shot still beats the BRIGHTEST moment of a
/// bare contact — `ALPHA_MAX * (1 - PULSE_DEPTH)` must exceed
/// `ALPHA_MIN + (ALPHA_MAX - ALPHA_MIN) * 0.25`. A test pins it, because
/// the obvious "make it more visible" edit is to raise `ALPHA_MIN`, and
/// that is exactly the edit that breaks it.
pub(crate) const ALPHA_MIN: f32 = 0.36;
pub(crate) const ALPHA_MAX: f32 = 0.95;

/// The most tracks the sensor carries. One per fighter slot; the sim
/// caps a match at 16.
pub(crate) const MAX_TRACKS: usize = 16;

// --- §3A: the CLEAR SHOT arc -----------------------------------------
//
// In a still, `Aiming` and `ClearShot` were near-indistinguishable: both
// light three pips and differ only by head size, alpha and pulse RATE —
// and a pulse rate is invisible in a photograph. The owner's fix is a
// curved arc segment drawn on the ring behind the marker, for
// `ClearShot` and nothing else. It is a shape that is either there or
// not, which is the only kind of difference a single frame can carry.

/// Angular width of the arc, degrees, centred on the marker's bearing.
pub(crate) const ARC_SPAN_DEG: f32 = 34.0;
/// How many blocks the arc is drawn from. A UI `Node` cannot be rotated
/// without a `Transform` fight (the same reason the pips are a march of
/// positions, not one glyph), so the curve is stepped. Enough segments
/// that consecutive blocks OVERLAP at the ring's radius, or the arc
/// reads as a dotted line instead of a bar.
pub(crate) const ARC_SEGMENTS: usize = 25;
/// Side of one arc block, UI px.
pub(crate) const ARC_THICK_PX: f32 = 8.0;
/// How far INSIDE the ring the arc sits. The pips march outward from the
/// ring, so an arc on the ring itself would be hidden under the head
/// pip; this pushes it under and behind them.
pub(crate) const ARC_INSET_PX: f32 = 11.0;
/// The arc's alpha at full presence. Deliberately NOT pulsed and
/// deliberately near the top of the scale: it is the one element that
/// has to survive being looked at for a sixtieth of a second.
pub(crate) const ARC_ALPHA: f32 = 0.92;

// --- §3B: clustering --------------------------------------------------

/// Two threats closer together than this in BEARING are one marker.
///
/// The owner's spec is explicit that a merged marker communicates the
/// STRONGEST threat from that direction, not an average and not the
/// nearest. Sized against the marker's own drawn width: the pip stack is
/// ~16 px on a ~259 px ring radius, and the arc spans `ARC_SPAN_DEG`, so
/// anything under roughly a dozen degrees is already drawing on top of
/// itself. Merging is what stops that overlap reading as one smeared
/// blob of unknown strength.
pub(crate) const MERGE_SEPARATION_DEG: f32 = 14.0;

/// The most tally dots drawn beside a merged marker. A merged marker of
/// five enemies shows the cap, not five dots — the count is a hint, not
/// a readout, and the owner's standing rule is "do not be intrusive".
pub(crate) const TALLY_MAX_DOTS: usize = 3;
/// Tally dot size, UI px. Tiny on purpose.
pub(crate) const TALLY_DOT_PX: f32 = 4.0;
/// How far OUTSIDE the last pip the tally row sits, UI px.
pub(crate) const TALLY_OUTSET_PX: f32 = 10.0;
/// Spacing between tally dots along the ring's tangent, UI px.
pub(crate) const TALLY_SPACING_PX: f32 = 6.0;

// ---------------------------------------------------------------------
// The state machine — pure, and therefore testable without a World.
// ---------------------------------------------------------------------

/// The owner's five-rung ladder. The value attached to each rung is the
/// owner's own: 0.25 possible / 0.50 visual contact / 0.75
/// tracking-aiming / 1.00 clear shot.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum ThreatState {
    /// No ray, or nothing in range. Draws nothing.
    #[default]
    NotDetected,
    /// The line is open — he COULD see you if he turned. He has not.
    Visible,
    /// You are inside his vision cone: visual contact.
    Tracking,
    /// You are inside the tighter aim cone: the weapon is on you.
    Aiming,
    /// On you AND the sim says you are the target he is engaging.
    ClearShot,
}

impl ThreatState {
    pub(crate) fn value(self) -> f32 {
        match self {
            ThreatState::NotDetected => 0.00,
            ThreatState::Visible => 0.25,
            ThreatState::Tracking => 0.50,
            ThreatState::Aiming => 0.75,
            ThreatState::ClearShot => 1.00,
        }
    }
    pub(crate) fn detected(self) -> bool {
        self != ThreatState::NotDetected
    }
}

/// Cosine thresholds, derived once from the degree constants so the
/// tuning knob stays in the unit a human thinks in.
pub(crate) fn vision_cos() -> f32 {
    VISION_HALF_ANGLE_DEG.to_radians().cos()
}
pub(crate) fn aim_cos() -> f32 {
    AIM_HALF_ANGLE_DEG.to_radians().cos()
}

/// THE classifier.
///
/// * `in_range` — the squared-distance gate, already applied.
/// * `los` — `TdmSim::los_clear` from his eye to your chest.
/// * `cos` — his facing (XZ, from `Fighter.yaw`) dotted with the
///   normalised XZ direction to you.
/// * `engaging_me` — the sim's own `Fighter.engaging` names you. This is
///   READ from the sim, not re-derived: only the bot brain knows which
///   body it picked.
pub(crate) fn classify(in_range: bool, los: bool, cos: f32, engaging_me: bool) -> ThreatState {
    if !in_range || !los {
        return ThreatState::NotDetected;
    }
    if cos < vision_cos() {
        // The line is open but he is looking somewhere else.
        ThreatState::Visible
    } else if cos < aim_cos() {
        ThreatState::Tracking
    } else if engaging_me {
        ThreatState::ClearShot
    } else {
        ThreatState::Aiming
    }
}

/// One frame of presence. Returns `(fade, hold)`.
///
/// The grace is the whole point: an LOS break of a few frames — someone
/// stepping behind a post — must NOT flicker the light. `hold` is
/// recharged to `GRACE_S` by a positive classification and drains in real
/// time; only once it is empty does `fade` start falling.
pub(crate) fn fade_step(fade: f32, hold: f32, dt: f32) -> (f32, f32) {
    let hold = (hold - dt).max(0.0);
    let fade = if hold > 0.0 {
        (fade + dt / FADE_IN_S).min(1.0)
    } else {
        (fade - dt / FADE_OUT_S).max(0.0)
    };
    (fade, hold)
}

/// Exponential approach, frame-rate independent. Used to interpolate the
/// threat value between the sensor's slow ticks so the ring reads smooth.
pub(crate) fn approach(cur: f32, target: f32, dt: f32, tau: f32) -> f32 {
    if tau <= 0.0 {
        return target;
    }
    cur + (target - cur) * (1.0 - (-dt / tau).exp())
}

/// Bearing of a world direction relative to where the camera looks, in
/// radians. 0 = dead ahead, +pi/2 = hard right, ±pi = behind. Horizontal
/// only, which is what a compass ring is.
pub(crate) fn screen_bearing(fwd: Vec2, right: Vec2, to_target: Vec2) -> f32 {
    if to_target.length_squared() < 1e-8 {
        return 0.0;
    }
    let d = to_target.normalize();
    d.dot(right).atan2(d.dot(fwd))
}

/// Where on the ring a bearing lands, as an offset in UI px from the
/// ring's centre. `+x` is right, `+y` is DOWN (UI convention), so a
/// bearing of 0 sits at the top of the ring.
pub(crate) fn ring_offset(bearing: f32, radius: f32) -> Vec2 {
    Vec2::new(bearing.sin() * radius, -bearing.cos() * radius)
}

/// The outward radial unit vector for a bearing — the axis the pips march
/// along, which is what makes an unrotatable stack of rectangles point
/// somewhere.
pub(crate) fn radial_dir(bearing: f32) -> Vec2 {
    Vec2::new(bearing.sin(), -bearing.cos())
}

/// Where an enemy LOOKS FROM. The idiom is the minimap's and
/// `muzzle_origin`'s: `EYE_REL` unless the body is too short for it.
///
/// This is `pub(crate)` and shared with the capture staging on purpose.
/// The first staged run had the staging asking `los_clear` from a bare
/// `EYE_REL` while the sensor asked from `EYE_REL.min(h - 0.12)` — four
/// centimetres apart — and over 14 m of low cover those four centimetres
/// disagreed. Three frames were staged on a line the sensor could not
/// see through, and the empty ring in them was CORRECT. One helper, one
/// answer.
pub(crate) fn eye_point(pos: [f32; 3], height: f32) -> [f32; 3] {
    [pos[0], pos[1] + sim::EYE_REL.min(height - 0.12), pos[2]]
}

/// Where an enemy is looking AT: centre mass, not the head. A threat
/// model built on head visibility would clear the moment you crouched
/// behind a crate that still leaves your chest exposed.
pub(crate) fn chest_point(pos: [f32; 3], height: f32) -> [f32; 3] {
    [pos[0], pos[1] + height * 0.55, pos[2]]
}

/// How many of the `MAX_PIPS` are lit at a given threat value. One pip is
/// a contact; three is a gun on you.
pub(crate) fn lit_pips(value: f32) -> usize {
    match value {
        v if v >= ThreatState::Aiming.value() => 3,
        v if v >= ThreatState::Tracking.value() => 2,
        _ => 1,
    }
    .min(MAX_PIPS)
}

/// The marker's alpha this frame: presence x threat x pulse.
pub(crate) fn marker_alpha(fade: f32, value: f32, elapsed: f32) -> f32 {
    let base = ALPHA_MIN + (ALPHA_MAX - ALPHA_MIN) * value.clamp(0.0, 1.0);
    let hz = PULSE_MIN_HZ + (PULSE_MAX_HZ - PULSE_MIN_HZ) * value.clamp(0.0, 1.0);
    let wave = 0.5 + 0.5 * (elapsed * hz * std::f32::consts::TAU).sin();
    let pulse = 1.0 - PULSE_DEPTH + PULSE_DEPTH * wave;
    (fade.clamp(0.0, 1.0) * base * pulse).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------
// §3B — CLUSTERING. Pure, allocation-free, and therefore testable
// without a World. The merge/separate decision is the whole feature, so
// it lives here rather than inside the painter where nothing could call
// it.
// ---------------------------------------------------------------------

/// Signed shortest angular difference `a - b`, wrapped to (-pi, pi].
///
/// This wrap is load-bearing and is the obvious thing to forget: two
/// enemies at bearings +179 deg and -179 deg are two degrees apart and
/// must merge, but a naive subtraction calls them 358 degrees apart and
/// draws two markers on top of each other at the bottom of the ring.
pub(crate) fn ang_delta(a: f32, b: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let mut d = (a - b) % TAU;
    if d > PI {
        d -= TAU;
    } else if d <= -PI {
        d += TAU;
    }
    d
}

/// One track's contribution to the ring, already reduced to what the
/// clusterer needs. Deliberately NOT the track: clustering has no
/// business knowing about world positions or grace timers.
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Blip {
    pub(crate) bearing: f32,
    pub(crate) value: f32,
    pub(crate) state: ThreatState,
    pub(crate) fade: f32,
}

/// One drawn marker. `count` is how many enemies it stands for; the
/// bearing, value and state are the LEADER's — the strongest threat from
/// that direction, which is what the owner's spec asks a merged marker
/// to communicate.
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Cluster {
    pub(crate) bearing: f32,
    pub(crate) value: f32,
    pub(crate) state: ThreatState,
    pub(crate) fade: f32,
    pub(crate) count: usize,
}

/// Sort key: strongest first. `value` is the smoothed threat; `fade`
/// breaks the tie so that between two equally-threatening contacts the
/// one that is actually drawn brighter leads.
fn blip_key(b: &Blip) -> (f32, f32) {
    (b.value, b.fade)
}

/// Cluster blips into markers. Returns how many of `out` are valid.
///
/// Greedy, over a list sorted strongest-first. That order is what makes
/// "the merged marker shows the strongest" fall out for free rather than
/// needing a second pass: the first blip to claim a direction is by
/// construction the worst threat in it, and it keeps its own bearing,
/// value and state while the others only bump the count.
///
/// Allocation-free — a fixed index array and an insertion sort, the same
/// discipline as the `[ThreatTrack; 16]` it feeds from. This runs every
/// frame in the painter; a `Vec` here would be a per-frame heap churn on
/// the render path.
pub(crate) fn cluster_blips(blips: &[Blip], out: &mut [Cluster; MAX_TRACKS]) -> usize {
    let n = blips.len().min(MAX_TRACKS);
    let mut order = [0usize; MAX_TRACKS];
    for (i, o) in order.iter_mut().enumerate().take(n) {
        *o = i;
    }
    // insertion sort, descending. n <= 16.
    for i in 1..n {
        let mut j = i;
        while j > 0 && blip_key(&blips[order[j]]) > blip_key(&blips[order[j - 1]]) {
            order.swap(j, j - 1);
            j -= 1;
        }
    }

    let thr = MERGE_SEPARATION_DEG.to_radians();
    let mut count = 0usize;
    for &idx in order.iter().take(n) {
        let b = &blips[idx];
        let mut merged = false;
        for c in out.iter_mut().take(count) {
            if ang_delta(c.bearing, b.bearing).abs() <= thr {
                c.count += 1;
                // presence is the loudest member's: a marker standing
                // for three men must not fade because two of them are
                // half gone.
                c.fade = c.fade.max(b.fade);
                merged = true;
                break;
            }
        }
        if !merged {
            out[count] = Cluster {
                bearing: b.bearing,
                value: b.value,
                state: b.state,
                fade: b.fade,
                count: 1,
            };
            count += 1;
        }
    }
    count
}

/// How many tally dots a marker draws. One per enemy BEYOND the first —
/// a lone contact draws none, which is the common case and must stay
/// clean.
pub(crate) fn tally_dots(count: usize) -> usize {
    count.saturating_sub(1).min(TALLY_MAX_DOTS)
}

/// Whether this marker gets the clear-shot arc.
pub(crate) fn wants_arc(state: ThreatState) -> bool {
    state == ThreatState::ClearShot
}

/// Offset of the j-th arc block from the MARKER's own position, in UI
/// px, for a marker at `bearing` on a ring of `radius`.
pub(crate) fn arc_offset(j: usize, bearing: f32, radius: f32) -> Vec2 {
    let span = ARC_SPAN_DEG.to_radians();
    let t = if ARC_SEGMENTS <= 1 {
        0.5
    } else {
        j as f32 / (ARC_SEGMENTS - 1) as f32
    };
    let a = bearing - span * 0.5 + span * t;
    ring_offset(a, radius - ARC_INSET_PX) - ring_offset(bearing, radius)
}

/// Pip size in UI px for the k-th pip out from the ring at a given threat.
pub(crate) fn pip_size(k: usize, value: f32) -> f32 {
    let head = PIP_MIN_PX + (PIP_MAX_PX - PIP_MIN_PX) * value.clamp(0.0, 1.0);
    // each pip further out is smaller; the taper is what points
    head * (1.0 - 0.26 * k as f32).max(0.35)
}

// ---------------------------------------------------------------------
// The resource
// ---------------------------------------------------------------------

/// One tracked enemy's threat state. `pos` is the LAST KNOWN world
/// position, held while the grace and the fade run — the marker keeps
/// pointing where he was, exactly as the minimap's ghost dot does.
#[derive(Clone, Copy, Default)]
pub(crate) struct ThreatTrack {
    pub(crate) live: bool,
    pub(crate) state: ThreatState,
    /// Smoothed 0..1 threat, what the painter reads.
    pub(crate) value: f32,
    pub(crate) fade: f32,
    pub(crate) hold: f32,
    pub(crate) pos: Vec3,
}

/// Client-side-only presentational state: what is pointed at the LOCAL
/// player right now. Never read by `sim.rs`, never affects a hit, a
/// damage number or an outcome. Indexed by fighter index, so the array is
/// its own slot table and nothing ever has to be claimed or freed.
#[derive(Resource)]
pub(crate) struct ThreatSensor {
    pub(crate) tracks: [ThreatTrack; MAX_TRACKS],
    /// Round-robin cursor into the roster — the stagger.
    cursor: usize,
    /// Cadence accumulator.
    accum: f32,
}

impl Default for ThreatSensor {
    fn default() -> Self {
        ThreatSensor {
            tracks: [ThreatTrack::default(); MAX_TRACKS],
            cursor: 0,
            accum: 0.0,
        }
    }
}

// ---------------------------------------------------------------------
// Detection — the only system here that touches a ray
// ---------------------------------------------------------------------

/// `JK_THREAT_DEBUG=1` prints what the sensor believes, twice a second.
/// Off by default and resolved once — a diagnostic for the capture lane,
/// because a marker that never appears looks identical whether the cause
/// is detection or layout.
fn debug_on(flag: &mut Option<bool>) -> bool {
    *flag.get_or_insert_with(|| std::env::var("JK_THREAT_DEBUG").is_ok())
}

fn detect_threats(
    game: Res<Game>,
    time: Res<Time>,
    mut sensor: ResMut<ThreatSensor>,
    mut dbg: Local<Option<bool>>,
    mut dbg_t: Local<f32>,
) {
    let dt = time.delta_secs();
    let s = &game.sim;
    let n = s.fighters.len().min(MAX_TRACKS);

    // Dead, or no body to threaten: everything decays out on its own.
    let me = &s.fighters[s.player];
    let alive = me.alive();

    if alive {
        sensor.accum += dt;
        if sensor.accum >= DETECT_INTERVAL_S {
            // One tick's worth of work, whatever the frame rate did —
            // never more than one slice per frame.
            sensor.accum = (sensor.accum - DETECT_INTERVAL_S).min(DETECT_INTERVAL_S);
            let chest = chest_point(me.pos, me.height());
            let my_team = me.team;
            let player = s.player;
            for _ in 0..STAGGER_PER_TICK.min(n) {
                let i = sensor.cursor % n.max(1);
                sensor.cursor = (sensor.cursor + 1) % n.max(1);
                let f = &s.fighters[i];
                if i == player || f.team == my_team || !f.alive() {
                    sensor.tracks[i].state = ThreatState::NotDetected;
                    continue;
                }
                // 1. the cheap reject, BEFORE any ray
                let (dx, dz) = (chest[0] - f.pos[0], chest[2] - f.pos[2]);
                let d2 = dx * dx + dz * dz;
                let in_range = d2 <= DETECT_RADIUS_M * DETECT_RADIUS_M;
                let st = if !in_range {
                    ThreatState::NotDetected
                } else {
                    // 2. the sim's own ray, from HIS eye to my chest
                    let los = s.los_clear(eye_point(f.pos, f.height()), chest);
                    // 3. the cones, in XZ off `Fighter.yaw` — see the
                    // module header on why there is no pitch to use.
                    let fwd = Vec2::new(f.yaw.sin(), f.yaw.cos());
                    let to_me = Vec2::new(dx, dz);
                    let cos = if to_me.length_squared() < 1e-6 {
                        1.0
                    } else {
                        fwd.dot(to_me.normalize())
                    };
                    classify(in_range, los, cos, f.engaging == player as i32)
                };
                let t = &mut sensor.tracks[i];
                t.state = st;
                if st.detected() {
                    t.live = true;
                    t.hold = GRACE_S;
                    t.pos = Vec3::new(f.pos[0], f.pos[1] + f.height() * 0.55, f.pos[2]);
                }
            }
        }
    }

    // Presence and smoothing run EVERY frame on EVERY track, in real
    // delta-time, so the ring is smooth between the sensor's slow ticks.
    let elapsed_kill = !alive;
    for t in sensor.tracks.iter_mut() {
        if !t.live {
            continue;
        }
        if elapsed_kill {
            t.hold = 0.0;
        }
        let (fade, hold) = fade_step(t.fade, t.hold, dt);
        t.fade = fade;
        t.hold = hold;
        let target = if hold > 0.0 { t.state.value() } else { 0.0 };
        t.value = approach(t.value, target, dt, VALUE_TAU_S);
        if t.fade <= 0.0 {
            *t = ThreatTrack::default();
        }
    }

    if debug_on(&mut dbg) {
        *dbg_t += dt;
        if *dbg_t > 0.5 {
            *dbg_t = 0.0;
            let live: Vec<String> = sensor
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.live)
                .map(|(i, t)| format!("{i}:{:?} v{:.2} f{:.2}", t.state, t.value, t.fade))
                .collect();
            eprintln!(
                "[threat] n={n} alive={alive} live=[{}]",
                live.join(", ")
            );
        }
    }
}

// ---------------------------------------------------------------------
// The ring — reads the resource, casts nothing
// ---------------------------------------------------------------------

#[derive(Component)]
struct SensorRoot;
/// Marker slot. Since §3 this indexes a CLUSTER, not a track: several
/// enemies on one bearing share one marker.
#[derive(Component)]
struct ThreatMarker(usize);
#[derive(Component)]
struct ThreatPip(usize, usize);
/// One block of the clear-shot arc: (marker slot, block index).
#[derive(Component)]
struct ThreatArc(usize, usize);
/// One "and another" dot: (marker slot, dot index).
#[derive(Component)]
struct ThreatTally(usize, usize);

fn spawn_sensor(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // Hidden for the same reason the strip's root is: the initial
            // state is Title and `hud_visibility` only fires on Playing's
            // enter/exit.
            Visibility::Hidden,
            HudRoot,
            SensorRoot,
        ))
        .with_children(|r| {
            for i in 0..MAX_TRACKS {
                r.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    Visibility::Hidden,
                    ThreatMarker(i),
                ))
                .with_children(|m| {
                    // The arc is spawned FIRST so it sits behind the
                    // pips: Bevy's UI stacks later siblings on top, and
                    // "behind the pips" is the owner's word for it.
                    for j in 0..ARC_SEGMENTS {
                        m.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderRadius::all(Val::Px(ARC_THICK_PX * 0.5)),
                            Visibility::Hidden,
                            ThreatArc(i, j),
                        ));
                    }
                    for k in 0..MAX_PIPS {
                        m.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor(Color::NONE),
                            BorderRadius::all(Val::Px(3.0)),
                            Visibility::Hidden,
                            ThreatPip(i, k),
                        ));
                    }
                    for d in 0..TALLY_MAX_DOTS {
                        m.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderRadius::all(Val::Px(TALLY_DOT_PX * 0.5)),
                            Visibility::Hidden,
                            ThreatTally(i, d),
                        ));
                    }
                });
            }
        });
}

/// What the painters draw, resolved once per frame.
///
/// §3 split `paint_sensor` into a RESOLVER and four one-query painters.
/// The four-way `ParamSet` the arc and the tally would have needed took
/// rustc's monomorphisation collector past its recursion limit on this
/// machine and killed the RELEASE build outright
/// (STATUS_STACK_BUFFER_OVERRUN inside
/// `collect_and_partition_mono_items`, the same crash class the debug
/// profile is already known for here). Passing the answer through a
/// plain resource is both the fix and the clearer shape: exactly one
/// place computes geometry, and four dumb systems move `Node`s.
#[derive(Resource)]
pub(crate) struct RingLayout {
    pub(crate) clusters: [Cluster; MAX_TRACKS],
    pub(crate) n: usize,
    /// Screen position of each cluster's marker, UI px.
    pub(crate) pos: [Vec2; MAX_TRACKS],
    /// Outward radial unit vector for each cluster.
    pub(crate) dir: [Vec2; MAX_TRACKS],
    /// Pulsed alpha for each cluster's pips.
    pub(crate) alpha: [f32; MAX_TRACKS],
    pub(crate) radius: f32,
}

impl Default for RingLayout {
    fn default() -> Self {
        RingLayout {
            clusters: [Cluster::default(); MAX_TRACKS],
            n: 0,
            pos: [Vec2::ZERO; MAX_TRACKS],
            dir: [Vec2::ZERO; MAX_TRACKS],
            alpha: [0.0; MAX_TRACKS],
            radius: 0.0,
        }
    }
}

impl RingLayout {
    /// Is slot `i` drawn at all? Every painter gates on this, so a
    /// marker's parts can never disagree about whether it exists.
    pub(crate) fn shown(&self, i: usize) -> bool {
        i < self.n && self.clusters[i].count > 0 && self.clusters[i].fade > 0.0
    }
}

fn resolve_ring(
    sensor: Res<ThreatSensor>,
    time: Res<Time>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cam_q: Query<&GlobalTransform, With<MainCam>>,
    mut out: ResMut<RingLayout>,
    mut dbg: Local<Option<bool>>,
    mut dbg_t: Local<f32>,
) {
    let (Ok(win), Ok(cam_tf)) = (windows.get_single(), cam_q.get_single()) else {
        out.n = 0;
        return;
    };
    // UI space, not physical: `sync_ui_scale` drives `UiScale` off the
    // window height, so a px here is a px there only after the divide.
    let scale = if ui_scale.0 > 0.0 { ui_scale.0 } else { 1.0 };
    let (w, h) = (win.width() / scale, win.height() / scale);
    let centre = Vec2::new(w * 0.5, h * 0.5);
    let radius = w.min(h) * 0.5 * RING_FRAC;

    let f3 = cam_tf.forward();
    let r3 = cam_tf.right();
    let fwd = Vec2::new(f3.x, f3.z);
    let right = Vec2::new(r3.x, r3.z);
    let (fwd, right) = if fwd.length_squared() < 1e-6 {
        // camera looking straight down: fall back on world axes rather
        // than dividing by zero and painting NaNs into the layout
        (Vec2::Y, Vec2::X)
    } else {
        (fwd.normalize(), right.normalize_or_zero())
    };
    let cam_xz = Vec2::new(cam_tf.translation().x, cam_tf.translation().z);
    let elapsed = time.elapsed_secs();

    // §3B — reduce every live track to a bearing, then CLUSTER. From
    // here down a "marker" is a cluster, and several enemies on one
    // bearing share one.
    let mut blips = [Blip::default(); MAX_TRACKS];
    let mut nb = 0usize;
    for t in sensor.tracks.iter() {
        if !t.live || t.fade <= 0.0 {
            continue;
        }
        blips[nb] = Blip {
            bearing: screen_bearing(fwd, right, Vec2::new(t.pos.x, t.pos.z) - cam_xz),
            value: t.value,
            state: t.state,
            fade: t.fade,
        };
        nb += 1;
    }
    let mut clusters = [Cluster::default(); MAX_TRACKS];
    let n = cluster_blips(&blips[..nb], &mut clusters);

    out.radius = radius;
    out.n = n;
    out.clusters = clusters;
    for i in 0..n {
        let c = clusters[i];
        out.pos[i] = centre + ring_offset(c.bearing, radius);
        out.dir[i] = radial_dir(c.bearing);
        out.alpha[i] = marker_alpha(c.fade, c.value, elapsed);
    }

    if debug_on(&mut dbg) {
        *dbg_t += time.delta_secs();
        if *dbg_t > 0.5 {
            *dbg_t = 0.0;
            let list: Vec<String> = (0..n)
                .map(|i| {
                    format!(
                        "{:?}x{}@{:.0},{:.0}",
                        clusters[i].state, clusters[i].count, out.pos[i].x, out.pos[i].y
                    )
                })
                .collect();
            eprintln!(
                "[threat/paint] ui={w:.0}x{h:.0} scale={scale:.2} r={radius:.0} \
                 blips={nb} clusters=[{}]",
                list.join(", ")
            );
        }
    }
}

fn paint_markers(lay: Res<RingLayout>, mut q: Query<(&ThreatMarker, &mut Node, &mut Visibility)>) {
    for (m, mut node, mut v) in q.iter_mut() {
        if lay.shown(m.0) {
            node.left = Val::Px(lay.pos[m.0].x);
            node.top = Val::Px(lay.pos[m.0].y);
            *v = Visibility::Inherited;
        } else {
            *v = Visibility::Hidden;
        }
    }
}

/// §3A — the CLEAR SHOT arc, and nothing below a clear shot.
fn paint_arcs(
    lay: Res<RingLayout>,
    mut q: Query<(&ThreatArc, &mut Node, &mut BackgroundColor, &mut Visibility)>,
) {
    let (er, eg, eb) = branding::signal::Side::Enemy.accent_rgb();
    for (arc, mut node, mut bg, mut v) in q.iter_mut() {
        if !lay.shown(arc.0) || !wants_arc(lay.clusters[arc.0].state) {
            *v = Visibility::Hidden;
            continue;
        }
        let off = arc_offset(arc.1, lay.clusters[arc.0].bearing, lay.radius);
        node.left = Val::Px(off.x - ARC_THICK_PX * 0.5);
        node.top = Val::Px(off.y - ARC_THICK_PX * 0.5);
        node.width = Val::Px(ARC_THICK_PX);
        node.height = Val::Px(ARC_THICK_PX);
        // NOT pulsed. The arc is a shape, not a brightness: a pulse
        // would put it back into the class of differences a single frame
        // cannot carry, which is the entire reason it exists.
        let a = lay.clusters[arc.0].fade.clamp(0.0, 1.0) * ARC_ALPHA;
        *bg = BackgroundColor(Color::srgba(er, eg, eb, a));
        *v = Visibility::Inherited;
    }
}

/// §3B — "and this many more from the same direction".
fn paint_tally(
    lay: Res<RingLayout>,
    mut q: Query<(&ThreatTally, &mut Node, &mut BackgroundColor, &mut Visibility)>,
) {
    let (er, eg, eb) = branding::signal::Side::Enemy.accent_rgb();
    for (tal, mut node, mut bg, mut v) in q.iter_mut() {
        let dots = if lay.shown(tal.0) {
            tally_dots(lay.clusters[tal.0].count)
        } else {
            0
        };
        if tal.1 >= dots {
            *v = Visibility::Hidden;
            continue;
        }
        let dir = lay.dir[tal.0];
        let tangent = Vec2::new(dir.y, -dir.x);
        let outset = PIP_GAP_PX * (MAX_PIPS - 1) as f32 + TALLY_OUTSET_PX;
        let spread = (tal.1 as f32 - (dots as f32 - 1.0) * 0.5) * TALLY_SPACING_PX;
        let off = dir * outset + tangent * spread;
        node.left = Val::Px(off.x - TALLY_DOT_PX * 0.5);
        node.top = Val::Px(off.y - TALLY_DOT_PX * 0.5);
        node.width = Val::Px(TALLY_DOT_PX);
        node.height = Val::Px(TALLY_DOT_PX);
        *bg = BackgroundColor(Color::srgba(er, eg, eb, lay.alpha[tal.0]));
        *v = Visibility::Inherited;
    }
}

#[allow(clippy::type_complexity)]
fn paint_pips(
    lay: Res<RingLayout>,
    mut q: Query<(
        &ThreatPip,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
        &mut Visibility,
    )>,
) {
    let (er, eg, eb) = branding::signal::Side::Enemy.accent_rgb();
    for (pip, mut node, mut bg, mut radius_c, mut v) in q.iter_mut() {
        let value = lay.clusters[pip.0].value;
        if !lay.shown(pip.0) || pip.1 >= lit_pips(value) {
            *v = Visibility::Hidden;
            continue;
        }
        let (dir, alpha) = (lay.dir[pip.0], lay.alpha[pip.0]);
        let size = pip_size(pip.1, value);
        // The pips march OUTWARD along the radial axis. That march is the
        // whole directional cue: a UI `Node` cannot be rotated without a
        // `Transform` fight, so the arrow is made of positions, not of
        // one rotated glyph.
        let off = dir * (PIP_GAP_PX * pip.1 as f32);
        node.left = Val::Px(off.x - size * 0.5);
        node.top = Val::Px(off.y - size * 0.5);
        node.width = Val::Px(size);
        node.height = Val::Px(size);
        // The head pip is round; the trailing ones squarer, so the stack
        // reads as a dart rather than a string of beads.
        *radius_c = BorderRadius::all(Val::Px(size * if pip.1 == 0 { 0.5 } else { 0.28 }));
        // trailing pips are dimmer, which also points
        let a = alpha * (1.0 - 0.22 * pip.1 as f32);
        *bg = BackgroundColor(Color::srgba(er, eg, eb, a));
        *v = Visibility::Inherited;
    }
}

// ---------------------------------------------------------------------
// The plugin — two lines of wiring in main.rs
// ---------------------------------------------------------------------

pub struct ThreatSensorPlugin;

impl Plugin for ThreatSensorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThreatSensor>()
            .init_resource::<RingLayout>()
            .add_systems(Startup, spawn_sensor)
            .add_systems(
                Update,
                (
                    detect_threats,
                    resolve_ring,
                    paint_markers,
                    paint_arcs,
                    paint_tally,
                    paint_pips,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
        capture::register(app);
    }
}

/// Keeps `palette` reachable for the marker work that Section 3 will add
/// (the cluster plate has a panel behind it); referenced here so the
/// import is not a warning today.
#[allow(dead_code)]
const _SCRIM: Color = palette::PANEL;

// =====================================================================
// THE CAPTURE — this feature cannot be seen without staging
// =====================================================================
//
// A threat indicator is invisible in a normal capture: bots wander, and
// no script has ever aimed one at the subject on purpose. So the states
// are STAGED — one enemy planted at a known bearing with a known facing,
// the rest of the roster parked at the far corner so the frame holds one
// reading at a time. Everything in here is inert without `JK_CAPTURE`.
pub mod capture {
    use super::*;
    use crate::{beat, CapBeat, CaptureMode};

    pub const SCRIPT: &str = "threat_sensor";

    /// What the staged enemy is doing at a given time. `bearing` is
    /// measured from the player's own facing (which the beats pin to
    /// yaw 0), `facing` is where the enemy looks: `Away`, `AtPlayer`, or
    /// `AtPlayerEngaged` (which also names the player as his target, the
    /// way `bot_act` would).
    #[derive(Clone, Copy)]
    enum Face {
        Away,
        AtPlayer,
        AtPlayerEngaged,
    }

    #[derive(Clone, Copy)]
    enum Where {
        /// At roughly this bearing (radians, from world +Z, which is the
        /// facing the beats pin the player to), on a line that is
        /// actually OPEN.
        ///
        /// The first cut passed a literal 14 m and got two usable frames
        /// out of six: Arena's cover blocked the line on three bearings
        /// and the sensor correctly reported nothing, which photographs
        /// identically to a broken feature. `capture_stage_pos` only
        /// proves the SUBJECT stands in the clear, never that anything
        /// can see him. So the distance is searched, and the bearing is
        /// allowed to wander.
        ///
        /// A LIST of acceptable bearings, tried in order, because a map
        /// can simply have a wall down one side: Arena's stage point has
        /// no open line anywhere to the player's right, at any range or
        /// jitter, so the lateral frame asks for "right, or failing that
        /// left" rather than photographing an honest but useless blank.
        Open(&'static [f32]),
        /// Somewhere this module SEARCHES for: a point at ~10-18 m whose
        /// line to the player is blocked by real cover. A literal cannot
        /// express that on a randomised map.
        BehindCover,
        /// The spec's "enemy strafing behind a post": he alternates
        /// between the open spot and the cover spot at `FLICKER_HZ`.
        ///
        /// The first attempt at this was ONE deterministic 0.6 s duck
        /// with the snapshot aimed at its middle. It never landed: the
        /// screenshot writer stalls the app for whole seconds, and a
        /// single frame's delta-time can leap the entire window, so
        /// frame 07 photographed an empty ring taken after he had
        /// already come back. A sustained square wave removes the
        /// timing problem entirely — the marker must be lit at EVERY
        /// instant of the window, so any frame inside it is evidence.
        Flicker,
    }

    /// How fast the strafing enemy crosses the post. Each half-cycle is
    /// far shorter than `GRACE_S`, which is exactly the condition the
    /// grace exists to survive.
    const FLICKER_HZ: f32 = 5.0;

    /// Distances tried, in order, when searching for an open line.
    const OPEN_RANGES_M: [f32; 7] = [14.0, 11.0, 17.0, 9.0, 20.0, 7.0, 23.0];
    /// How far the bearing may wander to find one, in radians. Ordered
    /// by magnitude so the bearing stays as close to the intent as the
    /// map allows.
    const OPEN_JITTER: [f32; 9] = [0.0, 0.20, -0.20, 0.42, -0.42, 0.62, -0.62, 0.88, -0.88];
    /// Clearance a staged body needs from every cover box, in metres.
    ///
    /// Without this the search happily returned a point INSIDE a crate:
    /// the line out of it read as open, the sim's collision push-out then
    /// moved the body somewhere else entirely, and the sensor - correctly
    /// - saw a wall. Three of six frames were empty for this reason and
    /// for no reason to do with detection at all.
    const OPEN_CLEAR_M: f32 = 1.2;

    /// What the OTHER staged enemies are doing. §3 needs three bodies,
    /// not one: the merge rule cannot be photographed with a single
    /// subject.
    #[derive(Clone, Copy, PartialEq)]
    enum Extra {
        /// Parked in the far corner, out of `DETECT_RADIUS_M`.
        Parked,
        /// One extra enemy off to the side, well beyond
        /// `MERGE_SEPARATION_DEG` — two markers must appear.
        Separated,
        /// Two extra enemies shoulder-to-shoulder with the subject, a
        /// few degrees apart — one marker must appear, and it must show
        /// the strongest of the three.
        CoBearing,
    }

    /// Lateral offsets, metres, tried in order when tucking a co-bearing
    /// enemy in beside the subject. A metre or two at ~14 m is only a
    /// few degrees of bearing, comfortably inside the merge threshold,
    /// while still being two distinct bodies the sim will not shove
    /// through each other.
    const SIDE_OFFSETS_M: [f32; 6] = [1.6, -1.6, 2.6, -2.6, 0.9, -0.9];

    const AHEAD: &[f32] = &[0.0];
    const BEHIND: &[f32] = &[std::f32::consts::PI];
    /// Right if the map allows it, left if it does not.
    const LATERAL: &[f32] = &[std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2];

    const STAGE: &[(f32, Where, Face)] = &[
        (0.0, Where::BehindCover, Face::AtPlayerEngaged),
        (1.6, Where::Open(AHEAD), Face::Away),
        (3.0, Where::Open(AHEAD), Face::AtPlayer),
        (4.4, Where::Open(AHEAD), Face::AtPlayerEngaged),
        (5.8, Where::Open(BEHIND), Face::AtPlayerEngaged),
        (7.2, Where::Open(LATERAL), Face::AtPlayerEngaged),
        // --- §3: reacquire dead ahead, then DUCK. Times below are on
        // the BEAT clock, which `drive` now shares.
        (8.6, Where::Open(AHEAD), Face::AtPlayerEngaged),
        // He strafes in and out of cover at FLICKER_HZ for a second and
        // a half. The marker must be lit at every instant of it.
        (9.8, Where::Flicker, Face::AtPlayerEngaged),
        (11.3, Where::Open(AHEAD), Face::AtPlayerEngaged),
        // --- §3: two separated threats. The subject stays dead ahead on
        // a CLEAR SHOT; the extra goes lateral, merely Aiming. One frame
        // then carries the arc and the no-arc marker side by side.
        (11.4, Where::Open(AHEAD), Face::AtPlayerEngaged),
        // --- §3: co-bearing. The subject is only AIMING; one of the two
        // men beside him has the clear shot, so the merged marker must
        // show HIS state, not the subject's.
        (13.0, Where::Open(AHEAD), Face::AtPlayer),
    ];

    const EXTRAS: &[(f32, Extra)] = &[
        (0.0, Extra::Parked),
        (11.4, Extra::Separated),
        (13.0, Extra::CoBearing),
    ];

    pub const BEATS: &[CapBeat] = &[
        CapBeat { look: Some((0.0, 0.0)), ..beat(0.2) },
        CapBeat { snap: Some("01-behind-cover-no-indicator"), ..beat(1.4) },
        CapBeat { snap: Some("02-open-looking-away"), ..beat(2.8) },
        CapBeat { snap: Some("03-aimed-at-me-no-clear-shot"), ..beat(4.2) },
        CapBeat { snap: Some("04-aimed-clear-shot"), ..beat(5.6) },
        CapBeat { snap: Some("05-behind-me-offscreen"), ..beat(7.0) },
        CapBeat { snap: Some("06-hard-right"), ..beat(8.4) },
        CapBeat { snap: Some("07-strafing-behind-post-grace-holds"), ..beat(10.6) },
        CapBeat { snap: Some("08-two-separated-two-markers"), ..beat(12.6) },
        CapBeat { snap: Some("09-co-bearing-one-merged-marker"), ..beat(14.2) },
        CapBeat { end: true, ..beat(14.8) },
    ];

    /// Which enemy the script drives, and where the player stands.
    #[derive(Resource, Default)]
    struct Stage {
        subject: Option<usize>,
        /// Two more enemies from the same team, for the §3 multi-threat
        /// frames. `None` if the roster is too small — which is a
        /// reported inconclusive, not a silent blank frame.
        extras: [Option<usize>; 2],
        home: Option<[f32; 3]>,
        cover_spot: Option<[f32; 3]>,
        t: f32,
        /// Last time the "no open line" warning fired, so a staging
        /// failure reports once a second rather than 144 times.
        warn_t: f32,
    }

    pub(super) fn register(app: &mut App) {
        app.init_resource::<Stage>()
            .add_systems(Update, drive.run_if(in_state(GameState::Playing)));
    }

    /// A point at roughly `bearing` from `home` whose eye-to-chest line
    /// is genuinely open. Falls back to the nominal range if the whole
    /// search fails, and says so — an inconclusive frame that announces
    /// itself is worth far more than one that looks like a bug report.
    fn open_spot(
        s: &sim::TdmSim,
        home: [f32; 3],
        bearings: &[f32],
        subject_h: f32,
        chest: [f32; 3],
    ) -> [f32; 3] {
        for b in bearings {
            if let Some(p) = open_spot_at(s, home, *b, subject_h, chest) {
                return p;
            }
        }
        eprintln!(
            "[threat_sensor capture] no OPEN line at any of {bearings:?} - \
             that frame is INCONCLUSIVE"
        );
        let b = bearings.first().copied().unwrap_or(0.0);
        [
            home[0] + b.sin() * OPEN_RANGES_M[0],
            home[1],
            home[2] + b.cos() * OPEN_RANGES_M[0],
        ]
    }

    fn open_spot_at(
        s: &sim::TdmSim,
        home: [f32; 3],
        bearing: f32,
        subject_h: f32,
        chest: [f32; 3],
    ) -> Option<[f32; 3]> {
        let clear = |x: f32, z: f32| {
            !s.cover.iter().any(|a| {
                x >= a.min[0] - OPEN_CLEAR_M
                    && x <= a.max[0] + OPEN_CLEAR_M
                    && z >= a.min[2] - OPEN_CLEAR_M
                    && z <= a.max[2] + OPEN_CLEAR_M
            })
        };
        for j in OPEN_JITTER {
            for d in OPEN_RANGES_M {
                let a = bearing + j;
                let p = [home[0] + a.sin() * d, home[1], home[2] + a.cos() * d];
                let eye = eye_point(p, subject_h);
                if p[0].abs() < s.half - 2.0
                    && p[2].abs() < s.half - 2.0
                    && clear(p[0], p[2])
                    && s.los_clear(eye, chest)
                {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Pins the whole scene every frame. Writes sim fields, which is
    /// exactly what `capture_quick_deploy` already does for hull, ammo
    /// and position — and like that code it is gated on `JK_CAPTURE`
    /// naming THIS script, so it can never run for a human.
    fn drive(cap: Res<CaptureMode>, mut game: ResMut<Game>, mut st: ResMut<Stage>) {
        if cap.script.as_deref() != Some(SCRIPT) {
            return;
        }
        // THE SAME CLOCK THE BEATS USE. §3's first run drove this off an
        // accumulator that started when `drive` first ran in `Playing`,
        // while `CapBeat` times are measured from `CaptureMode::t` —
        // which starts about a second earlier, during load. Every frame
        // therefore photographed the stage phase BEFORE the one it named,
        // and the 0.6 s duck (frame 07) is far shorter than the skew, so
        // it landed in the reacquire instead and photographed an empty
        // ring that read exactly like a broken grace period. One clock.
        st.t = cap.t;
        let s = &mut game.sim;
        let player = s.player;

        // ---- first frame: plant the player in the clear and find the
        // one enemy this script drives.
        if st.home.is_none() {
            let stage = crate::capture_stage_pos(s);
            st.home = Some(stage);
            let my_team = s.fighters[player].team;
            let mut enemies = s
                .fighters
                .iter()
                .enumerate()
                .filter(|(i, f)| *i != player && f.team != my_team)
                .map(|(i, _)| i);
            st.subject = enemies.next();
            st.extras = [enemies.next(), enemies.next()];
            if st.extras[1].is_none() {
                eprintln!(
                    "[threat_sensor capture] fewer than three enemies on the roster - \
                     frames 08 and 09 (multi-threat) are INCONCLUSIVE"
                );
            }
            // A point at 8-18 m whose line to the player's chest is
            // genuinely blocked. Searched, not asserted: a literal would
            // be a claim about a randomised map. Same two helpers the
            // sensor itself uses, so "blocked" means blocked to IT.
            let chest = chest_point(stage, s.fighters[player].height());
            let sh = st
                .subject
                .map_or(1.8, |j| s.fighters[j].height());
            'search: for ring in [10.0_f32, 13.0, 16.0, 8.0, 19.0] {
                for k in 0..24 {
                    let a = k as f32 / 24.0 * std::f32::consts::TAU;
                    let p = [stage[0] + a.sin() * ring, stage[1], stage[2] + a.cos() * ring];
                    if !s.los_clear(eye_point(p, sh), chest) {
                        st.cover_spot = Some(p);
                        break 'search;
                    }
                }
            }
            if st.cover_spot.is_none() {
                eprintln!(
                    "[threat_sensor capture] no blocked line found near the stage - \
                     frame 01 is INCONCLUSIVE, not a proof of the LOS gate"
                );
            }
        }
        let home = st.home.unwrap_or([0.0, 0.0, 0.0]);

        // ---- the player: pinned, upright, and unkillable for 9 seconds
        {
            let p = &mut s.fighters[player];
            p.pos = home;
            p.vel = [0.0, 0.0];
            p.health = p.health.max(100.0);
        }

        // ---- everyone else on the other team: parked in the far corner,
        // beyond DETECT_RADIUS_M, so exactly one reading is on the ring.
        let my_team = s.fighters[player].team;
        let far = s.half - 2.0;
        let park = [
            if home[0] > 0.0 { -far } else { far },
            0.0,
            if home[2] > 0.0 { -far } else { far },
        ];
        let subject = st.subject;
        let extras = st.extras;
        for (i, f) in s.fighters.iter_mut().enumerate() {
            if i == player
                || f.team == my_team
                || Some(i) == subject
                || extras.contains(&Some(i))
            {
                continue;
            }
            f.pos = park;
            f.vel = [0.0, 0.0];
            f.engaging = -1;
        }

        // ---- the subject
        let Some(j) = subject else { return };
        let mut cur = STAGE[0];
        for e in STAGE {
            if st.t >= e.0 {
                cur = *e;
            }
        }
        let subject_h = s.fighters[j].height();
        let chest = chest_point(home, s.fighters[player].height());
        let pos = match cur.1 {
            Where::BehindCover => st.cover_spot.unwrap_or(park),
            Where::Open(bearing) => open_spot(s, home, bearing, subject_h, chest),
            // half the square wave behind the post, half in the open
            Where::Flicker => {
                if ((st.t * FLICKER_HZ) as i32) % 2 == 0 {
                    st.cover_spot.unwrap_or(park)
                } else {
                    open_spot(s, home, AHEAD, subject_h, chest)
                }
            }
        };
        let to_player = (home[0] - pos[0], home[2] - pos[2]);
        let at_player = to_player.0.atan2(to_player.1);
        // A frame that MEANT to show a reading and cannot must say so.
        // `open_spot` verifies the CANDIDATE point, but the sim's
        // collision push-out then moves the body, and from where he
        // actually ended up last frame the line can be blocked. The
        // sensor is right to draw nothing; the STAGE is what failed, and
        // an empty ring photographs identically either way — so it is
        // announced rather than left to be read as a bug in the feature.
        let actual = s.fighters[j].pos;
        if !matches!(cur.1, Where::BehindCover | Where::Flicker)
            && !s.los_clear(eye_point(actual, subject_h), chest)
            && st.t - st.warn_t > 1.0
        {
            st.warn_t = st.t;
            eprintln!(
                "[threat_sensor capture] t={:.1}: the staged subject has NO open line \
                 from where the sim actually left him - any frame here is INCONCLUSIVE, \
                 not a negative result",
                st.t
            );
        }
        if std::env::var("JK_THREAT_DEBUG").is_ok() {
            let was = actual;
            eprintln!(
                "[threat/stage] t={:.1} want={pos:?} actual={was:?} los={} alive={}",
                st.t,
                s.los_clear(eye_point(was, subject_h), chest),
                s.fighters[j].alive()
            );
        }
        let f = &mut s.fighters[j];
        f.pos = pos;
        f.vel = [0.0, 0.0];
        // The first run lost him to a teammate's rifle at t=6.4 and the
        // last two frames photographed a corpse's empty ring. A staged
        // subject that can die is not a stage.
        f.health = f.health.max(100.0);
        f.yaw = match cur.2 {
            Face::Away => at_player + std::f32::consts::PI,
            Face::AtPlayer | Face::AtPlayerEngaged => at_player,
        };
        f.engaging = match cur.2 {
            Face::AtPlayerEngaged => player as i32,
            _ => -1,
        };

        // ---- §3: the other two bodies.
        let mut mode = EXTRAS[0].1;
        for e in EXTRAS {
            if st.t >= e.0 {
                mode = e.1;
            }
        }
        for (k, ex) in extras.iter().enumerate() {
            let Some(x) = *ex else { continue };
            let xh = s.fighters[x].height();
            // `Separated` only wants ONE extra on the ring; the second
            // stays parked so frame 08 is unambiguously two markers.
            let active = match mode {
                Extra::Parked => false,
                Extra::Separated => k == 0,
                Extra::CoBearing => true,
            };
            if !active {
                let f = &mut s.fighters[x];
                f.pos = park;
                f.vel = [0.0, 0.0];
                f.engaging = -1;
                continue;
            }
            let xpos = match mode {
                // far enough round the ring that no merge threshold
                // could swallow it
                Extra::Separated => open_spot(s, home, LATERAL, xh, chest),
                // shoulder to shoulder with the subject
                Extra::CoBearing => beside(s, pos, home, SIDE_OFFSETS_M[k * 2], xh, chest),
                Extra::Parked => park,
            };
            let to_p = (home[0] - xpos[0], home[2] - xpos[2]);
            let f = &mut s.fighters[x];
            f.pos = xpos;
            f.vel = [0.0, 0.0];
            f.health = f.health.max(100.0);
            f.yaw = to_p.0.atan2(to_p.1);
            // Frame 08: the subject has the clear shot, the lateral man
            // is merely aiming — the arc and the no-arc marker in one
            // photograph. Frame 09: the SUBJECT is only aiming and the
            // first man beside him has the clear shot, so a merged
            // marker that showed the subject's state would show the
            // wrong one. That is the assertion the frame makes.
            f.engaging = match (mode, k) {
                (Extra::CoBearing, 0) => player as i32,
                _ => -1,
            };
        }
    }

    /// A body tucked in beside `pos`, `off` metres along the tangent, on
    /// a line that is still open. Falls back through `SIDE_OFFSETS_M`
    /// and finally onto `pos` itself — a co-bearing frame with two
    /// bodies at one point still photographs a merge.
    fn beside(
        s: &sim::TdmSim,
        pos: [f32; 3],
        home: [f32; 3],
        first: f32,
        h: f32,
        chest: [f32; 3],
    ) -> [f32; 3] {
        let (dx, dz) = (pos[0] - home[0], pos[2] - home[2]);
        let len = (dx * dx + dz * dz).sqrt().max(1e-3);
        // tangent to the player->subject line, so the offset moves
        // BEARING and not range
        let (tx, tz) = (-dz / len, dx / len);
        let mut order = [first; SIDE_OFFSETS_M.len()];
        order[1..].copy_from_slice(&SIDE_OFFSETS_M[..SIDE_OFFSETS_M.len() - 1]);
        for o in order {
            let p = [pos[0] + tx * o, pos[1], pos[2] + tz * o];
            if p[0].abs() < s.half - 2.0
                && p[2].abs() < s.half - 2.0
                && s.los_clear(eye_point(p, h), chest)
            {
                return p;
            }
        }
        pos
    }
}

// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blocked_line_is_never_a_threat_whatever_he_is_pointing_at() {
        // dead-on aim, engaging me, in range - but no line
        assert_eq!(
            classify(true, false, 1.0, true),
            ThreatState::NotDetected,
            "LOS is the gate; nothing above it may fire without it"
        );
        // and out of range is dead too, even with a clear line
        assert_eq!(classify(false, true, 1.0, true), ThreatState::NotDetected);
    }

    #[test]
    fn the_ladder_climbs_in_the_owners_order() {
        // looking away: line open, outside the vision cone
        let away = classify(true, true, (110.0_f32).to_radians().cos(), false);
        assert_eq!(away, ThreatState::Visible);
        // inside vision, outside aim
        let seen = classify(true, true, (30.0_f32).to_radians().cos(), false);
        assert_eq!(seen, ThreatState::Tracking);
        // inside aim, not his chosen target
        let aimed = classify(true, true, (4.0_f32).to_radians().cos(), false);
        assert_eq!(aimed, ThreatState::Aiming);
        // inside aim AND the sim says I am the target
        let shot = classify(true, true, (4.0_f32).to_radians().cos(), true);
        assert_eq!(shot, ThreatState::ClearShot);
        assert!(
            away.value() < seen.value() && seen.value() < aimed.value()
                && aimed.value() < shot.value()
        );
    }

    #[test]
    fn the_threat_values_are_the_owners_quarters() {
        assert_eq!(ThreatState::NotDetected.value(), 0.0);
        assert_eq!(ThreatState::Visible.value(), 0.25);
        assert_eq!(ThreatState::Tracking.value(), 0.50);
        assert_eq!(ThreatState::Aiming.value(), 0.75);
        assert_eq!(ThreatState::ClearShot.value(), 1.00);
        assert!(!ThreatState::NotDetected.detected());
        assert!(ThreatState::Visible.detected());
    }

    #[test]
    fn the_cone_boundary_is_exactly_where_the_degree_constants_put_it() {
        let just_in = classify(true, true, (VISION_HALF_ANGLE_DEG - 1.0).to_radians().cos(), false);
        let just_out = classify(true, true, (VISION_HALF_ANGLE_DEG + 1.0).to_radians().cos(), false);
        assert_eq!(just_in, ThreatState::Tracking);
        assert_eq!(just_out, ThreatState::Visible);
        let aim_in = classify(true, true, (AIM_HALF_ANGLE_DEG - 1.0).to_radians().cos(), false);
        let aim_out = classify(true, true, (AIM_HALF_ANGLE_DEG + 1.0).to_radians().cos(), false);
        assert_eq!(aim_in, ThreatState::Aiming);
        assert_eq!(aim_out, ThreatState::Tracking);
        assert!(AIM_HALF_ANGLE_DEG < VISION_HALF_ANGLE_DEG, "aim must be the tighter cone");
    }

    #[test]
    fn a_brief_los_break_does_not_flicker_the_light() {
        // fully faded in, grace full
        let (mut fade, mut hold) = (1.0_f32, GRACE_S);
        // eight frames at 60 Hz with NO positive re-evaluation: well
        // inside the grace, so presence must not move at all.
        for _ in 0..8 {
            let r = fade_step(fade, hold, 1.0 / 60.0);
            fade = r.0;
            hold = r.1;
        }
        assert_eq!(fade, 1.0, "an LOS break of 8 frames dimmed the marker");
        assert!(hold > 0.0);
        // and once the grace really is spent it does fade, but not
        // instantly
        for _ in 0..60 {
            let r = fade_step(fade, hold, 1.0 / 60.0);
            fade = r.0;
            hold = r.1;
        }
        assert!(fade < 1.0 && fade > 0.0, "fade {fade} is a toggle, not a fade");
    }

    #[test]
    fn an_enemy_strafing_behind_a_post_never_flickers_the_marker() {
        // The spec's hardest case, and the one most likely to be broken:
        // he ducks in and out of cover at 5 Hz for two seconds. Presence
        // must sit at 1.0 the whole time — not "mostly", not "recovers
        // quickly". Every 0.1 s duck is far inside GRACE_S, so every
        // re-acquisition recharges the hold before it can drain.
        let dt = 1.0 / 60.0;
        let (mut fade, mut hold) = (1.0_f32, GRACE_S);
        let mut worst = 1.0_f32;
        for k in 0..120 {
            let t = k as f32 * dt;
            // 5 Hz square wave: visible for half of every 0.2 s
            let visible = ((t * 5.0) as i32) % 2 == 0;
            if visible {
                hold = GRACE_S;
            }
            let r = fade_step(fade, hold, dt);
            fade = r.0;
            hold = r.1;
            worst = worst.min(fade);
        }
        assert_eq!(worst, 1.0, "a 5 Hz strafe behind cover dimmed the marker to {worst}");
        // sanity that the wave really did break the line: with NO
        // re-acquisition over the same two seconds the marker does die,
        // so the test above is not passing because nothing was tested.
        let (mut f2, mut h2) = (1.0_f32, GRACE_S);
        for _ in 0..120 {
            let r = fade_step(f2, h2, dt);
            f2 = r.0;
            h2 = r.1;
        }
        assert_eq!(f2, 0.0);
    }

    #[test]
    fn the_grace_outlasts_the_worst_case_stagger_gap() {
        // The stagger's own promise: with a full roster, this is how long
        // a track can wait between evaluations. If GRACE_S were shorter,
        // a permanently visible enemy would blink at the sensor's period.
        let gap = (MAX_TRACKS as f32 / STAGGER_PER_TICK as f32).ceil() * DETECT_INTERVAL_S;
        assert!(GRACE_S > gap, "grace {GRACE_S}s does not cover the {gap}s stagger gap");
    }

    #[test]
    fn presence_fades_in_and_all_the_way_out() {
        let (mut fade, mut hold) = (0.0_f32, GRACE_S);
        for _ in 0..(60.0 * FADE_IN_S) as usize + 4 {
            let r = fade_step(fade, hold, 1.0 / 60.0);
            fade = r.0;
            hold = GRACE_S.max(r.1); // as if re-detected each tick
        }
        assert_eq!(fade, 1.0);
        hold = 0.0;
        for _ in 0..(60.0 * (FADE_OUT_S + 0.2)) as usize {
            let r = fade_step(fade, hold, 1.0 / 60.0);
            fade = r.0;
            hold = r.1;
        }
        assert_eq!(fade, 0.0, "a track that never reaches 0 never frees its slot");
    }

    #[test]
    fn the_value_interpolates_rather_than_snapping() {
        let dt = 1.0 / 60.0;
        let one = approach(0.0, 1.0, dt, VALUE_TAU_S);
        assert!(one > 0.0 && one < 0.5, "a single frame jumped {one} of the way");
        let mut v = 0.0;
        for _ in 0..60 {
            v = approach(v, 1.0, dt, VALUE_TAU_S);
        }
        assert!(v > 0.99, "one second was not enough to arrive: {v}");
    }

    #[test]
    fn the_bearing_is_zero_dead_ahead_and_positive_to_the_right() {
        let fwd = Vec2::new(0.0, 1.0);
        let right = Vec2::new(1.0, 0.0);
        assert!(screen_bearing(fwd, right, Vec2::new(0.0, 10.0)).abs() < 1e-5);
        let r = screen_bearing(fwd, right, Vec2::new(10.0, 0.0));
        assert!((r - std::f32::consts::FRAC_PI_2).abs() < 1e-5, "right read as {r}");
        let l = screen_bearing(fwd, right, Vec2::new(-10.0, 0.0));
        assert!((l + std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        let b = screen_bearing(fwd, right, Vec2::new(0.0, -10.0)).abs();
        assert!((b - std::f32::consts::PI).abs() < 1e-4, "behind read as {b}");
    }

    #[test]
    fn the_marker_sits_on_the_ring_on_the_correct_side() {
        let r = 100.0;
        let ahead = ring_offset(0.0, r);
        assert!(ahead.y < -99.0 && ahead.x.abs() < 1e-4, "ahead is not at the top: {ahead}");
        let right = ring_offset(std::f32::consts::FRAC_PI_2, r);
        assert!(right.x > 99.0 && right.y.abs() < 1e-4, "right is not to the right: {right}");
        let behind = ring_offset(std::f32::consts::PI, r);
        assert!(behind.y > 99.0, "behind is not at the bottom: {behind}");
        // it IS a ring: every bearing lands the same distance out, so a
        // marker can never leave the frame the ring fits inside.
        for k in 0..32 {
            let a = k as f32 / 32.0 * std::f32::consts::TAU;
            assert!((ring_offset(a, r).length() - r).abs() < 1e-3);
        }
        // and the pips march the same way the marker sits
        for k in 0..8 {
            let a = k as f32 / 8.0 * std::f32::consts::TAU;
            let d = radial_dir(a);
            assert!((d.length() - 1.0).abs() < 1e-4);
            assert!(d.dot(ring_offset(a, r).normalize()) > 0.999, "pips march inward");
        }
    }

    #[test]
    fn the_ring_never_crosses_the_centre_of_the_screen() {
        // "must not obscure the world": at 16:9 the ring's radius is a
        // large fraction of the short axis, so no marker can sit in the
        // middle third of the frame in the vertical axis.
        let (w, h) = (1280.0_f32, 720.0_f32);
        let radius = w.min(h) * 0.5 * RING_FRAC;
        assert!(radius > h / 6.0, "the ring has collapsed into the sight picture");
        assert!(radius <= h * 0.5, "the ring has left the frame");
    }

    #[test]
    fn intensity_climbs_with_threat_and_never_blinks_fully_out() {
        // sampled across a pulse period at each rung: the WEAKEST moment
        // of a high threat must still beat the STRONGEST moment of a low
        // one, or the ladder is not readable.
        let sample = |v: f32, f: fn(f32) -> f32| {
            let mut lo = f32::INFINITY;
            let mut hi: f32 = 0.0;
            for k in 0..200 {
                let a = marker_alpha(1.0, v, k as f32 * 0.01);
                lo = lo.min(a);
                hi = hi.max(a);
            }
            let _ = f;
            (lo, hi)
        };
        let id = |x: f32| x;
        let (lo_v, hi_v) = sample(ThreatState::Visible.value(), id);
        let (lo_s, _) = sample(ThreatState::ClearShot.value(), id);
        assert!(lo_v > 0.0, "the faintest marker blinks fully out");
        assert!(lo_s > hi_v, "a clear shot at its dimmest ({lo_s}) is not louder than a contact at its brightest ({hi_v})");
        // presence scales it all the way to nothing
        assert_eq!(marker_alpha(0.0, 1.0, 0.3), 0.0);
        // and nothing ever saturates
        assert!(marker_alpha(1.0, 1.0, 0.0) <= 1.0);
    }

    #[test]
    fn a_stronger_threat_lights_more_pips_and_a_bigger_head() {
        assert_eq!(lit_pips(ThreatState::Visible.value()), 1);
        assert_eq!(lit_pips(ThreatState::Tracking.value()), 2);
        assert_eq!(lit_pips(ThreatState::Aiming.value()), 3);
        assert_eq!(lit_pips(ThreatState::ClearShot.value()), 3);
        assert!(lit_pips(1.0) <= MAX_PIPS);
        assert!(
            pip_size(0, 1.0) > pip_size(0, 0.25),
            "the head pip does not grow with threat"
        );
        // the taper is what points: each pip out is smaller than the last
        for v in [0.25_f32, 0.5, 1.0] {
            for k in 1..MAX_PIPS {
                assert!(pip_size(k, v) < pip_size(k - 1, v), "pip {k} does not taper");
            }
            assert!(pip_size(MAX_PIPS - 1, v) > 0.0);
        }
    }

    // -----------------------------------------------------------------
    // §3B — clustering
    // -----------------------------------------------------------------

    fn blip(deg: f32, st: ThreatState) -> Blip {
        Blip {
            bearing: deg.to_radians(),
            value: st.value(),
            state: st,
            fade: 1.0,
        }
    }

    #[test]
    fn well_separated_threats_stay_separate_markers() {
        let b = [
            blip(0.0, ThreatState::ClearShot),
            blip(90.0, ThreatState::Aiming),
            blip(-140.0, ThreatState::Visible),
        ];
        let mut out = [Cluster::default(); MAX_TRACKS];
        let n = cluster_blips(&b, &mut out);
        assert_eq!(n, 3, "three bearings a quadrant apart collapsed into {n}");
        for c in out.iter().take(n) {
            assert_eq!(c.count, 1);
        }
    }

    #[test]
    fn co_bearing_threats_merge_into_one_marker_showing_the_strongest() {
        // three men within a few degrees; the WEAKEST is listed first,
        // so a clusterer that simply kept the first arrival would show
        // "possible contact" while a rifle was on the player.
        let b = [
            blip(2.0, ThreatState::Visible),
            blip(-3.0, ThreatState::ClearShot),
            blip(5.0, ThreatState::Tracking),
        ];
        let mut out = [Cluster::default(); MAX_TRACKS];
        let n = cluster_blips(&b, &mut out);
        assert_eq!(n, 1, "{n} markers for one direction");
        assert_eq!(out[0].count, 3);
        assert_eq!(
            out[0].state,
            ThreatState::ClearShot,
            "the merged marker must speak for the worst man in the group"
        );
        assert_eq!(out[0].value, ThreatState::ClearShot.value());
        // and it points at HIM, not at the mean of the three
        assert!(
            (out[0].bearing - (-3.0_f32).to_radians()).abs() < 1e-5,
            "the merged marker drifted off the leader's bearing"
        );
    }

    #[test]
    fn the_merge_threshold_is_the_named_const_and_nothing_else() {
        let mut out = [Cluster::default(); MAX_TRACKS];
        // just inside: one marker
        let inside = [
            blip(0.0, ThreatState::Aiming),
            blip(MERGE_SEPARATION_DEG - 1.0, ThreatState::Visible),
        ];
        assert_eq!(cluster_blips(&inside, &mut out), 1);
        // just outside: two
        let outside = [
            blip(0.0, ThreatState::Aiming),
            blip(MERGE_SEPARATION_DEG + 1.0, ThreatState::Visible),
        ];
        assert_eq!(cluster_blips(&outside, &mut out), 2);
    }

    #[test]
    fn two_men_either_side_of_dead_astern_are_one_marker_not_two() {
        // the wrap case: +179 and -179 are two degrees apart. A naive
        // subtraction calls them 358 apart and draws two markers on top
        // of each other at the bottom of the ring.
        assert!(ang_delta(179.0_f32.to_radians(), (-179.0_f32).to_radians()).abs() < 0.05);
        let b = [
            blip(179.0, ThreatState::Tracking),
            blip(-179.0, ThreatState::ClearShot),
        ];
        let mut out = [Cluster::default(); MAX_TRACKS];
        assert_eq!(cluster_blips(&b, &mut out), 1, "the bearing wrap is unhandled");
        assert_eq!(out[0].state, ThreatState::ClearShot);
    }

    #[test]
    fn a_crowd_never_spams_more_markers_than_there_are_slots() {
        // sixteen enemies spread evenly: every gap is 22.5 deg, wider
        // than the threshold, so this is the worst case for marker
        // count - and it must still fit the slot table.
        let mut b = [Blip::default(); MAX_TRACKS];
        for (k, x) in b.iter_mut().enumerate() {
            *x = blip(k as f32 * 360.0 / MAX_TRACKS as f32 - 180.0, ThreatState::Tracking);
        }
        let mut out = [Cluster::default(); MAX_TRACKS];
        let n = cluster_blips(&b, &mut out);
        assert!(n <= MAX_TRACKS, "{n} markers for {MAX_TRACKS} slots");
        assert_eq!(out.iter().take(n).map(|c| c.count).sum::<usize>(), MAX_TRACKS,
            "clustering lost or invented an enemy");
        // and sixteen men in ONE doorway are one light, not sixteen
        let same = [blip(30.0, ThreatState::Visible); MAX_TRACKS];
        assert_eq!(cluster_blips(&same, &mut out), 1);
        assert_eq!(out[0].count, MAX_TRACKS);
    }

    #[test]
    fn the_tally_is_a_hint_and_stays_capped() {
        assert_eq!(tally_dots(1), 0, "a lone contact drew a tally dot");
        assert_eq!(tally_dots(2), 1);
        assert_eq!(tally_dots(0), 0);
        assert_eq!(tally_dots(99), TALLY_MAX_DOTS);
        assert!(TALLY_DOT_PX < PIP_MIN_PX, "the tally is louder than the marker");
    }

    // -----------------------------------------------------------------
    // §3A — the clear-shot arc
    // -----------------------------------------------------------------

    #[test]
    fn the_arc_is_a_clear_shot_and_nothing_below_it() {
        assert!(wants_arc(ThreatState::ClearShot));
        for s in [
            ThreatState::NotDetected,
            ThreatState::Visible,
            ThreatState::Tracking,
            ThreatState::Aiming,
        ] {
            assert!(!wants_arc(s), "{s:?} drew the clear-shot arc");
        }
    }

    #[test]
    fn the_arc_is_a_continuous_bar_on_the_ring_behind_the_marker() {
        let radius = 259.0_f32; // the 1280x720 ring
        let bearing = 0.7_f32;
        let marker = ring_offset(bearing, radius);
        let mut prev: Option<Vec2> = None;
        let mut span_lo = f32::INFINITY;
        let mut span_hi = f32::NEG_INFINITY;
        for j in 0..ARC_SEGMENTS {
            let p = arc_offset(j, bearing, radius) + marker;
            // every block sits on a ring INSIDE the marker's own, so it
            // cannot cover the pips that march outward from it
            let r = p.length();
            assert!(
                (r - (radius - ARC_INSET_PX)).abs() < 1e-2,
                "arc block {j} is at radius {r}, not on the inner ring"
            );
            // and the blocks OVERLAP: a gap turns the bar into a dotted
            // line, which is exactly the kind of faint difference the
            // arc exists to avoid.
            if let Some(q) = prev {
                let step = (p - q).length();
                assert!(
                    step < ARC_THICK_PX,
                    "arc blocks are {step:.1} px apart but only {ARC_THICK_PX} px wide - dotted"
                );
            }
            prev = Some(p);
            let a = ang_delta(p.x.atan2(-p.y), bearing);
            span_lo = span_lo.min(a);
            span_hi = span_hi.max(a);
        }
        let span = (span_hi - span_lo).to_degrees();
        assert!(
            (span - ARC_SPAN_DEG).abs() < 1.0,
            "the arc spans {span} deg, not the {ARC_SPAN_DEG} the const asks for"
        );
        // it is centred on the marker
        assert!((span_hi + span_lo).abs() < 1e-3, "the arc is not centred on its marker");
    }

    #[test]
    fn the_arc_is_a_bigger_signal_than_the_step_from_aiming_to_clear_shot_ever_was() {
        // The whole reason §3A exists: in a STILL, `Aiming` and
        // `ClearShot` differed only by alpha, head size and a pulse RATE
        // that a photograph cannot show. Pin how small that difference
        // is, so nobody deletes the arc believing the old cues were
        // enough.
        let (a, c) = (ThreatState::Aiming.value(), ThreatState::ClearShot.value());
        assert_eq!(lit_pips(a), lit_pips(c), "this test's premise has changed");
        let d_size = pip_size(0, c) - pip_size(0, a);
        assert!(d_size < 3.0, "pip size alone now separates them ({d_size} px)");
        // the arc, by contrast, is either a solid bar or nothing
        assert!(wants_arc(ThreatState::ClearShot) && !wants_arc(ThreatState::Aiming));
        assert!(ARC_SEGMENTS >= 8 && ARC_THICK_PX >= 4.0);
        assert!(ARC_ALPHA > ALPHA_MAX * (1.0 - PULSE_DEPTH), "the arc can be dimmer than a pip");
    }

    #[test]
    fn the_pulse_gets_faster_as_the_threat_rises() {
        // count zero crossings of the wave over one second at each rung
        let crossings = |v: f32| {
            let mut n = 0;
            let mut prev = marker_alpha(1.0, v, 0.0);
            for k in 1..1000 {
                let a = marker_alpha(1.0, v, k as f32 * 0.001);
                if (a - prev).abs() > 1e-6 && ((a > prev) != (prev > marker_alpha(1.0, v, (k as f32 - 2.0) * 0.001))) {
                    n += 1;
                }
                prev = a;
            }
            n
        };
        assert!(
            crossings(1.0) > crossings(0.25),
            "the clear-shot pulse is no faster than the contact pulse"
        );
    }
}
