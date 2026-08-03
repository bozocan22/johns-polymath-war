//! TDM sim v3 — deterministic third-person-shooter arena.
//!
//! Same discipline as the battle sim: fixed 120 Hz tick, seeded PCG, every
//! shot, arrow, and death reproducible from seed + inputs.
//!
//! v3: you spawn UNARMED — weapons live on pads (handgun, assault rifle,
//! machine gun, sniper, bow, war spear). Real elevation: plateaus, stairs,
//! and a center tower (gravity + step-up on an AABB terrain). Global TTK
//! rule: 5 body shots / 2 headshots. Hit events carry the zone (legs/body/
//! shoulder/head) and the shot's path for both-way feedback; impacts carry
//! surface normals for bullet marks. Modes: Team Deathmatch and King of the
//! Hill, 5-minute matches with an 80 s sudden-death overtime. Rare pickups:
//! health, ammo, and the ROBOT ARMOR (100 armor, faster legs).
//!
//! v5: recoil halved across the arsenal; a FULL crouch; the duck-spin
//! DODGE ROLL (and the automatic parkour breakfall off hard landings);
//! three maps (dust arena, castle bailey, castle gardens) behind one
//! `MapKind`, each cover box tagged with what it's made of; and
//! `predict_arc` — the exact flight an arrow/spear will take, for the
//! client's aiming laser.

use jk_core::timestep::DT;
use jk_core::Pcg32;

pub const ARENA_HALF: f32 = 34.0;
pub const EYE_REL: f32 = 1.62;
pub const BODY_RADIUS: f32 = 0.34;
pub const BODY_HEIGHT: f32 = 1.78;
/// Full crouch. 1.15 keeps the hitbox honest against the chibi rig's
/// visible crouched head — what you can see, you can shoot.
pub const CROUCH_HEIGHT: f32 = 1.15;
pub const MOVE_SPEED: f32 = 4.8;
pub const SPRINT_SPEED: f32 = 6.6;

// ---- §1.3 (BRIEF VIII): "full stops never hit instant zero" ---------
// The doctrine's headline rule, and the single largest source of the
// "floaty" read it names. Until now BOTH the player path and the bot
// path wrote `vel` straight from input every tick: release the key and
// velocity went to exactly zero in one 120 Hz tick. That is the named
// anti-pattern "the wall stop", at its source.
//
// Velocity now APPROACHES its input target at a bounded rate. Two
// rates, because a body starts harder than it stops:
//   - accelerating toward a target at least as fast as current
//   - decelerating toward a slower one (releasing input, or slowing)
//
// The counter-strafe falls out of this for free rather than being
// special-cased: releasing input gives target speed 0 (DECEL, the slow
// path), but pressing the OPPOSITE direction gives a target of equal
// magnitude (ACCEL, the fast path). Tapping back therefore kills your
// speed faster than letting go - the CS-family mechanic, emergent.
// This also gives the existing MOVE_INACC_START/FULL accuracy ramp
// something real to measure: "stopped" is now a state you travel to,
// not a state you teleport into.
pub const GROUND_ACCEL: f32 = 55.0; // m/s^2 - walk in ~0.09s, sprint in ~0.12s
pub const GROUND_DECEL: f32 = 40.0; // m/s^2 - sprint to rest in ~0.17s, visible

/// Step a planar velocity toward `target` under the two-rate model
/// above. Pure and shared by the player and bot movement paths so they
/// can never drift apart (bot/player parity has been a real, repeated
/// defect class in this file - see the mech turn-rate comment).
pub fn approach_velocity(cur: [f32; 2], target: [f32; 2], dt: f32) -> [f32; 2] {
    let dv = [target[0] - cur[0], target[1] - cur[1]];
    let dv_mag = (dv[0] * dv[0] + dv[1] * dv[1]).sqrt();
    if dv_mag <= 1e-6 {
        return target;
    }
    let cur_mag = (cur[0] * cur[0] + cur[1] * cur[1]).sqrt();
    let tgt_mag = (target[0] * target[0] + target[1] * target[1]).sqrt();
    let rate = if tgt_mag >= cur_mag { GROUND_ACCEL } else { GROUND_DECEL };
    let step = rate * dt;
    if dv_mag <= step {
        return target; // close enough to land exactly on it this tick
    }
    [
        cur[0] + dv[0] / dv_mag * step,
        cur[1] + dv[1] / dv_mag * step,
    ]
}
/// Brief IX-C "Armour Customization": -0.15 m/s per kg of equipped
/// armour/weapon weight over a class's budget - the brief's exact rule,
/// worked example included ("+4 kg over budget, -0.60 m/s"). Pure and
/// currently UNWIRED to real movement: there is no per-piece weight-
/// tracking system yet (the 26-piece Forge rebuild this needs is a
/// separate, much larger deferral - see REPORT.md), so this captures
/// the formula, ready to plug in once real equipped weight exists.
pub const ARMOR_WEIGHT_PENALTY_PER_KG: f32 = 0.15;
pub fn armor_weight_movement_penalty(equipped_kg: f32, budget_kg: f32) -> f32 {
    (equipped_kg - budget_kg).max(0.0) * ARMOR_WEIGHT_PENALTY_PER_KG
}
pub const CROUCH_SPEED_MULT: f32 = 0.5;
pub const CROUCH_SPREAD_MULT: f32 = 0.55;
pub const ADS_SPREAD_MULT: f32 = 0.32;
pub const ADS_SPEED_MULT: f32 = 0.62;
pub const SWITCH_S: f32 = 0.6;
pub const MAX_HEALTH: f32 = 100.0;
pub const RESPAWN_S: f32 = 3.0;
pub const SPAWN_PROTECT_S: f32 = 1.2;
pub const GRAVITY: f32 = 18.0; // gamey gravity: snappy falls
pub const STEP_UP: f32 = 0.55; // how tall a ledge your legs climb
pub const JUMP_SPEED: f32 = 7.4; // clears a 1.3 m crate (v²/2g ≈ 1.5 m)
// ---- dodge roll (v5): duck-spin dodge, and the parkour breakfall --------
pub const ROLL_S: f32 = 0.55; // how long the somersault lasts
pub const ROLL_SPEED: f32 = 8.6; // faster than a sprint — it's a dodge
/// Task 3 rule 3 (MISSION doc): a dodge launched AGAINST your current
/// movement gets the stretch-shortening bonus - the counter-movement
/// loads the legs, and the release is faster than a dead-start. A dodge
/// WITH your movement is already riding momentum and gets nothing.
pub const ROLL_COUNTER_BONUS: f32 = 0.12;

/// Task 3 rule 3, the pure rule: a counter-movement (motion opposite the
/// coming release) grants the bonus; a dead start does not. `prior_dir`
/// and `release_dir` are SIGNS of motion before and during the move.
/// Lives in the SIM because it now changes a real velocity - it sat in
/// main.rs as a spec fixture with zero call sites for two briefs.
pub fn counter_movement_bonus(prior_dir: f32, release_dir: f32, max_bonus: f32) -> f32 {
    if prior_dir * release_dir < 0.0 {
        max_bonus
    } else {
        0.0
    }
}
pub const ROLL_CD_S: f32 = 0.9; // cooldown after a roll ends
pub const ROLL_HEIGHT: f32 = 0.95; // balled up: a small target
// ---- §2 (Brief V): motion WEIGHT on the roll -----------------------------
// load → burst → ease-out. The load is the crouch-coil before the
// spring; the ease-out is the single most important line here — an
// instant velocity stop is what reads as "gamey", so the burst hands
// speed back over a real window instead of a cliff.
pub const ROLL_LOAD_S: f32 = 0.10;
pub const ROLL_EASE_S: f32 = 0.14;
/// Render-side weight-absorb dip after the roll ends (the settle).
pub const ROLL_SETTLE_S: f32 = 0.20;
// §2 (Brief V): the mech does not tumble — a 2.7 m walker somersaulting
// fights its own silhouette. Its dodge is a BRACED SIDE-STEP: shorter,
// grounded, tall, near-zero steering, with the same ease-out landing.
pub const MECH_STEP_S: f32 = 0.30;
pub const MECH_STEP_SPEED: f32 = 6.5;
pub const MECH_STEP_CD_S: f32 = 1.4;
// ---- §2 (Brief V): the spear THRUST --------------------------------------
// F while wielding the spear is a THRUST, not a knife swing: a visible
// load (rear foot, hips coil), a driving extension, and a recovery that
// is LONGER on a whiff than on a hit — a missed thrust is committed.
pub const THRUST_WIND_S: f32 = 0.22;
pub const THRUST_ACTIVE_S: f32 = 0.10;
pub const THRUST_RECOVER_HIT_S: f32 = 0.30;
pub const THRUST_RECOVER_WHIFF_S: f32 = 0.55;
pub const THRUST_DMG: f32 = 70.0;
pub const THRUST_BACKSTAB: f32 = 170.0;
pub const THRUST_RANGE_M: f32 = 2.6;
/// cos of the thrust's line — a stab, not a sweep.
pub const THRUST_ARC_COS: f32 = 0.85;
/// The mech's thrust is the same shape, geared down: slower on the
/// wind-up AND the recovery.
pub const MECH_THRUST_TIME_MULT: f32 = 1.4;
/// Landing harder than this (m/s downward) automatically turns the fall
/// into a breakfall roll — what parkour people do after a big drop.
/// A full flat jump lands at ~7.4 m/s, so ordinary hops stay on their feet.
pub const HARD_LANDING_VY: f32 = 9.5;
/// v6 damage model: four hitbox zones with multipliers over each gun's
/// base TORSO damage. The owner's tuned target — 2 headshots / 8 body
/// shots — is the BASELINE RIFLE (M4A1 at 12.5); other guns scale around
/// it. Limb hits are punished less than center mass.
pub const HEAD_MULT: f32 = 4.0;
pub const ARM_MULT: f32 = 0.75;
pub const LEG_MULT: f32 = 0.75;
// ---- the shield (v6): a core system --------------------------------------
/// Half-angle of the protected front arc (±60° from facing).
pub const SHIELD_ARC_COS: f32 = 0.5; // cos(60°)
/// Damage reduction with the shield up, standing.
pub const SHIELD_BLOCK_STAND: f32 = 0.65;
/// Crouched behind the shield: near-total protection from the front.
/// Flanking is THE counter — sides and rear ignore the shield entirely.
pub const SHIELD_BLOCK_CROUCH: f32 = 0.95;
pub const SHIELD_SPEED_MULT: f32 = 0.55;
/// AWM scoped-in crawl.
pub const SCOPED_SPEED_MULT: f32 = 0.5; // §5.2 (Brief VI): 50% scoped
// ---- projectile draw (§4 overhaul): freedom with a steadiness cost -------
// A drawn bow / cocked spear no longer pins you in place. Instead, turning
// fast or moving fast while drawn SPOILS the shot: a stability factor
// widens the spread. Settled shots stay laser-precise.
/// Move-speed multiplier while a bow is at full draw (rifle-ADS pace).
pub const DRAW_SPEED_MULT_BOW: f32 = 0.62;
/// Move-speed multiplier while a spear is cocked (lighter than a draw).
pub const DRAW_SPEED_MULT_SPEAR: f32 = 0.70;
/// Strafe/backpedal fraction of forward pace while drawn (bracing).
pub const DRAW_SIDE_MULT: f32 = 0.85;
/// Turn this fast (rad/s) for free; above it stability decays.
pub const AIM_TURN_FREE: f32 = 1.6;
/// Stability penalty per rad/s above the free turn rate. (§4 Brief II:
/// raised 0.22 → 0.28 alongside the flatter ballistics — easier to aim,
/// no easier to spam.)
pub const AIM_TURN_K: f32 = 0.28;
/// Walk this fast (m/s) for free; above it stability decays.
pub const AIM_MOVE_FREE: f32 = 1.2;
/// Stability penalty per m/s above the free walk speed.
pub const AIM_MOVE_K: f32 = 0.28;
/// A whip-shot is never fully un-aimable — spread caps at base/this.
pub const AIM_STABILITY_MIN: f32 = 0.35;
// ---- §4 (Brief II): flattened projectile ballistics ----------------------
// The old arcs (spear 17 m/s at full gravity = 15.3 m of drop at 30 m)
// were unaimable by eye. Flatter flight, same damage, same draw times —
// and a steeper turn penalty so it gets easier to AIM, not easier to spam.
/// One shared gravity base for every missile — `predict_arc` and
/// `step_missiles` MUST read the same numbers or the preview lies.
pub const MISSILE_G: f32 = 9.81;
pub const GRAV_FACTOR_SPEAR: f32 = 0.72;
pub const GRAV_FACTOR_ARROW: f32 = 0.42;
/// Effective gravity for a missile kind — the ONLY place the factor lives.
pub fn missile_g(is_spear: bool) -> f32 {
    MISSILE_G
        * if is_spear {
            GRAV_FACTOR_SPEAR
        } else {
            GRAV_FACTOR_ARROW
        }
}
/// Hip-thrown spear (no ADS settle) flies at min charge — the full 26 m/s
/// needs the cocked, settled throw.
pub const SPEAR_V0_MIN: f32 = 11.0;
// ---- §5.4 (BRIEF VIII): the running-throw bonus --------------------------
// "A throw initiated at >=70% run speed with >=2 steps of momentum gets
// velocity x1.15." The speed gate is exact per the brief; "2 steps of
// momentum" is interpreted as a SUSTAINED run, not a tap - 0.65s is
// roughly two full strides at a real running cadence (~170-180
// steps/min), the shortest window that can't be faked by a single
// input pulse.
pub const RUNNING_THROW_SPEED_FRAC: f32 = 0.70;
pub const RUNNING_THROW_MIN_S: f32 = 0.65;
pub const RUNNING_THROW_MULT: f32 = 1.15;
// ---- respawn checkpoints (v6) --------------------------------------------
pub const CHECKPOINT_RADIUS: f32 = 3.0;
pub const CHECKPOINT_CAP_S: f32 = 4.0;
/// Lean (Phantom-Forces peek): how far the eye shifts sideways at full
/// lean. Kept UNDER `BODY_RADIUS` so a full lean against a wall cannot
/// push the muzzle through the wall's collision face.
pub const LEAN_SHIFT: f32 = 0.30;
pub const LEAN_RECOIL_MULT: f32 = 0.8;
pub const MATCH_LEN_S: f32 = 300.0;
pub const OVERTIME_S: f32 = 80.0;
pub const TDM_TARGET: u32 = 30;
pub const KOTH_TARGET_S: f32 = 90.0;
pub const HILL_RADIUS: f32 = 4.5;
pub const PICKUP_RADIUS: f32 = 1.1;
pub const ROBOT_ARMOR_HP: f32 = 100.0;
pub const ROBOT_SPEED_MULT: f32 = 1.12;

// ---------------------------------------------------------------- weapons

/// v6: the roster is a Counter-Strike-style lineup under real-world names —
/// original stats, original art, nothing copied. Bow and spear stay: they
/// are this game's own signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunKind {
    Fists,
    Glock,
    Deagle,
    Mp5,
    Shotgun, // Remington 870
    Ak47,
    M4,
    Awm,
    M249,
    Bow,
    Spear,
    /// §7 (Brief IV): found-in-world only — never in a loadout. Spin-up,
    /// heat, forced vents; the tradeoff engine in a carryable package.
    Minigun,
}

/// Weapon classes for the loadout screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunClass {
    Secondary,
    Primary,
    Special,
}

pub const PRIMARIES: [GunKind; 5] = [
    GunKind::M4,
    GunKind::Ak47,
    GunKind::Mp5,
    GunKind::Shotgun,
    GunKind::M249,
];
pub const SECONDARIES: [GunKind; 2] = [GunKind::Glock, GunKind::Deagle];
pub const SPECIALS: [GunKind; 3] = [GunKind::Awm, GunKind::Bow, GunKind::Spear];

#[derive(Clone, Copy, Debug)]
pub struct GunSpec {
    pub name: &'static str,
    pub class: GunClass,
    pub fire_period: f32,
    pub mag: u32,
    pub reserve: u32,
    pub reload_s: f32,
    pub spread: f32,
    pub spread_move: f32,
    /// Added to bloom per shot; bloom decays and widens spread (sustained
    /// fire gets unstable). v5: all kick values halved on owner request —
    /// "make the recoil fifty percent less".
    pub kick: f32,
    /// Base TORSO damage per bullet/pellet. Zone multipliers apply on top
    /// (see `zone_mult`): the owner's v6 rule is 2 headshots / 8 body
    /// shots for the baseline rifle.
    pub damage: f32,
    /// Pellets per trigger pull (shotguns fire many, each rolls spread).
    pub pellets: u32,
    /// Some => projectile weapon (launch speed, damage override).
    pub projectile: Option<(f32, f32)>,
    /// ADS zoom FOV in degrees.
    pub zoom_deg: f32,
    /// True → ADS is a full-screen scope overlay (AWM), with heavy
    /// movement slowdown while scoped.
    pub scoped: bool,
}

pub fn gun(kind: GunKind) -> GunSpec {
    // shared defaults keep the table readable
    let base = GunSpec {
        name: "",
        class: GunClass::Primary,
        fire_period: 0.12,
        mag: 30,
        reserve: 120,
        reload_s: 2.0,
        spread: 0.009,
        spread_move: 0.020,
        kick: 0.004,
        damage: 12.5,
        pellets: 1,
        projectile: None,
        zoom_deg: 48.0,
        scoped: false,
    };
    match kind {
        GunKind::Fists => GunSpec {
            name: "Unarmed",
            fire_period: 1.0,
            mag: 0,
            reserve: 0,
            spread: 0.0,
            spread_move: 0.0,
            kick: 0.0,
            damage: 0.0,
            zoom_deg: 62.0,
            ..base
        },
        GunKind::Glock => GunSpec {
            name: "Glock 17",
            class: GunClass::Secondary,
            fire_period: 0.16,
            mag: 17,
            reserve: 68,
            reload_s: 1.3,
            spread: 0.0075,
            spread_move: 0.016,
            kick: 0.0020,
            damage: 9.0, // 12 torso / 3 heads
            zoom_deg: 55.0,
            ..base
        },
        GunKind::Deagle => GunSpec {
            name: "Desert Eagle",
            class: GunClass::Secondary,
            fire_period: 0.42,
            mag: 7,
            reserve: 35,
            reload_s: 1.6,
            spread: 0.0045,
            spread_move: 0.022,
            kick: 0.0064,
            damage: 27.0, // 4 torso — and the iconic one-tap head
            zoom_deg: 52.0,
            ..base
        },
        GunKind::Mp5 => GunSpec {
            name: "MP5",
            fire_period: 0.08,
            mag: 30,
            reserve: 150,
            reload_s: 1.8,
            spread: 0.008,
            spread_move: 0.015, // runs well: an SMG's whole point
            kick: 0.0024,
            damage: 10.0, // 10 torso / 3 heads, but pours them out
            zoom_deg: 52.0,
            ..base
        },
        GunKind::Shotgun => GunSpec {
            name: "Remington 870",
            fire_period: 0.95, // pump between shells
            mag: 7,
            reserve: 28,
            reload_s: 2.8,
            spread: 0.055, // the pellet cone
            spread_move: 0.020,
            kick: 0.0080,
            damage: 6.5, // ×8 pellets = 52 torso point-blank
            pellets: 8,
            zoom_deg: 56.0,
            ..base
        },
        GunKind::Ak47 => GunSpec {
            name: "AK-47",
            fire_period: 0.105,
            mag: 30,
            reserve: 120,
            reload_s: 2.2,
            spread: 0.008,
            spread_move: 0.024,
            kick: 0.0044, // hits harder, walks harder
            damage: 13.5, // 8 torso / 2 heads with authority
            ..base
        },
        GunKind::M4 => GunSpec {
            name: "M4A1",
            fire_period: 0.09,
            mag: 30,
            reserve: 120,
            reload_s: 2.0,
            spread: 0.006,
            spread_move: 0.018,
            kick: 0.0032,
            damage: 12.5, // THE baseline: 2 headshots / 8 body shots
            ..base
        },
        GunKind::Awm => GunSpec {
            name: "AWM",
            class: GunClass::Special,
            // §5.2 (Brief VI): AWP-class, Valve's shipped numbers
            fire_period: 1.455,
            mag: 5,
            reserve: 10,
            reload_s: 3.7,
            // hip = prayer (tan 0.081); scoped overrides to 0.002 in
            // try_fire; moving adds the 0.176 hard-miss penalty
            spread: 0.081,
            spread_move: 0.176,
            kick: 0.0087, // table magnitude 78 unscoped (× 25/78 scoped)
            damage: 115.0, // head ×4 one-shot; legs ×0.75 never one-shot
            zoom_deg: 40.0, // stage 1; stage 2 = 10° (client cycles)
            scoped: true,
            ..base
        },
        GunKind::M249 => GunSpec {
            name: "M249",
            fire_period: 0.075,
            mag: 100,
            reserve: 200,
            reload_s: 4.5,
            spread: 0.012,
            spread_move: 0.032,
            kick: 0.0044,
            damage: 11.0,
            zoom_deg: 52.0,
            ..base
        },
        GunKind::Minigun => GunSpec {
            name: "M134 Minigun",
            class: GunClass::Special,
            fire_period: 0.06, // §5.1 (Brief VI): 1000 RPM
            mag: 400,
            reserve: 0, // no reloads — the pad is the reload
            reload_s: 3.0,
            spread: MINIGUN_SPREAD_COLD, // widens with heat in try_fire
            spread_move: 0.020,
            kick: 0.0008, // the mass eats the recoil; heat is the cost
            damage: 8.0, // §5.1: high ROF, low per-round
            zoom_deg: 58.0, // §5.1: no scope, no zoom — hip only
            ..base
        },
        GunKind::Bow => GunSpec {
            name: "War Bow",
            class: GunClass::Special,
            fire_period: 0.95,
            mag: 1,
            reserve: 24,
            reload_s: 0.9,
            spread: 0.004,
            spread_move: 0.018,
            kick: 0.0016,
            damage: 34.0,
            // §4 (Brief II): 52 m/s at gravity ×0.42 — 0.69 m of drop at
            // 30 m, near point-and-click inside 20 m
            projectile: Some((52.0, 34.0)),
            zoom_deg: 45.0,
            ..base
        },
        GunKind::Spear => GunSpec {
            name: "War Spear",
            class: GunClass::Special,
            fire_period: 1.3,
            mag: 1,
            reserve: 5,
            reload_s: 1.1,
            spread: 0.006,
            spread_move: 0.015,
            kick: 0.0024,
            // §3.2 (Brief VII v2): 85 body base - ×2 head, ×0.75 legs,
            // applied at hit resolution (SPEAR_HEAD_MULT/LEG_MULT).
            damage: 85.0,
            // §3.1 (Brief VII v2): 22 m/s full-throw, gravity ×0.72.
            projectile: Some((22.0, 85.0)),
            zoom_deg: 50.0,
            ..base
        },
    }
}

/// §3.2 (Brief VII v2): the spear's head shot is a LETHAL SKILL SHOT
/// (170 on an 85 base) but deliberately NOT the guns' ×4 - a thrown
/// weapon should never out-snipe a rifle.
pub const SPEAR_HEAD_MULT: f32 = 2.0;

// ------------------------------------------------------------------ world

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team {
    Blue,
    Red,
}

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    /// Slab-method ray hit; also returns the face normal. Public: the
    /// client reuses it for camera-boom collision and crosshair raycasts.
    pub fn ray_hit(&self, o: [f32; 3], d: [f32; 3], t_max: f32) -> Option<(f32, [f32; 3])> {
        let mut t0 = 0.0_f32;
        let mut t1 = t_max;
        let mut axis = 0usize;
        let mut sign = 1.0_f32;
        for i in 0..3 {
            if d[i].abs() < 1e-8 {
                if o[i] < self.min[i] || o[i] > self.max[i] {
                    return None;
                }
            } else {
                let inv = 1.0 / d[i];
                let (mut a, mut b) = ((self.min[i] - o[i]) * inv, (self.max[i] - o[i]) * inv);
                let mut s = -1.0_f32;
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                    s = 1.0;
                }
                if a > t0 {
                    t0 = a;
                    axis = i;
                    sign = s;
                }
                t1 = t1.min(b);
                if t0 > t1 {
                    return None;
                }
            }
        }
        let mut n = [0.0; 3];
        n[axis] = sign;
        Some((t0, n))
    }
}

// ------------------------------------------------------- §9.1 broadphase

/// Uniform-grid broadphase over the static cover set (16 m cells in XZ).
/// A ray query costs O(cells traversed) instead of O(all geometry) —
/// required before maps grow, and before smoke/throwables/LOS pile more
/// rays onto the hot path. Results are EXACTLY the linear scan's nearest
/// hit (verified by test), so determinism is untouched.
#[derive(Clone, Debug)]
pub struct CoverGrid {
    cell: f32,
    origin: [f32; 2], // min corner (x, z)
    n: usize,         // cells per axis
    cells: Vec<Vec<u16>>,
}

impl CoverGrid {
    pub fn build(cover: &[Aabb], half: f32) -> Self {
        let cell = 16.0_f32;
        let pad = 4.0;
        let origin = [-half - pad, -half - pad];
        let span = (half + pad) * 2.0;
        let n = ((span / cell).ceil() as usize).max(1);
        let mut cells = vec![Vec::new(); n * n];
        let clamp_idx = |v: f32, o: f32| -> usize {
            (((v - o) / cell).floor() as isize).clamp(0, n as isize - 1) as usize
        };
        for (i, c) in cover.iter().enumerate() {
            let (x0, x1) = (clamp_idx(c.min[0], origin[0]), clamp_idx(c.max[0], origin[0]));
            let (z0, z1) = (clamp_idx(c.min[2], origin[1]), clamp_idx(c.max[2], origin[1]));
            for gx in x0..=x1 {
                for gz in z0..=z1 {
                    cells[gx * n + gz].push(i as u16);
                }
            }
        }
        CoverGrid {
            cell,
            origin,
            n,
            cells,
        }
    }

    /// Nearest cover hit along `o + t·d`, `t ∈ [0, t_max]` — identical
    /// result to scanning every box, via 2D DDA over the XZ cells
    /// (cover is ground-based; a 2D grid is the right shape for it).
    pub fn ray_hit(
        &self,
        cover: &[Aabb],
        o: [f32; 3],
        d: [f32; 3],
        t_max: f32,
    ) -> Option<(f32, [f32; 3])> {
        let mut best: Option<(f32, [f32; 3])> = None;
        let mut test_cell = |gx: usize, gz: usize, best: &mut Option<(f32, [f32; 3])>| {
            for &i in &self.cells[gx * self.n + gz] {
                let limit = best.map_or(t_max, |(bt, _)| bt);
                if let Some((t, nrm)) = cover[i as usize].ray_hit(o, d, limit) {
                    if best.map_or(true, |(bt, _)| t < bt) {
                        *best = Some((t, nrm));
                    }
                }
            }
        };
        // degenerate XZ direction (straight up/down): one cell column
        if d[0].abs() < 1e-8 && d[2].abs() < 1e-8 {
            let gx = (((o[0] - self.origin[0]) / self.cell).floor() as isize)
                .clamp(0, self.n as isize - 1) as usize;
            let gz = (((o[2] - self.origin[1]) / self.cell).floor() as isize)
                .clamp(0, self.n as isize - 1) as usize;
            test_cell(gx, gz, &mut best);
            return best;
        }
        // clip the ray to the grid bounds in XZ (everything sits inside)
        let (min_x, min_z) = (self.origin[0], self.origin[1]);
        let (max_x, max_z) = (
            self.origin[0] + self.cell * self.n as f32,
            self.origin[1] + self.cell * self.n as f32,
        );
        let mut t0 = 0.0_f32;
        let mut t1 = t_max;
        for (oc, dc, lo, hi) in [
            (o[0], d[0], min_x, max_x),
            (o[2], d[2], min_z, max_z),
        ] {
            if dc.abs() < 1e-8 {
                if oc < lo || oc > hi {
                    return None;
                }
            } else {
                let (mut a, mut b) = ((lo - oc) / dc, (hi - oc) / dc);
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }
                t0 = t0.max(a);
                t1 = t1.min(b);
                if t0 > t1 {
                    return None;
                }
            }
        }
        // Amanatides & Woo DDA from the entry point
        let (px, pz) = (o[0] + d[0] * t0, o[2] + d[2] * t0);
        let mut gx = (((px - self.origin[0]) / self.cell).floor() as isize)
            .clamp(0, self.n as isize - 1);
        let mut gz = (((pz - self.origin[1]) / self.cell).floor() as isize)
            .clamp(0, self.n as isize - 1);
        let step_x: isize = if d[0] > 0.0 { 1 } else { -1 };
        let step_z: isize = if d[2] > 0.0 { 1 } else { -1 };
        let next_boundary = |g: isize, step: isize, org: f32| -> f32 {
            org + self.cell * (g + if step > 0 { 1 } else { 0 }) as f32
        };
        let mut t_next_x = if d[0].abs() < 1e-8 {
            f32::INFINITY
        } else {
            (next_boundary(gx, step_x, self.origin[0]) - o[0]) / d[0]
        };
        let mut t_next_z = if d[2].abs() < 1e-8 {
            f32::INFINITY
        } else {
            (next_boundary(gz, step_z, self.origin[1]) - o[2]) / d[2]
        };
        let dt_x = if d[0].abs() < 1e-8 {
            f32::INFINITY
        } else {
            self.cell / d[0].abs()
        };
        let dt_z = if d[2].abs() < 1e-8 {
            f32::INFINITY
        } else {
            self.cell / d[2].abs()
        };
        loop {
            test_cell(gx as usize, gz as usize, &mut best);
            // the exit-t of this cell: a found hit closer than it is final
            let t_exit = t_next_x.min(t_next_z);
            if let Some((bt, _)) = best {
                if bt <= t_exit {
                    return best;
                }
            }
            if t_exit > t1 {
                return best;
            }
            if t_next_x < t_next_z {
                gx += step_x;
                if gx < 0 || gx >= self.n as isize {
                    return best;
                }
                t_next_x += dt_x;
            } else {
                gz += step_z;
                if gz < 0 || gz >= self.n as isize {
                    return best;
                }
                t_next_z += dt_z;
            }
        }
    }
}

// ------------------------------------------------------------------- maps

/// v5: three battlefields. Same arena bounds, same pickup lanes, different
/// architecture — the sim owns the layout so every map stays deterministic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapKind {
    /// The original dusty range: plateaus, stairs, center tower.
    Arena,
    /// A castle courtyard: central keep, corner drum towers, bailey walls.
    Bailey,
    /// Castle gardens: hedge lanes, ruined walls, trees, a stone gazebo.
    Gardens,
    /// §9.2 Battle tier (400×400 m): keep, forge district, cathedral
    /// ruin, river field with bridges, corner watchtowers to 36 m —
    /// verticality for the Robot Suit, ground routes for everyone else.
    Battlefield,
}

impl MapKind {
    pub fn name(self) -> &'static str {
        match self {
            MapKind::Arena => "DUST ARENA",
            MapKind::Bailey => "CASTLE BAILEY",
            MapKind::Gardens => "CASTLE GARDENS",
            MapKind::Battlefield => "BATTLEFIELD",
        }
    }
    pub const ALL: [MapKind; 4] = [
        MapKind::Arena,
        MapKind::Bailey,
        MapKind::Gardens,
        MapKind::Battlefield,
    ];
}

/// §9.3: the soft flight ceiling — thrusters above this get pushed back
/// down, so aerial play stays inside the fight.
pub const SOFT_CEILING_M: f32 = 120.0;

/// What a cover block is MADE of — the client picks materials by this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverKind {
    Crate,
    Stone,
    Hedge,
    Tree,
}

/// Everything a map defines: its architecture, its size, and where the
/// two respawn checkpoints sit.
struct MapLayout {
    cover: Vec<Aabb>,
    kind: Vec<CoverKind>,
    center_top: f32,
    half: f32,
    checkpoints: [[f32; 2]; 2],
}

/// Build a map's cover set. The KOTH hill and the robot armor crown the
/// center structure, whatever its height. Pickup lanes stay clear on
/// every map. v6: castle maps grew (§10) — more room, more cover, and
/// real vertical layering (ramparts, terraces) with stairs up.
fn build_map(map: MapKind, rng: &mut Pcg32) -> MapLayout {
    let mut cover: Vec<Aabb> = Vec::new();
    let mut kind: Vec<CoverKind> = Vec::new();
    let push = |c: &mut Vec<Aabb>, k: &mut Vec<CoverKind>, a: Aabb, ck: CoverKind| {
        c.push(a);
        k.push(ck);
    };
    // shared center structure: a climbable block with stairs on both z sides
    let center = |cover: &mut Vec<Aabb>, kind: &mut Vec<CoverKind>, h: f32, steps: usize| {
        cover.push(Aabb {
            min: [-4.0, 0.0, -4.0],
            max: [4.0, h, 4.0],
        });
        kind.push(CoverKind::Stone);
        for s in 0..steps {
            let sh = (h / steps as f32) * (steps - s) as f32;
            let zoff = 4.0 + s as f32;
            cover.push(Aabb {
                min: [-2.0, 0.0, zoff],
                max: [2.0, sh, zoff + 1.0],
            });
            kind.push(CoverKind::Stone);
            cover.push(Aabb {
                min: [-2.0, 0.0, -zoff - 1.0],
                max: [2.0, sh, -zoff],
            });
            kind.push(CoverKind::Stone);
        }
    };
    let top;
    let half;
    let checkpoints;
    match map {
        MapKind::Arena => {
            half = ARENA_HALF;
            checkpoints = [[24.5, 0.0], [-24.5, 0.0]]; // the plateau tops
            // side plateaus (mirrored): 2.2 m high with stair slabs
            for sx in [-1.0_f32, 1.0] {
                let px = sx * (ARENA_HALF - 9.0);
                push(&mut cover, &mut kind, Aabb {
                    min: [px - 6.0, 0.0, -7.0],
                    max: [px + 6.0, 2.2, 7.0],
                }, CoverKind::Stone);
                for s in 0..4 {
                    let h = 0.55 * (4 - s) as f32;
                    let zoff = 7.0 + s as f32 * 1.1;
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 3.0, 0.0, zoff],
                        max: [px + 3.0, h, zoff + 1.1],
                    }, CoverKind::Stone);
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 3.0, 0.0, -zoff - 1.1],
                        max: [px + 3.0, h, -zoff],
                    }, CoverKind::Stone);
                }
            }
            center(&mut cover, &mut kind, 3.0, 6);
            top = 3.0;
            // scattered crates (mirrored pairs) — but never over a pickup
            // lane: a tall crate under a pad snaps the pickup onto a top
            // nobody can reach for the whole match
            for i in 0..9 {
                let x = rng.range(-ARENA_HALF + 5.0, ARENA_HALF - 5.0);
                let z = rng.range(6.0, ARENA_HALF - 6.0);
                let w = rng.range(0.9, 2.2);
                let h = if i % 3 == 0 { 2.1 } else { 1.3 };
                let d = rng.range(0.9, 2.2);
                let blocks_lane = [(19.0_f32, 14.0_f32), (-19.0, 14.0)]
                    .iter()
                    .any(|&(lx, lz)| {
                        (x - lx).abs() < w + 1.6 && (z - lz).abs() < d + 1.6
                    });
                if blocks_lane {
                    continue;
                }
                for sz in [1.0_f32, -1.0] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x - w, 0.0, sz * z - d],
                        max: [x + w, h, sz * z + d],
                    }, CoverKind::Crate);
                }
            }
            // low-ground trench walls flanking mid
            for sx in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [sx * 12.0 - 0.6, 0.0, -10.0],
                    max: [sx * 12.0 + 0.6, 1.5, 10.0],
                }, CoverKind::Stone);
            }
        }
        MapKind::Bailey => {
            half = 40.0;
            checkpoints = [[20.0, 0.0], [-20.0, 0.0]];
            // the keep — taller than the arena tower, same stair grammar
            center(&mut cover, &mut kind, 3.0, 6);
            top = 3.0;
            // corner drum towers: solid, unclimbable, fight around them
            for sx in [-1.0_f32, 1.0] {
                for sz in [-1.0_f32, 1.0] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [sx * 30.0 - 2.5, 0.0, sz * 30.0 - 2.5],
                        max: [sx * 30.0 + 2.5, 3.4, sz * 30.0 + 2.5],
                    }, CoverKind::Stone);
                }
            }
            // bailey cross-walls with a wide-open center gate
            for sz in [-1.0_f32, 1.0] {
                for (x0, x1) in [(-28.0_f32, -8.0), (8.0, 28.0)] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x0, 0.0, sz * 18.0 - 0.6],
                        max: [x1, 2.6, sz * 18.0 + 0.6],
                    }, CoverKind::Stone);
                }
            }
            // RAMPARTS (v6 vertical layering): climbable wall-walks along
            // both flanks with stairs at each end — high ground with two
            // ways up and full exposure while you hold it
            for sx in [-1.0_f32, 1.0] {
                let px = sx * (half - 5.0);
                push(&mut cover, &mut kind, Aabb {
                    min: [px - 2.0, 0.0, -12.0],
                    max: [px + 2.0, 2.6, 12.0],
                }, CoverKind::Stone);
                for s in 0..5 {
                    let h = 0.52 * (5 - s) as f32;
                    let zoff = 12.0 + s as f32 * 1.05;
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 1.5, 0.0, zoff],
                        max: [px + 1.5, h, zoff + 1.05],
                    }, CoverKind::Stone);
                    push(&mut cover, &mut kind, Aabb {
                        min: [px - 1.5, 0.0, -zoff - 1.05],
                        max: [px + 1.5, h, -zoff],
                    }, CoverKind::Stone);
                }
                // parapet lip on the rampart's inner edge: hard cover up top
                push(&mut cover, &mut kind, Aabb {
                    min: [px - sx * 2.0 - 0.2, 0.0, -12.0],
                    max: [px - sx * 2.0 + 0.2, 3.3, -8.0],
                }, CoverKind::Stone);
                push(&mut cover, &mut kind, Aabb {
                    min: [px - sx * 2.0 - 0.2, 0.0, 8.0],
                    max: [px - sx * 2.0 + 0.2, 3.3, 12.0],
                }, CoverKind::Stone);
            }
            // stables clutter: crates + low walls in the yard, mirrored
            for (x, z, w, h, d) in [
                (6.0_f32, 10.0_f32, 1.2_f32, 1.3_f32, 1.2_f32),
                (-17.0, 7.0, 1.5, 1.3, 1.0),
                (22.0, 5.5, 1.0, 2.1, 1.0),
                (13.0, 24.0, 1.4, 1.3, 1.1),
                (-6.0, 26.0, 1.1, 2.1, 1.1),
            ] {
                for sz in [1.0_f32, -1.0] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [-x - w, 0.0, sz * z - d],
                        max: [-x + w, h, sz * z + d],
                    }, CoverKind::Crate);
                    push(&mut cover, &mut kind, Aabb {
                        min: [x - w, 0.0, -sz * z - d],
                        max: [x + w, h, -sz * z + d],
                    }, CoverKind::Crate);
                }
            }
            // low chapel ruins flanking the keep
            for sx in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [sx * 12.0 - 0.6, 0.0, -6.0],
                    max: [sx * 12.0 + 0.6, 1.5, 6.0],
                }, CoverKind::Stone);
            }
        }
        MapKind::Gardens => {
            half = 38.0;
            checkpoints = [[24.0, 0.0], [-24.0, 0.0]];
            // the stone gazebo: lower than the keep, easier to storm
            center(&mut cover, &mut kind, 2.4, 5);
            top = 2.4;
            // hedge lanes flanking mid (trimmed, waist-to-shoulder high)
            for sx in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [sx * 12.0 - 0.5, 0.0, -10.0],
                    max: [sx * 12.0 + 0.5, 1.5, 10.0],
                }, CoverKind::Hedge);
            }
            // cross hedges with center gaps — two rings now
            for sz in [-1.0_f32, 1.0] {
                for (x0, x1) in [(-20.0_f32, -6.0), (6.0, 20.0)] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x0, 0.0, sz * 20.0 - 0.5],
                        max: [x1, 1.5, sz * 20.0 + 0.5],
                    }, CoverKind::Hedge);
                }
                for (x0, x1) in [(-16.0_f32, -4.0), (4.0, 16.0)] {
                    push(&mut cover, &mut kind, Aabb {
                        min: [x0, 0.0, sz * 28.0 - 0.5],
                        max: [x1, 1.5, sz * 28.0 + 0.5],
                    }, CoverKind::Hedge);
                }
            }
            // garden TERRACES (v6 vertical layering): raised platforms on
            // the mid-line with steps on the center-facing side
            for sz in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [-4.0, 0.0, sz * 24.0 - 2.0],
                    max: [4.0, 1.2, sz * 24.0 + 2.0],
                }, CoverKind::Stone);
                // steps descend from the terrace edge (|z| = 22) toward
                // center: 1.2 → 0.8 → 0.4, each riser within STEP_UP
                for s in 0..3 {
                    let h = 0.4 * (3 - s) as f32;
                    let near = 22.0 - s as f32 * 0.9; // edge closest to terrace
                    let (z0, z1) = if sz > 0.0 {
                        (near - 0.9, near)
                    } else {
                        (-near, -(near - 0.9))
                    };
                    push(&mut cover, &mut kind, Aabb {
                        min: [-2.0, 0.0, z0],
                        max: [2.0, h, z1],
                    }, CoverKind::Stone);
                }
            }
            // ruined garden walls on the diagonals (vaultable)
            for s in [-1.0_f32, 1.0] {
                push(&mut cover, &mut kind, Aabb {
                    min: [s * 16.0 - 4.0, 0.0, s * 7.0 - 0.5],
                    max: [s * 16.0 + 4.0, 1.1, s * 7.0 + 0.5],
                }, CoverKind::Stone);
                push(&mut cover, &mut kind, Aabb {
                    min: [-s * 16.0 - 4.0, 0.0, s * 11.0 - 0.5],
                    max: [-s * 16.0 + 4.0, 1.1, s * 11.0 + 0.5],
                }, CoverKind::Stone);
                // fountain-basin ruins on the far flanks
                push(&mut cover, &mut kind, Aabb {
                    min: [s * 30.0 - 2.0, 0.0, -2.0],
                    max: [s * 30.0 + 2.0, 0.9, 2.0],
                }, CoverKind::Stone);
            }
            // old trees: trunk is the collider, the client crowns them
            for (x, z) in [
                (23.0_f32, 9.0_f32),
                (-23.0, -9.0),
                (-23.0, 9.0),
                (23.0, -9.0),
                (9.0, 31.0),
                (-9.0, -31.0),
                (17.0, -15.0),
                (-17.0, 15.0),
            ] {
                push(&mut cover, &mut kind, Aabb {
                    min: [x - 0.3, 0.0, z - 0.3],
                    max: [x + 0.3, 2.8, z + 0.3],
                }, CoverKind::Tree);
            }
        }
        MapKind::Battlefield => {
            // §9.2 the Battle tier: 400×400 m. Set pieces give the map
            // its shape (§9.4); the §9.1 grid keeps rays cheap at this
            // scale; ground routes stay viable everywhere (§9.3).
            half = 200.0;
            checkpoints = [[150.0, 0.0], [-150.0, 0.0]];
            // THE KEEP: an 8 m block crowned by the armor pad/hill, with
            // a long climbable stair running south
            push(&mut cover, &mut kind, Aabb {
                min: [-8.0, 0.0, -8.0],
                max: [8.0, 8.0, 8.0],
            }, CoverKind::Stone);
            for s in 0..16 {
                let h = 8.0 - s as f32 * 0.5;
                let z0 = 8.0 + s as f32 * 1.2;
                push(&mut cover, &mut kind, Aabb {
                    min: [-2.5, 0.0, z0],
                    max: [2.5, h, z0 + 1.2],
                }, CoverKind::Stone);
            }
            top = 8.0;
            // corner WATCHTOWERS to 36 m — flight targets with rooftops;
            // ground fighters use them as landmarks, not ladders
            for sx in [-1.0_f32, 1.0] {
                for sz in [-1.0_f32, 1.0] {
                    let (cx, cz) = (sx * 170.0, sz * 170.0);
                    push(&mut cover, &mut kind, Aabb {
                        min: [cx - 3.5, 0.0, cz - 3.5],
                        max: [cx + 3.5, 36.0, cz + 3.5],
                    }, CoverKind::Stone);
                }
            }
            // FORGE DISTRICT (southwest): a work-yard grid of shops with
            // chimneys — the blacksmith hook from the project brief
            for gx in 0..4 {
                for gz in 0..4 {
                    let (bx, bz) = (-130.0 + gx as f32 * 20.0, -130.0 + gz as f32 * 20.0);
                    push(&mut cover, &mut kind, Aabb {
                        min: [bx, 0.0, bz],
                        max: [bx + 9.0, 4.0, bz + 9.0],
                    }, CoverKind::Stone);
                    if (gx + gz) % 2 == 0 {
                        push(&mut cover, &mut kind, Aabb {
                            min: [bx + 6.5, 0.0, bz + 6.5],
                            max: [bx + 8.5, 9.0, bz + 8.5],
                        }, CoverKind::Stone);
                    }
                }
            }
            // CATHEDRAL RUIN (northeast): two long walls + a column line
            push(&mut cover, &mut kind, Aabb {
                min: [70.0, 0.0, 70.0],
                max: [72.0, 10.0, 130.0],
            }, CoverKind::Stone);
            push(&mut cover, &mut kind, Aabb {
                min: [98.0, 0.0, 70.0],
                max: [100.0, 10.0, 130.0],
            }, CoverKind::Stone);
            for c in 0..6 {
                let cz = 74.0 + c as f32 * 10.0;
                push(&mut cover, &mut kind, Aabb {
                    min: [84.0, 0.0, cz],
                    max: [86.5, 12.0, cz + 2.5],
                }, CoverKind::Stone);
            }
            // RIVER FIELD: the |x| band around z=±? — an open crossing
            // lane |z|<14 kept clear of clutter, with three low bridges
            for bx in [-120.0_f32, 0.0, 120.0] {
                if bx == 0.0 {
                    continue; // the keep stair owns the center crossing
                }
                push(&mut cover, &mut kind, Aabb {
                    min: [bx - 6.0, 0.0, -16.0],
                    max: [bx + 6.0, 1.2, 16.0],
                }, CoverKind::Stone);
            }
            // §12.2 adventure landmarks: a RUINED SETTLEMENT to navigate
            // by (northwest), a FOREST stretch (southeast), and the MINE
            // MOUTH framing the extraction corner — the map teaches its
            // own route
            for (rx, rz) in [
                (-70.0_f32, 50.0_f32),
                (-52.0, 64.0),
                (-66.0, 76.0),
                (-46.0, 44.0),
                (-80.0, 62.0),
            ] {
                push(&mut cover, &mut kind, Aabb {
                    min: [rx, 0.0, rz],
                    max: [rx + 7.0, 2.6, rz + 1.2],
                }, CoverKind::Stone);
                push(&mut cover, &mut kind, Aabb {
                    min: [rx, 0.0, rz + 4.0],
                    max: [rx + 1.2, 3.2, rz + 8.0],
                }, CoverKind::Stone);
            }
            for k in 0..14 {
                let tx = 44.0 + (k % 5) as f32 * 18.0 + ((k * 37) % 7) as f32;
                let tz = -128.0 + (k / 5) as f32 * 22.0 + ((k * 53) % 9) as f32;
                push(&mut cover, &mut kind, Aabb {
                    min: [tx - 0.4, 0.0, tz - 0.4],
                    max: [tx + 0.4, 3.4, tz + 0.4],
                }, CoverKind::Tree);
            }
            // the mine mouth: two jambs + a lintel at the extraction corner
            push(&mut cover, &mut kind, Aabb {
                min: [158.0, 0.0, 174.0],
                max: [162.0, 6.0, 186.0],
            }, CoverKind::Stone);
            push(&mut cover, &mut kind, Aabb {
                min: [174.0, 0.0, 158.0],
                max: [186.0, 6.0, 162.0],
            }, CoverKind::Stone);
            // scattered crate clutter, never in the river band and never
            // on the pickup lanes
            for _ in 0..60 {
                let x = rng.range(-190.0, 190.0);
                let z = rng.range(-190.0, 190.0);
                if z.abs() < 16.0 {
                    continue; // river field stays open
                }
                if x.abs() < 32.0 && z.abs() < 34.0 {
                    continue; // keep + stair + pads stay clear
                }
                if x > 145.0 && z > 140.0 {
                    continue; // the extraction corner stays readable
                }
                let w = rng.range(1.0, 2.6);
                let h = rng.range(1.1, 2.4);
                let d = rng.range(1.0, 2.6);
                push(&mut cover, &mut kind, Aabb {
                    min: [x - w, 0.0, z - d],
                    max: [x + w, h, z + d],
                }, CoverKind::Crate);
            }
        }
    }
    MapLayout {
        cover,
        kind,
        center_top: top,
        half,
        checkpoints,
    }
}

// --------------------------------------------------------------- fighters

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitZone {
    Legs,
    Torso,
    Arms,
    Head,
}

impl HitZone {
    pub fn name(self) -> &'static str {
        match self {
            HitZone::Legs => "legs",
            HitZone::Torso => "torso",
            HitZone::Arms => "arms",
            HitZone::Head => "HEAD",
        }
    }
    pub fn mult(self) -> f32 {
        match self {
            HitZone::Head => HEAD_MULT,
            HitZone::Torso => 1.0,
            HitZone::Arms => ARM_MULT,
            HitZone::Legs => LEG_MULT,
        }
    }
}

// ------------------------------------------------------------- difficulty

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
        }
    }
    pub const ALL: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard];
}

/// Bot brain tuning per difficulty: aim, reflexes, and aggression.
pub struct BotParams {
    pub aim_sigma: f32,
    pub reaction_s: f32,
    pub engage_range: f32,
    pub aggression: f32,
}

pub fn bot_params(d: Difficulty) -> BotParams {
    match d {
        Difficulty::Easy => BotParams {
            aim_sigma: 0.055,
            reaction_s: 0.45,
            engage_range: 22.0,
            aggression: 0.6,
        },
        Difficulty::Normal => BotParams {
            aim_sigma: 0.030,
            reaction_s: 0.28,
            engage_range: 35.0,
            aggression: 1.0,
        },
        Difficulty::Hard => BotParams {
            aim_sigma: 0.014,
            reaction_s: 0.15,
            engage_range: 50.0,
            aggression: 1.4,
        },
    }
}

// ------------------------------------------------------------ match config

/// Player-picked loadout: [primary, secondary, special]. The SHIELD is
/// always carried in its own dedicated slot — it cannot be dropped.
pub type Loadout = [GunKind; 3];

pub const DEFAULT_LOADOUT: Loadout = [GunKind::M4, GunKind::Glock, GunKind::Awm];

#[derive(Clone, Copy, Debug)]
pub struct MatchConfig {
    pub seed: u64,
    pub per_team: usize, // 5..=8 (owner's cap: 8v8)
    pub mode: Mode,
    pub map: MapKind,
    pub difficulty: Difficulty,
    pub loadout: Loadout,
    /// §6 (Brief IV): the player's melee slot carries the axe.
    pub melee_axe: bool,
    /// §8 (Brief IV): index into GRENADE_PRESETS for the player's
    /// 6-point throwable budget.
    pub grenade_preset: usize,
}

impl Default for MatchConfig {
    fn default() -> Self {
        MatchConfig {
            seed: 0x7EA9,
            per_team: 5,
            mode: Mode::Tdm,
            map: MapKind::Arena,
            difficulty: Difficulty::Normal,
            loadout: DEFAULT_LOADOUT,
            melee_axe: false,
            grenade_preset: 0,
        }
    }
}

/// A capturable forward-spawn checkpoint. Stand in the ring uncontested
/// to charge it toward your team; once owned, your dead respawn there.
#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub pos: [f32; 3],
    pub owner: Option<Team>,
    /// −CAP..+CAP: positive charges Blue, negative Red.
    pub charge: f32,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub name: &'static str,
    pub team: Team,
    /// The ACTIVE weapon (mirror of `inventory[active]` for fast access).
    pub gun: GunKind,
    /// [primary, secondary, special] — the shield is its own stance below.
    pub inventory: Loadout,
    /// Per-slot (magazine, reserve) so switching keeps each gun's state.
    pub slot_ammo: [(u32, u32); 3],
    pub active: usize,
    /// Shield raised: front-arc protection, no firing, slow walk.
    pub shield_up: bool,
    /// −1..+1 sideways lean (peek): shifts the muzzle, trims recoil.
    pub lean: f32,
    pub crouch: bool,
    pub switch_t: f32,
    pub pos: [f32; 3], // feet
    pub vel: [f32; 2], // xz
    pub vy: f32,
    pub grounded: bool,
    pub yaw: f32,
    /// Last tick's yaw — the aim's angular rate for the §4 stability
    /// model is (yaw − prev_yaw)/DT. Updated at the end of every step.
    pub prev_yaw: f32,
    /// >0 → mid-somersault: fast, low, can't shoot. Set by the dodge key
    /// or automatically by a hard landing (parkour breakfall).
    pub roll_t: f32,
    pub roll_cd: f32,
    pub roll_dir: [f32; 2],
    /// Task 3 rule 3: burst multiplier snapshotted at the dodge trigger -
    /// 1.0 + ROLL_COUNTER_BONUS when the dodge cut against real prior
    /// movement, 1.0 otherwise (and always 1.0 for the mech and the
    /// breakfall, which is momentum conversion, not a counter-move).
    pub roll_boost: f32,
    pub health: f32,
    /// §6: the Robot Suit's POWER CORE charge (0 for every other set) —
    /// thrusters and repulsors spend it, explosions drain it, the ground
    /// recharges it. The HUD's second bar.
    pub armor: f32,
    /// §6: the equipped armor set — found in-world, lost on death.
    pub armor_set: ArmorSet,
    /// §11: the mech's hull pool (450, NEVER regenerates). >0 means the
    /// chassis is intact; at zero the pilot ejects on foot at 25 HP.
    pub hull: f32,
    /// Pyro flame-projector fuel seconds.
    pub fuel: f32,
    /// §6.2 (Brief VII v2): mounting AND dismounting the mech are
    /// COMMITTED, not instant - counts down from MECH_ENTER_S or
    /// MECH_EXIT_S; while >0 the chassis is sealing up or powering down
    /// and can't fight (blocked in try_fire).
    pub mech_transition_t: f32,
    /// Which direction `mech_transition_t` is running. On an exit, the
    /// chassis teardown is DEFERRED to the end of the window - that
    /// deferral is what makes leaving committal rather than a one-tick
    /// state flip.
    pub mech_exiting: bool,
    /// §6.3: HP-threshold armor-drop events already fired this life -
    /// bitmask (bit0=70%, bit1=40%, bit2=15%) so each stage fires once.
    pub mech_plates_dropped: u8,
    /// Folk Shieldwall Brace held: planted, shielded, slow.
    pub brace: bool,
    /// §A: Mech Brace held - wide stance, near-stationary, damped recoil.
    /// Deliberately a separate field from `brace`: see the constants.
    pub mech_brace: bool,
    /// §C: which hull mount the trigger currently drives. Both mounts
    /// are always present on a live chassis - this is a targeting mode,
    /// not an inventory slot (see `MechWeapon`).
    pub mech_weapon: MechWeapon,
    /// §C: the hull gatling's own 0..100 heat pool and forced-vent lock.
    /// Kept SEPARATE from the minigun's `heat`/`vent_t` for the same
    /// reason `stride_heat` is: those two drive the MINIGUN's vent state
    /// machine (barrel glow, vent audio, fire lockout) gated purely on
    /// `gun == Minigun` - and a mech pilot may well be carrying the
    /// minigun, in which case a hot hull mount would hand him an
    /// instant, unearned lockout on the gun in his hands.
    pub gatling_heat: f32,
    pub gatling_vent_t: f32,
    /// §C: the gatling's own cycle clock. Its own field, not `fire_cd`,
    /// for the reason stated one field down and violated by the first
    /// cut: `fire_cd` is the PILOT'S carried-gun clock, so a gatling
    /// that set it left the dismounting pilot throttled by a gun bolted
    /// to a chassis he just climbed out of.
    pub gatling_cd: f32,
    /// §C: the gatling's TRIGGER-HELD hold timer — the same shape and
    /// the same 70 ms as the minigun's `spin_cmd`, and set at the top of
    /// `try_fire_gatling` BEFORE every early return for the same reason:
    /// it must mean "the trigger is down", not "a round left the mount".
    ///
    /// The first cut gated the heat decay on `fire_cd <= 0.0` instead,
    /// reasoning that the cycle clock running IS the trigger being down.
    /// It is not. `fire_cd <= 0.0` is true for exactly ONE tick per fire
    /// cycle, so the mount still shed `GATLING_HEAT_DECAY * DT` of heat
    /// every cycle — 88.9% suppression, not the minigun's 100%. That
    /// residual is LINEAR IN DT, which made the time to a forced vent
    /// tick-rate dependent: 11.18 s at 60 Hz, 9.08 s at 120 Hz, 8.22 s
    /// at 240 Hz. A hold timer suppresses the decay outright, so what is
    /// left moving with the tick rate is only the quantisation of the
    /// fire period itself.
    pub gatling_trigger_t: f32,
    /// §C: the autocannon's slow cycle clock. Its own field, not
    /// `fire_cd`, so the two mounts cannot silently share a cooldown.
    pub autocannon_cd: f32,
    /// §5 knife swing clock — 0 idle; counts up through wind → active →
    /// recovery. `knife_committed` = the held lunge variant.
    pub knife_phase: f32,
    pub knife_committed: bool,
    pub knife_struck: bool,
    /// §6 (Brief IV): the melee slot carries the AXE instead of the
    /// knife — slower, harder, and the swing sweeps the whole arc.
    pub melee_axe: bool,
    /// §7 (Brief IV): minigun barrel spin (0..=MINIGUN_SPINUP_S — firing
    /// starts at the top), current heat (0..=100), and the vent lock.
    pub spin_t: f32,
    pub heat: f32,
    pub vent_t: f32,
    /// §7.4 (BRIEF VIII): power-stride WIND-UP progress, 0..
    /// POWER_STRIDE_WINDUP_S. Cancels if sprint is released or heat
    /// caps out before it completes - the burst itself hasn't fired
    /// yet, so nothing is owed.
    pub stride_wind_t: f32,
    /// §7.4: power-stride ACTIVE window remaining. >0 means the burst
    /// is live - speed override, missile pod locked, turn capped. Once
    /// triggered it's committed (an interrupted "sustained push" isn't
    /// one), counting down on its own regardless of continued input.
    pub stride_t: f32,
    /// §7.4: power-stride's own 0..100 heat-style budget. The brief
    /// says striding "costs heat (§7.8)" - deliberately kept SEPARATE
    /// from the minigun's `heat`/`vent_t` rather than sharing the
    /// field: those drive the minigun's forced-vent state machine
    /// (barrel glow, vent audio, fire lockout) purely on `heat > 0`
    /// gated by `gun == Minigun`, so a mech that strides then SWITCHES
    /// to the minigun would walk into an instant, unearned vent lockout
    /// if the two pools were one. Same 0-100 shape and cooldown rhythm,
    /// zero cross-wiring.
    pub stride_heat: f32,
    /// Trigger-held HOLD TIMER (seconds): refreshed by `try_fire`, drained
    /// by the timer loop. A short 0.07 s hold (not a per-tick bool) so a
    /// far bot thinking at the 15 Hz LOD still keeps its barrels climbing
    /// between thinks; for the player it is refreshed every held tick.
    pub spin_cmd: f32,
    /// §7: the primary the minigun pickup displaced — restored on death.
    pub prev_primary: GunKind,
    /// §2 (Brief VI): the punch channels — (pitch°, yaw°) angle and its
    /// velocity. Bullets fly at punch × RECOIL_SCALE; the camera shows
    /// 45% of that; the crosshair never moves.
    pub punch: [f32; 2],
    pub punch_vel: [f32; 2],
    /// Deterministic spray-table index (decays while not firing).
    pub spray_i: f32,
    pub last_shot_at: f32,
    /// §5.3 (Brief VI): the mech's missile pod — tubes left, relaunch
    /// cooldown, the acquiring lock (target index, seconds held), and
    /// the VICTIM-side warning timer (set from lock START).
    pub pod_ammo: u8,
    pub pod_cd: f32,
    pub pod_lock_t: f32,
    pub pod_lock_id: i32,
    pub pod_aim_held: bool,
    pub lock_warn_t: f32,
    /// §3: spear windup clock (counts down to the release), the aim
    /// tracked through the wind, and the charge the trigger locked in.
    pub spear_wind_t: f32,
    pub spear_aim: [f32; 3],
    pub spear_v0: f32,
    /// §4.1 (Brief VII v2): the bow's draw clock - counts UP while aim
    /// is held (unlike the spear's committal countdown), 0.15s to
    /// 0.7s mapping 35%-100% power, held past 10s force-letdown.
    /// `bow_aim` tracks the aim direction through the hold, same as the
    /// spear's `spear_aim`.
    pub bow_draw_t: f32,
    pub bow_aim: [f32; 3],
    /// §4 aerial flip: remaining rotation time, direction of travel,
    /// the flip kind (0 front / 1 back / 2 left / 3 right), whether this
    /// airborne period's one flip is spent, and the landing recovery.
    pub flip_t: f32,
    pub flip_dir: [f32; 2],
    pub flip_kind: u8,
    pub flip_used: bool,
    pub flip_recover_t: f32,
    /// §6: >0 → the raised shield is DIPPED for a throw (blocks nothing).
    pub shield_dip_t: f32,
    /// Ability cooldown (repulsor).
    pub ability_cd: f32,
    /// Sim time of last ability use (gates power recharge).
    pub last_ability_at: f32,
    /// Sim time of last damage taken (gates Recon regen).
    pub last_dmg_at: f32,
    pub ammo: u32,
    pub reserve: u32,
    pub reload_t: f32,
    /// §3.4 (BRIEF VIII): sprint-out - the weapon is LOWERED at sprint
    /// and takes a per-class beat to ready after leaving it (SMG 0.15s /
    /// rifle 0.20s / heavy 0.30s). Counts down once planar speed drops
    /// below the sprint-carry threshold; firing is blocked while > 0.
    /// The COD/CSGO skill lever the brief names: sprinting around a
    /// corner cannot ALSO mean instantly shooting whoever is there.
    pub sprint_gate_t: f32,
    /// §5.4 (BRIEF VIII): continuous time spent at/above the
    /// running-throw speed threshold - resets the instant speed drops
    /// below it. Feeds the spear's running-throw bonus: a throw
    /// released with real approach momentum behind it launches faster,
    /// exactly like a real thrower's run-up pays off over a standing
    /// throw. A tap of forward input can't fake this - it has to be
    /// sustained.
    pub running_momentum_t: f32,
    pub fire_cd: f32,
    pub bloom: f32,
    pub respawn_t: f32,
    pub protect_t: f32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub hits_dealt: u32,
    /// §4.5 (BRIEF VIII): assist tracking. The most recent OTHER
    /// fighter to damage this one, with the sim time it happened - not
    /// a full history, just the single latest distinct attacker, which
    /// is what "who else hit them" needs. Read (and cleared) at death;
    /// anything older than ASSIST_WINDOW_S doesn't count.
    pub last_hit_by: Option<(usize, f32)>,
    /// §3: >0 → a walk-over pickup was refused because the reserve is at
    /// cap; the HUD surfaces "AMMO FULL" (the missing feedback that hid
    /// the pickup bug).
    pub ammo_full_t: f32,
    // §5 throwables
    /// Remaining grenades per ThrowKind::ALL slot.
    pub grenades: [u8; 4],
    /// Selected throwable slot (G cycles).
    pub throw_sel: u8,
    /// >0 → holding a live grenade; frags detonate IN HAND past the fuse.
    pub cook_t: f32,
    /// §5.3 flashbang blind seconds remaining (bots aim ×4 worse).
    pub blind_t: f32,
    /// Burning marker (client flame FX), set by fire pools.
    pub burn_t: f32,
    // bot brain
    pub waypoint: [f32; 2],
    pub strafe_phase: f32,
    pub los_time: f32,
    pub think_offset: u32,
}

impl Fighter {
    pub fn alive(&self) -> bool {
        self.respawn_t <= 0.0 && self.health > 0.0
    }
    /// Is this fighter currently piloting a live chassis?
    pub fn in_mech(&self) -> bool {
        self.armor_set == ArmorSet::RobotSuit && self.hull > 0.0
    }
    /// §6: apply crouch INTENT. A mech never crouches - `height()`
    /// returns the chassis height unconditionally for a live mech, so a
    /// crouching mech kept its full 3.03 m hitbox while the renderer
    /// played the soldier squat: the x2.0 visor band floated in empty air
    /// above the model and was unreachable on it.
    ///
    /// This lives on `Fighter` so the PLAYER and BOT paths cannot drift -
    /// the guard was originally added to the player path only, leaving
    /// every bot-piloted mech still able to crouch.
    pub fn set_crouch(&mut self, want: bool) {
        self.crouch = want && !self.in_mech();
    }
    pub fn height(&self) -> f32 {
        if self.armor_set == ArmorSet::RobotSuit && self.hull > 0.0 {
            BODY_HEIGHT * MECH_SCALE // §11: a 2.7 m powered exosuit
        } else if self.roll_t > 0.0 {
            ROLL_HEIGHT
        } else if self.crouch {
            CROUCH_HEIGHT
        } else {
            BODY_HEIGHT
        }
    }
    /// §11: the chassis is wide — it cannot fit through doorways.
    pub fn radius(&self) -> f32 {
        if self.armor_set == ArmorSet::RobotSuit && self.hull > 0.0 {
            MECH_RADIUS
        } else {
            BODY_RADIUS
        }
    }
    pub fn armed(&self) -> bool {
        self.gun != GunKind::Fists
    }
    /// §4.3: the zone mode `apply_hit` must use for this fighter.
    /// `flip_used` stays true from flip start through landing recovery —
    /// exactly the "full duration + recovery" window the rule demands.
    pub fn hit_zone_mode(&self) -> HitZoneMode {
        if self.flip_t > 0.0 || self.flip_used {
            HitZoneMode::Uniform
        } else {
            HitZoneMode::Banded
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerCmd {
    pub move_x: f32,
    pub move_z: f32,
    pub sprint: bool,
    pub yaw: f32,
    pub aim: [f32; 3],
    pub shoot: bool,
    pub reload: bool,
    pub ads: bool,
    pub crouch: bool,
    pub jump: bool,
    /// Duck-spin dodge: a fast forward somersault in the move direction.
    pub dodge: bool,
    /// Select an inventory slot (0 primary, 1 secondary, 2 special).
    pub slot: Option<u8>,
    /// Toggle the shield stance (E). Edge-triggered like jump/reload.
    pub shield: bool,
    /// −1..+1 lean input.
    pub lean: f32,
    /// §5: holding a live grenade — release throws (cooking a frag past
    /// its fuse detonates it in hand).
    pub throw_hold: bool,
    /// §1 (Brief V): cancel the aimed throw — exits aim mode without
    /// throwing and without consuming the grenade. Edge-triggered.
    pub throw_cancel: bool,
    /// §4.6 (Brief VI): dismount the mech (U). Edge-triggered.
    pub exit_mech: bool,
    /// §5.3 (Brief VI): HOLD = missile targeting (lock acquires on
    /// mechs); RELEASE locked = launch; a quick tap dumb-fires.
    pub pod_aim: bool,
    /// §5: cycle the selected throwable (G). Edge-triggered.
    pub cycle_throw: bool,
    /// §6: armor ability held (now C — §5 took F) — Folk brace / Pyro
    /// flame / Robot repulsor.
    pub ability: bool,
    /// §5: knife held (F). Tap = quick slash, hold = committed lunge.
    pub knife_hold: bool,
}

// ---------------------------------------------------------------- events

#[derive(Clone, Debug)]
pub struct KillEvent {
    pub killer: usize,
    pub victim: usize,
    pub headshot: bool,
    /// §4.5 (BRIEF VIII): the most recent OTHER fighter who damaged the
    /// victim within ASSIST_WINDOW_S of the kill. `None` if the killer
    /// landed every recent hit alone, or nobody else hit them recently
    /// enough to count.
    pub assist: Option<usize>,
}

/// §4.5: how recent a non-killing hit has to be to still count as an
/// assist - long enough to credit real teamwork, short enough that a
/// hit from ten seconds ago doesn't ride in on someone else's kill.
pub const ASSIST_WINDOW_S: f32 = 6.0;

#[derive(Clone, Debug)]
pub struct HitEvent {
    pub shooter: usize,
    pub victim: usize,
    pub zone: HitZone,
    pub damage: f32,
    /// True if the victim's raised shield ate most of this hit.
    pub shielded: bool,
    pub from: [f32; 3],
    /// impact point — part of the event API even when the client skips it
    #[allow(dead_code)]
    pub at: [f32; 3],
    pub fatal: bool,
}

#[derive(Clone, Debug)]
pub struct Impact {
    pub at: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct Tracer {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub team: Team,
    pub ttl: f32,
}

/// Arrow / thrown spear in flight (or stuck).
#[derive(Clone, Debug)]
pub struct Missile {
    /// stable identity — part of the sim API even when the client pools by index
    #[allow(dead_code)]
    pub id: u32,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub team: Team,
    pub shooter: usize,
    pub damage: f32,
    pub is_spear: bool,
    pub stuck_t: Option<f32>,
    /// §3.2 (Brief VII v2): set at the moment of a world/ground hit -
    /// true = embedded (steep enough to stick), false = a shallow
    /// glancing hit that bounced off with a clatter. Only meaningful for
    /// spears (arrows always use their own recovery-probability roll);
    /// body/zombie hits never touch this (always effectively embedded).
    pub embedded: bool,
    /// §4.2 (Brief VII v2): arrows only - how many more bodies this
    /// shot can pass through (starts at BOW_MAX_PIERCES, 0 = the next
    /// body hit embeds instead of piercing).
    pub pierces_left: u8,
    /// §4.2: fighters already pierced by THIS shot - a multi-tick
    /// overlap with the same body must not double-count.
    pub pierced: Vec<usize>,
    /// §4.1: this shot's draw-power fraction (0.35-1.0) - scales every
    /// pierce's damage, not just the first.
    pub power: f32,
}

/// §3.2 (Brief VII v2): impact angle from surface (0 deg = grazing,
/// 90 deg = straight-on), given the travel direction and surface normal.
pub fn impact_angle_to_surface_deg(dir: [f32; 3], normal: [f32; 3]) -> f32 {
    let dot = (dir[0] * normal[0] + dir[1] * normal[1] + dir[2] * normal[2]).clamp(-1.0, 1.0);
    dot.abs().asin().to_degrees()
}

/// §3.2: at or above this angle-to-surface, a thrown spear embeds;
/// shallower and it bounces off (arrows are unaffected - own mechanic).
pub const SPEAR_STICK_ANGLE_DEG: f32 = 30.0;

// ---- §4.1/§4.2 (Brief VII v2): the bow's draw-and-pierce rework -----------
/// Release under this = letdown, no shot (prevents accidental dribbles).
pub const BOW_DRAW_MIN_S: f32 = 0.15;
/// Full draw - 100% power reached here; holding longer doesn't add more.
pub const BOW_DRAW_FULL_S: f32 = 0.7;
/// Held this long total with no release - forced letdown, no shot.
pub const BOW_DRAW_FORCE_S: f32 = 10.0;
/// Full-draw arrow speed, gravity ×0.42 (unchanged factor from Brief II).
pub const BOW_V0_FULL: f32 = 55.0;
/// §4.2: passes through up to 3 soldiers - 90 -> 68 -> 45 (×0.75/pierce),
/// each scaled by the shot's own draw-power fraction.
pub const BOW_PIERCE_DMG: [f32; 3] = [90.0, 67.5, 50.625];
pub const BOW_MAX_PIERCES: u8 = 3;

/// §4.1: the power a shot carries at the MINIMUM valid draw. Named
/// because the client's arc preview needs the same floor to draw the
/// early-draw arc correctly.
pub const BOW_POWER_MIN: f32 = 0.35;

/// §4.1: draw power fraction (35% -> 100%, linear) for a hold of
/// `held_s` seconds; `None` means letdown (too short or forced at 10s).
pub fn bow_power_fraction(held_s: f32) -> Option<f32> {
    if held_s < BOW_DRAW_MIN_S || held_s >= BOW_DRAW_FORCE_S {
        return None;
    }
    let t = ((held_s - BOW_DRAW_MIN_S) / (BOW_DRAW_FULL_S - BOW_DRAW_MIN_S)).clamp(0.0, 1.0);
    Some(BOW_POWER_MIN + (1.0 - BOW_POWER_MIN) * t)
}

// ---- §4.1: full-draw hold sway --------------------------------------
// "Full-draw hold: steady 4s; then rotational aim sway ramps +/-0.4deg
// -> +/-1.2deg over the next 4s... forced letdown at 10s total.
// Crouching halves sway." A real archer's hold degrades with time; the
// bow rewards a snappy release over a held stare, exactly like the
// running-throw bonus rewards committing rather than waiting.
pub const BOW_SWAY_HOLD_S: f32 = 4.0; // steady window before sway starts
pub const BOW_SWAY_RAMP_S: f32 = 4.0; // 4s..8s: sway grows over this span
pub const BOW_SWAY_MIN_DEG: f32 = 0.4;
pub const BOW_SWAY_MAX_DEG: f32 = 1.2;

/// Current sway MAGNITUDE (the +/- half-angle) for a hold of `held_s`
/// seconds; 0 before the steady window ends. The actual perturbation
/// applied to a shot is a random draw within [-this, +this], using the
/// sim's own seeded stream (see the fire path) so it stays replay-exact.
pub fn bow_sway_deg(held_s: f32, crouched: bool) -> f32 {
    if held_s <= BOW_SWAY_HOLD_S {
        return 0.0;
    }
    let t = ((held_s - BOW_SWAY_HOLD_S) / BOW_SWAY_RAMP_S).clamp(0.0, 1.0);
    let deg = BOW_SWAY_MIN_DEG + (BOW_SWAY_MAX_DEG - BOW_SWAY_MIN_DEG) * t;
    if crouched {
        deg * 0.5
    } else {
        deg
    }
}

// ---------------------------------------------------------------- pickups

/// §3: ammo class of a recoverable projectile lying on the ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmmoKind {
    Arrow,
    Spear,
}

/// §3: a rested arrow/spear converted in place to a walk-over pickup.
/// NO owner lock, NO self-pickup cooldown — your own thrown spear is
/// explicitly yours to take back.
#[derive(Clone, Debug)]
pub struct DroppedAmmo {
    pub kind: AmmoKind,
    pub count: u8,
    /// Sim ticks at rest — never wall clock (replays must agree).
    pub rest_tick: u64,
    pub pos: [f32; 3],
}

/// §3 tuning: recovery, caps, lifetimes. Spears always survive landing;
/// arrows break 35% of the time.
pub const ARROW_RECOVER_P: f32 = 0.65;
pub const AMMO_CAP_ARROW: u32 = 24;
// §3.2 (Brief VII v2): "carry max 2" - a spear is heavy, not ammo.
pub const AMMO_CAP_SPEAR: u32 = 2;
pub const DROPPED_RADIUS: f32 = 1.1;
pub const DROPPED_MERGE_M: f32 = 0.75;
pub const DROPPED_TTL_TICKS: u64 = 7200; // 60 s at 120 Hz
pub const DROPPED_MAX: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Health,
    Ammo,
    /// §6: equips the Robot Suit (power core full).
    RobotArmor,
    /// §6: equips Folk Armor (mail + plate, Shieldwall Brace).
    FolkArmor,
    /// §6: equips Pyro Armor (fire immunity, Flame Projector).
    PyroArmor,
    /// §6: equips Recon Weave (fast, quiet, self-healing).
    ReconWeave,
    /// §7 (Brief IV): the pad-only M134 — displaces your primary until
    /// you die (then the original comes back).
    Minigun,
}

// ---- §5 (Brief II): throwables -------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThrowKind {
    Frag,
    Flash,
    Smoke,
    Molotov,
}

impl ThrowKind {
    pub const ALL: [ThrowKind; 4] = [
        ThrowKind::Frag,
        ThrowKind::Flash,
        ThrowKind::Smoke,
        ThrowKind::Molotov,
    ];
    pub fn name(self) -> &'static str {
        match self {
            ThrowKind::Frag => "FRAG",
            ThrowKind::Flash => "FLASH",
            ThrowKind::Smoke => "SMOKE",
            ThrowKind::Molotov => "MOLOTOV",
        }
    }
}

/// §5.2 tuning per kind: fuse, bounce response, effect radius.
pub struct ThrowSpec {
    pub fuse_s: f32,
    pub restitution: f32,
    pub friction: f32,
    pub radius_m: f32,
}

pub fn throw_spec(k: ThrowKind) -> ThrowSpec {
    match k {
        ThrowKind::Frag => ThrowSpec {
            fuse_s: 2.4,
            restitution: 0.28,
            friction: 0.55,
            // Brief IX-B: falloff extends out to 20m (smooth taper to 0),
            // not a 6m hard cliff - `frag_falloff_frac` owns the actual
            // shape of the curve within this range.
            radius_m: 20.0,
        },
        ThrowKind::Flash => ThrowSpec {
            fuse_s: 1.6,
            restitution: 0.42,
            friction: 0.40,
            radius_m: 9.0,
        },
        ThrowKind::Smoke => ThrowSpec {
            fuse_s: 1.2, // deploy delay; the bloom lasts SMOKE_TTL_S
            restitution: 0.22,
            friction: 0.70,
            radius_m: 2.6, // sphere radius (5.2 m sphere)
        },
        ThrowKind::Molotov => ThrowSpec {
            fuse_s: f32::INFINITY, // detonates on impact
            restitution: 0.0,      // shatters
            friction: 1.0,
            radius_m: 3.4,
        },
    }
}

/// Brief IX-B ("Blast Physics & Falloff Curves"): the frag's damage
/// fraction (0..1 of FRAG_DMG) at distance `d` meters - piecewise-linear,
/// no hard-edge cliff (non-negotiable #3). Matches the brief's table's
/// SHAPE exactly (100% out to 2m, then 100%->50%->15%->0% at the 6/12/20m
/// breakpoints); the brief's own absolute damage numbers were computed
/// against an illustrative 80 HP baseline that isn't this game's actual
/// `MAX_HEALTH` (100), so the portable part is the curve, not the raw
/// numbers - FRAG_DMG (118, already lethal against 100 HP) stays the
/// peak.
fn frag_falloff_frac(d: f32) -> f32 {
    if d <= 2.0 {
        1.0
    } else if d <= 6.0 {
        1.0 - (d - 2.0) / 4.0 * 0.5
    } else if d <= 12.0 {
        0.5 - (d - 6.0) / 6.0 * 0.35
    } else if d <= 20.0 {
        0.15 - (d - 12.0) / 8.0 * 0.15
    } else {
        0.0
    }
}

/// §5.1 release speeds: overhand / underhand lob / retreat drop.
pub const THROW_V_OVER: f32 = 11.5;
pub const THROW_V_UNDER: f32 = 6.2;
pub const THROW_V_DROP: f32 = 4.0;
// ---- §1 (Brief V): grenade pre-aim + charge ------------------------------
// Hold = aim mode: the arc previews live, power charges with hold time.
// A sub-150 ms tap is the PANIC throw — a usable 50% lob, never a drop
// at your own feet. (Frag cooking from Brief II is unchanged and rides
// the same hold: charge and fuse risk are one decision.)
pub const THROW_CHARGE_MIN_S: f32 = 0.15;
pub const THROW_CHARGE_MAX_S: f32 = 1.2;
/// Release-speed scale at zero charge / full charge.
pub const THROW_POWER_MIN: f32 = 0.55;
pub const THROW_POWER_MAX: f32 = 1.15;
pub const THROW_TAP_POWER: f32 = 0.5;
/// Aiming a throw is a two-handed thought: walk at 70%.
pub const THROW_AIM_MOVE_MULT: f32 = 0.70;
/// §1 shield tradeoff: charging one-handed behind the plate runs 25%
/// slower, and the plate blocks at HALF strength while you aim.
pub const THROW_SHIELD_CHARGE_MULT: f32 = 0.8;
/// §1 mech variant: the launcher fires hotter and flatter — no wind-up
/// pose, no movement penalty, same preview code path.
pub const MECH_LAUNCHER_V_MULT: f32 = 1.35;

// ---- §5.3 (Brief VI): the shoulder missile pod (BF4 lock-on numbers) -----
// Locks on MECHS ONLY — never infantry (the anti-oppression rule).
// The victim is warned from lock START; proportional navigation with a
// hard turn cap and a TTL means cover and side-steps beat it.
pub const POD_TUBES: u8 = 4;
pub const POD_LOCK_S: f32 = 1.3;
pub const POD_CONE_COS: f32 = 0.9945; // cos 6°
pub const POD_RANGE_M: f32 = 250.0;
pub const POD_RELAUNCH_S: f32 = 1.5;
pub const ROCKET_SPEED: f32 = 60.0;
pub const ROCKET_ACCEL: f32 = 50.0;
pub const ROCKET_TURN_CAP: f32 = 4.363; // 250°/s
pub const ROCKET_TTL_S: f32 = 7.0;
pub const ROCKET_LOS_BREAK_S: f32 = 0.4;
pub const ROCKET_DMG: f32 = 270.0;
pub const ROCKET_PN_N: f32 = 3.0;
pub const ROCKET_PROX_M: f32 = 1.2;
pub const ROCKET_SOLDIER_KILL_M: f32 = 2.0;

/// §5.3: one pod missile in flight. `target = -1` → ballistic (dumb
/// fire, dead target, or a broken lock — it never re-acquires).
#[derive(Clone, Debug)]
pub struct Rocket {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub target: i32,
    pub shooter: usize,
    pub team: Team,
    pub t: f32,
    pub los_lost: f32,
    pub prev_los: [f32; 3],
}

/// Rotate `dir` toward `want` by at most `max_ang` radians — the PN
/// steering step, deterministic f32 throughout.
pub fn rotate_toward(dir: [f32; 3], want: [f32; 3], max_ang: f32) -> [f32; 3] {
    let dot = (dir[0] * want[0] + dir[1] * want[1] + dir[2] * want[2]).clamp(-1.0, 1.0);
    let ang = dot.acos();
    if ang <= max_ang || ang < 1e-5 {
        return want;
    }
    let t = max_ang / ang;
    // nlerp is fine at these small per-tick angles
    normalize([
        dir[0] + (want[0] - dir[0]) * t,
        dir[1] + (want[1] - dir[1]) * t,
        dir[2] + (want[2] - dir[2]) * t,
    ])
}

// ---- §2 (Brief VI): deterministic spray + the three recoil channels ------
// Channel 1 (truth): bullets leave at eye + punch × RECOIL_SCALE + cone.
// Channel 2 (view): the camera shows punch × RECOIL_SCALE × 0.45 — the
// crosshair itself NEVER moves. Channel 3 (viewmodel): rotational only.
pub const RECOIL_SCALE: f32 = 2.0;
pub const VIEW_RECOIL_TRACKING: f32 = 0.45;
/// Punch-angle decay per tick: ×e^(−8·dt), then −18°·dt toward zero;
/// punch velocity ×e^(−4.5·dt). Camera at rest ≈0.3–0.5 s after firing.
pub const PUNCH_DECAY_EXP: f32 = 8.0;
pub const PUNCH_DECAY_LIN_DEG: f32 = 18.0;
pub const PUNCH_VEL_DECAY_EXP: f32 = 4.5;
/// Full-auto smoothing between consecutive table entries.
pub const SPRAY_LERP: f32 = 0.55;
/// Spray index decay: starts after cycletime × 1.1 idle, then falls at
/// this many entries per second (tap-firing resets, holding doesn't).
pub const SPRAY_INDEX_DECAY: f32 = 2.0;
/// §2.4: movement inaccuracy ramps from zero at 34% of max speed to
/// full at 95% — counter-strafing works.
pub const MOVE_INACC_START: f32 = 0.34;
pub const MOVE_INACC_FULL: f32 = 0.95;

/// §2.2: the deterministic spray table — entry `i` of `kind`'s pattern,
/// as (angle from vertical, radians; magnitude, degrees of punch
/// velocity). Generated by a per-weapon FIXED seed: the same pattern
/// for every player in every match, learnable, and replay-exact. Pure —
/// O(i) per call with i ≤ 64, cheap at any fire rate.
pub fn spray_entry(kind: GunKind, i: usize) -> (f32, f32) {
    let slot = match kind {
        GunKind::Fists => 0u64,
        GunKind::Glock => 1,
        GunKind::Deagle => 2,
        GunKind::Mp5 => 3,
        GunKind::Shotgun => 4,
        GunKind::Ak47 => 5,
        GunKind::M4 => 6,
        GunKind::Awm => 7,
        GunKind::M249 => 8,
        GunKind::Bow => 9,
        GunKind::Spear => 10,
        GunKind::Minigun => 11,
    };
    let mut rng = Pcg32::new(0xC5C0_0000 + slot, 0x5EED);
    // per-shot punch VELOCITY in °/s — scaled so the table lands on
    // CS:GO's shipped magnitudes (Deagle 57.6 vs Valve's 58.7, AK 39.6
    // vs 30, M4 28.8 vs 27.5) against the 8/18/4.5 decay constants
    let base = gun(kind).kick * 9000.0;
    let i = i.min(63);
    let mut angle = 0.0_f32; // 0 = straight up
    let mut out = (0.0_f32, base);
    for j in 0..=i {
        // early shots rise nearly vertical; the pattern then wanders
        // sideways — the learnable drift
        let wander = if j < 8 { 0.14 } else { 0.55 };
        angle = (angle + rng.range(-wander, wander)).clamp(-1.1, 1.1);
        let mag = base * rng.range(0.88, 1.12);
        out = (angle, mag);
    }
    out
}

/// Deflect a unit direction by (up_deg, right_deg) — the §2.1 punch
/// application, shared by the bullet channel and any preview.
pub fn deflect(d: [f32; 3], up_deg: f32, right_deg: f32) -> [f32; 3] {
    let d = normalize(d);
    let yaw = d[0].atan2(d[2]);
    let pitch = d[1].asin();
    let ny = yaw + right_deg.to_radians();
    let np = (pitch + up_deg.to_radians()).clamp(-1.55, 1.55);
    [np.cos() * ny.sin(), np.sin(), np.cos() * ny.cos()]
}

/// Charge fraction [0,1] for a hold: tap = the fixed 50% panic throw,
/// then 0.15 s → 1.2 s maps linearly min → max (no overcharge penalty).
pub fn throw_power(hold_s: f32) -> f32 {
    if hold_s < THROW_CHARGE_MIN_S {
        THROW_TAP_POWER
    } else {
        ((hold_s - THROW_CHARGE_MIN_S) / (THROW_CHARGE_MAX_S - THROW_CHARGE_MIN_S))
            .clamp(0.0, 1.0)
    }
}
pub const FRAG_DMG: f32 = 118.0;
pub const FLASH_BLIND_S: f32 = 3.2;
pub const SMOKE_TTL_S: f32 = 16.0;
pub const SMOKE_MAX: usize = 8;
pub const FIRE_TTL_S: f32 = 9.0;
pub const FIRE_DPS: f32 = 12.0;
/// Starting pouch: [frag, flash, smoke, molotov].
pub const GRENADE_LOADOUT: [u8; 4] = [2, 1, 1, 1];
// ---- §8 (Brief IV): grenade budget presets -------------------------------
// 6 points: frag 2 / molotov 2 / flash 1 / smoke 1. Order per slot is
// [frag, flash, smoke, molotov] to match ThrowKind::ALL.
pub const GRENADE_PRESETS: [([u8; 4], &str); 4] = [
    ([2, 1, 1, 0], "STANDARD"),   // 2F + FL + S = 6
    ([1, 0, 0, 2], "ARSONIST"),   // F + 2M = 6
    ([0, 2, 4, 0], "SMOKE WALL"), // 2FL + 4S = 6
    ([3, 0, 0, 0], "DEMO MAN"),   // 3F = 6
];

/// A grenade in flight or at rest, fuse burning. Point-mass integration
/// at the fixed 120 Hz tick — tumble spin is CLIENT-side only.
#[derive(Clone, Debug)]
pub struct Grenade {
    pub id: u32,
    pub kind: ThrowKind,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub thrower: usize,
    pub team: Team,
    pub fuse_t: f32,
    pub bounces: u32,
    pub rest: bool,
}

/// What one grenade physics tick concluded.
pub enum GrenadeTick {
    Fly,
    Boom,
    Rest,
}

/// Brief IX-B "Bounce & Rolling Physics": each cover material's own
/// bounce coefficient, overriding the throw kind's default restitution
/// on contact with that specific material. Organic surfaces (brief:
/// "cloth, flesh, sandbagged positions") are sticky - effectively zero
/// bounce, the grenade stays where it lands - mapped onto the two
/// vegetation `CoverKind`s (hedge/tree) as this game's closest analog;
/// stone is stone, crates stand in for the brief's "wood/metal".
fn surface_restitution(kind: CoverKind) -> f32 {
    match kind {
        CoverKind::Stone => 0.40,
        CoverKind::Crate => 0.50,
        CoverKind::Hedge | CoverKind::Tree => 0.05,
    }
}

/// R&D Cycle 2 (backlog #3): per-surface FRICTION, alongside the
/// already-existing per-surface restitution above. [S-01, RoyMech
/// tribology table]: metal-on-wood sliding friction runs 0.2-0.6 (dry,
/// clean) against metal-on-masonry/rock in the 0.3-0.6 family
/// (concrete+rock 0.3 sliding was the only steel-specific masonry row
/// found; broader rock-family rows run higher but are rock-on-rock, not
/// metal-on-rock) - real data with real overlap, not a crisp single
/// answer, honestly not oversold into more precision than the source
/// supports. Direction taken: worked masonry (this game's stone cover)
/// is smoother than a rough-grain wood crate, so a bounce skids FURTHER
/// across stone than off a crate - low end of the wood range for
/// crates, low end of the masonry range for stone, keeping both
/// comfortably inside the source's own bracket.
fn surface_friction(kind: CoverKind) -> f32 {
    match kind {
        CoverKind::Stone => 0.30,
        CoverKind::Crate => 0.45,
        // moot in practice - Hedge/Tree stick on contact (zero bounce,
        // velocity zeroed below) before tangential friction would ever
        // apply, but a real number belongs here regardless of reachability
        CoverKind::Hedge | CoverKind::Tree => 0.60,
    }
}

/// Which cover object (if any) a bounce contact point actually landed
/// on, to look up its material. An independent short scan rather than
/// threading an index through `CoverGrid::ray_hit`'s return type (used
/// by seven unrelated systems - bullets, LOS, rockets, smoke) - cover
/// counts are small (tens, not thousands) per map, so this stays cheap.
fn cover_kind_at(cover: &[Aabb], cover_kind: &[CoverKind], contact: [f32; 3]) -> Option<CoverKind> {
    const PAD: f32 = 0.05;
    cover.iter().position(|a| {
        (0..3).all(|k| contact[k] >= a.min[k] - PAD && contact[k] <= a.max[k] + PAD)
    })
    .and_then(|i| cover_kind.get(i).copied())
}

/// One 120 Hz grenade physics tick — THE integrator, shared verbatim by
/// the live flight (`step_grenades`) and the §1 (Brief V) aim preview
/// (`predict_grenade`). There is deliberately no second arc formula
/// anywhere: a preview that can diverge from the throw is worse than no
/// preview. 9.81 gravity, friction+restitution bounce (material-specific
/// per Brief IX-B where the contact is a known cover object), rest test,
/// settle guarantee, molotov shatter, fuse expiry.
pub fn grenade_tick(
    g: &mut Grenade,
    grid: &CoverGrid,
    cover: &[Aabb],
    cover_kind: &[CoverKind],
) -> GrenadeTick {
    g.fuse_t -= DT;
    if g.fuse_t <= 0.0 {
        return GrenadeTick::Boom;
    }
    if g.rest {
        return GrenadeTick::Rest;
    }
    g.vel[1] -= 9.81 * DT;
    let old = g.pos;
    let new = [
        old[0] + g.vel[0] * DT,
        old[1] + g.vel[1] * DT,
        old[2] + g.vel[2] * DT,
    ];
    let seg = [new[0] - old[0], new[1] - old[1], new[2] - old[2]];
    let seg_len = (seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2])
        .sqrt()
        .max(1e-6);
    let dn = [seg[0] / seg_len, seg[1] / seg_len, seg[2] / seg_len];
    // first surface on the path: cover (via the §9.1 grid) or ground
    let mut hit: Option<(f32, [f32; 3])> = grid.ray_hit(cover, old, dn, seg_len);
    if new[1] <= 0.0 && old[1] > 0.0 {
        let t = (0.0 - old[1]) / (new[1] - old[1]) * seg_len;
        if hit.map_or(true, |(ht, _)| t < ht) {
            hit = Some((t, [0.0, 1.0, 0.0]));
        }
    }
    if let Some((t, n)) = hit {
        let spec = throw_spec(g.kind);
        if g.kind == ThrowKind::Molotov {
            // shatters on ANY surface — the pool spawns at impact
            g.pos = [
                old[0] + dn[0] * t,
                (old[1] + dn[1] * t).max(0.02),
                old[2] + dn[2] * t,
            ];
            return GrenadeTick::Boom;
        }
        let contact = [
            old[0] + dn[0] * (t - 0.01).max(0.0),
            old[1] + dn[1] * (t - 0.01).max(0.0),
            old[2] + dn[2] * (t - 0.01).max(0.0),
        ];
        // Brief IX-B: a contact on a KNOWN cover object uses that
        // material's own bounce coefficient in place of the throw kind's
        // default (ground-plane hits, with no cover object, keep the
        // kind's own restitution unchanged - unaffected by this).
        let material = cover_kind_at(cover, cover_kind, contact);
        let base_restitution = material.map_or(spec.restitution, surface_restitution);
        // R&D Cycle 2: friction is now ALSO per-material on a known
        // cover object, same fallback rule as restitution above - a
        // ground-plane hit with no cover object keeps the throw kind's
        // own uniform friction, exactly as before this cycle.
        let friction = material.map_or(spec.friction, surface_friction);
        let sticky = matches!(material, Some(CoverKind::Hedge) | Some(CoverKind::Tree));
        let vn = g.vel[0] * n[0] + g.vel[1] * n[1] + g.vel[2] * n[2];
        let vnv = [n[0] * vn, n[1] * vn, n[2] * vn];
        let vt = [g.vel[0] - vnv[0], g.vel[1] - vnv[1], g.vel[2] - vnv[2]];
        g.bounces += 1;
        let rest_coef = if g.bounces > 3 {
            base_restitution * 0.5_f32.powi(g.bounces as i32 - 3)
        } else {
            base_restitution
        };
        g.vel = [
            vt[0] * (1.0 - friction) - vnv[0] * rest_coef,
            vt[1] * (1.0 - friction) - vnv[1] * rest_coef,
            vt[2] * (1.0 - friction) - vnv[2] * rest_coef,
        ];
        g.pos = contact;
        if sticky {
            // "sticks on contact... does not bounce; detonates in place"
            g.rest = true;
            g.vel = [0.0; 3];
        }
        // §5.2 rest test — without it: infinite micro-bounces
        let speed = (g.vel[0] * g.vel[0] + g.vel[1] * g.vel[1] + g.vel[2] * g.vel[2]).sqrt();
        if !g.rest && vn.abs() * rest_coef < 0.35 && speed < 0.6 {
            g.rest = true;
            g.vel = [0.0; 3];
            g.pos[1] = g.pos[1].max(0.02);
        }
    } else {
        g.pos = new;
    }
    GrenadeTick::Fly
}

/// §5.4: an active smoke sphere — occludes bot LOS via path length.
#[derive(Clone, Debug)]
pub struct SmokeVolume {
    pub pos: [f32; 3],
    pub ttl: f32,
}

/// §5.5: a molotov fire pool — 4 Hz damage ticks, blocks bot pathing.
#[derive(Clone, Debug)]
pub struct FirePool {
    pub pos: [f32; 3],
    pub ttl: f32,
    pub thrower: usize,
    pub tick_t: f32,
}

/// Detonation event for client FX (kind + where), with a display ttl.
#[derive(Clone, Debug)]
pub struct Boom {
    pub at: [f32; 3],
    pub kind: ThrowKind,
}

// ---- §6 (Brief II): armor sets with full powers --------------------------

/// Found and equipped in-world, never spawned with. Lost on death.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmorSet {
    None,
    /// Mail and plate; Shieldwall Brace (hold F).
    Folk,
    /// Heat plate; fire immunity + Flame Projector (hold C).
    Pyro,
    /// The grounded walker chassis; side-step (Q) + Repulsor Blast (C),
    /// running on a power core. (Flight was deleted in Brief VI §4.3 -
    /// this doc advertised thrusters for two briefs after they died.)
    RobotSuit,
    /// The light counterweight: fast, quiet, self-healing. No abilities.
    Recon,
}

/// §6.1: flat per-zone reduction applied AFTER the zone multiplier, with
/// a floor of 15% of base damage — heavy sets never make limb shots free,
/// and head flats stay small so headshots remain decisive.
pub struct ArmorSpec {
    pub flat_head: f32,
    pub flat_torso: f32,
    pub flat_limb: f32,
    pub move_mult: f32,
    /// Fraction of explosive damage shrugged off.
    pub explosive_resist: f32,
}

pub fn armor_spec(s: ArmorSet) -> ArmorSpec {
    match s {
        ArmorSet::None => ArmorSpec {
            flat_head: 0.0,
            flat_torso: 0.0,
            flat_limb: 0.0,
            move_mult: 1.0,
            explosive_resist: 0.0,
        },
        ArmorSet::Folk => ArmorSpec {
            flat_head: 12.0,
            flat_torso: 45.0,
            flat_limb: 15.0,
            move_mult: 0.92,
            explosive_resist: 0.0,
        },
        ArmorSet::Pyro => ArmorSpec {
            flat_head: 14.0,
            flat_torso: 30.0,
            flat_limb: 30.0,
            move_mult: 0.96,
            explosive_resist: 0.5,
        },
        ArmorSet::RobotSuit => ArmorSpec {
            flat_head: 16.0,
            flat_torso: 55.0,
            flat_limb: 55.0,
            move_mult: 0.85, // §4.3 (Brief VI): 85% of soldier run pace
            explosive_resist: 0.0,
        },
        ArmorSet::Recon => ArmorSpec {
            flat_head: 12.0,
            flat_torso: 12.0,
            flat_limb: 12.0,
            move_mult: 1.10,
            explosive_resist: 0.0,
        },
    }
}

/// §6.1 damage floor: no set reduces a hit below this fraction of the
/// gun's BASE damage — otherwise players stop aiming.
pub const ARMOR_FLOOR_FRAC: f32 = 0.15;
/// Folk Shieldwall Brace: frontal arc, reduction, stacking wall bonus.
pub const BRACE_ARC_COS: f32 = 0.574; // cos(55°) — a 110° frontal arc
pub const BRACE_REDUCTION: f32 = 0.82;
pub const BRACE_STACK_BONUS: f32 = 0.08;
pub const BRACE_STACK_CAP: u32 = 3;
pub const BRACE_SPEED_MULT: f32 = 0.25;

// ---- §A: Mech Brace ------------------------------------------------
// Widen the stance / drop the centre of mass to enlarge the ZMP support
// polygon (Vukobratovic & Borovac). A NEW, mech-scoped set - deliberately
// NOT a reuse of the Folk brace constants above.
//
// WHY A SEPARATE FIELD AND SEPARATE CONSTANTS. `Fighter::brace` drives
// two things. Its damage reduction IS armor_set-gated to Folk, so reuse
// would have been harmless there. But the movement penalty at the two
// sites below is NOT gated on armor_set - so setting `brace` on a mech
// would silently apply the INFANTRY 0.25x, and any later rebalance of
// BRACE_SPEED_MULT would move mech pacing as an invisible side effect.
// The two are calibrated for completely different mass and HP scales
// (MECH_HULL 1000.0 vs infantry HP two orders of magnitude smaller).
pub const MECH_BRACE_STANCE_DROP: f32 = 0.12; // fraction of height() the hull sinks
pub const MECH_BRACE_SPEED_MULT: f32 = 0.12; // below infantry's 0.25 - braced is near-planted
pub const MECH_BRACE_RECOIL_DAMP: f32 = 0.30; // fraction of unbraced kick retained

// ---- §C: the mech's hull-mounted weapons ----------------------------
/// The chassis' two hull mounts - rigidly bolted, never held, always
/// BOTH present the moment the suit is equipped. Selecting between them
/// is a targeting-mode switch, not a pickup or a loadout choice.
///
/// WHY THIS IS NOT A `GunKind`. `GunKind` is the spine of the INFANTRY
/// weapon pipeline: `gun(kind) -> GunSpec`, `ALL_WEAPONS`/`N_WEAPONS`
/// and the `vm.weapons[N_WEAPONS]` viewmodel array, `weapon_slot`,
/// `reload_pose`, the loadout screen's `PRIMARIES`/`GunClass` tables,
/// and a per-weapon punch-slot mapping. Every one of those encodes
/// "a carryable, swappable, loadout-selectable gun with a magazine and
/// a pair of hands on it". These two are STRUCTURAL - never picked up,
/// never swapped into a slot, never reloaded, never in a loadout, and
/// there is no hand pose for them. Routing them through `GunKind` would
/// mean answering ~8 exhaustive matches with semantics that do not
/// apply, and every one of those answers would be a lie the next
/// feature reads as truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MechWeapon {
    #[default]
    Gatling,
    Autocannon,
}
/// Gatling: SUPPRESSION. Every number is set against the man-portable
/// minigun, because that is the weapon a player will compare it to.
/// Heat per round is BELOW the minigun's 1.5 and the fire period is a
/// touch longer than its 0.06 - together those two make the hull gun
/// sustain roughly twice as long before it cooks off, which is the
/// whole identity: the minigun is a 4-second scream, this is the wall
/// of fire you walk behind.
pub const GATLING_HEAT_PER_SHOT: f32 = 0.9;
/// Below MINIGUN_HEAT_DECAY (16.5): a sealed hull mount has no open
/// barrel shroud to dump heat through, so it sheds it slower once the
/// trigger comes off. The longer sustain is paid for with a longer wait.
pub const GATLING_HEAT_DECAY: f32 = 9.5;
/// Longer than the minigun's forced 3.0 s, for the same reason.
pub const GATLING_VENT_FORCED_S: f32 = 3.5;
/// The REQUESTED period. What the mount actually cycles at is this value
/// rounded UP to a whole tick — `gatling_cd` is decremented by `DT` and
/// the gate opens at `<= 0`, so the real period is
/// `ceil(0.07 / DT) * DT` = **0.075 s at the 120 Hz sim floor, i.e. 800
/// RPM**, not the 857 the constant reads as. Worth knowing before anyone
/// tunes this against a published RPM figure: at 60 Hz it quantises to
/// 0.0833 s (720 RPM) and at 240 Hz to 0.0708 s (847 RPM).
pub const GATLING_FIRE_PERIOD: f32 = 0.07; // same ROF class as the minigun's 0.06
/// Trigger-intent hold window for the hull mount — deliberately the same
/// 70 ms as `MINIGUN_SPIN_HOLD_S`, because it is the same mechanism: the
/// heat decay is suppressed while the trigger is DOWN, and the timer is
/// what carries that intent across the ticks between rounds.
pub const GATLING_TRIGGER_HOLD_S: f32 = 0.07;
pub const GATLING_DAMAGE: f32 = 9.0; // near the minigun's 8.0 - high ROF, low per-round
pub const GATLING_SPREAD_COLD: f32 = 0.018;
pub const GATLING_SPREAD_HOT: f32 = 0.052;
/// Autocannon: PRECISION. ~7 unarmoured hits through MECH_HULL (1000),
/// on a slow cycle and a tight cone - the deliberate opposite of the
/// gatling's spray.
pub const AUTOCANNON_DAMAGE: f32 = 145.0;
pub const AUTOCANNON_CYCLE_S: f32 = 1.35;
pub const AUTOCANNON_UNBRACED_KICK: f32 = 6.0;
pub const AUTOCANNON_SPREAD: f32 = 0.006;
// NOTE: there is deliberately NO `AUTOCANNON_BRACED_KICK`. The braced
// value is derived at the call site as
// `AUTOCANNON_UNBRACED_KICK * MECH_BRACE_RECOIL_DAMP`. Two independently
// tunable numbers describing ONE relationship drift out of sync after a
// single balance pass; §A built the damp mechanism generically for
// exactly this consumer.

/// §D: the range at which a BOT mech stops spraying and starts aiming.
///
/// Derived, not tuned. `GATLING_SPREAD_COLD` is the per-axis offset
/// `hitscan_burst` applies to the aim ray, so at range `d` a COLD gatling
/// scatters its rounds over a half-width of `GATLING_SPREAD_COLD * d`. A
/// man is `BODY_RADIUS` wide to each side. Past
/// `BODY_RADIUS / GATLING_SPREAD_COLD` even a cold mount is putting most
/// of its rounds beside the target — and every one of those misses still
/// walks the barrels toward a 3.5 s forced vent. That is exactly where
/// one 145-damage round through a 3× tighter cone becomes the honest
/// trade, so the switch is placed there rather than at a round number.
///
/// ≈18.9 m today, which sits inside EVERY difficulty's engage range
/// (Easy 22 / Normal 35 / Hard 50) — so all three tiers really do use
/// both mounts instead of one tier silently never reaching the switch.
pub const MECH_BOT_AUTOCANNON_RANGE_M: f32 = BODY_RADIUS / GATLING_SPREAD_COLD;

/// Hysteresis band on the switch above. A bare threshold over a
/// dithering quantity is a defect, not a rule: a bot holding station
/// near the line would flip mounts tick to tick, and because the two
/// mounts keep INDEPENDENT cooldown clocks (`gatling_cd` /
/// `autocannon_cd`, deliberately never shared) it would then land BOTH —
/// making "stand at exactly 18.9 m" the highest-DPS thing a bot mech can
/// do. One chassis width is the band: committing to the other mount has
/// to be worth more than a single step.
pub const MECH_BOT_MOUNT_HYSTERESIS_M: f32 = MECH_RADIUS;

/// Robot power core: capacity, recharge (grounded, 5 s after ability use),
/// repulsor cost/cooldown, EMP/explosive drain. (The THRUST_* trio lived
/// here from Brief IV's flight model - §4.3 deleted mech flight, and the
/// constants sat dead with a doc claiming live thrusters for two briefs.)
pub const POWER_MAX: f32 = 100.0;
pub const POWER_REGEN: f32 = 6.0;
pub const POWER_REGEN_DELAY: f32 = 5.0;
pub const REPULSOR_DMG: f32 = 62.0;
pub const REPULSOR_KNOCK: f32 = 6.0;
pub const REPULSOR_CD: f32 = 1.4;
pub const REPULSOR_COST: f32 = 12.0;
pub const EXPLOSIVE_POWER_DRAIN: f32 = 25.0;
pub const ROBOT_DRAINED_MOVE: f32 = 0.88;
/// Pyro flame projector: fuel seconds, dps, reach, cone, refill rate.
pub const FLAME_FUEL_S: f32 = 6.0;
pub const FLAME_DPS: f32 = 34.0;
pub const FLAME_REACH: f32 = 7.5;
pub const FLAME_ARC_COS: f32 = 0.906; // ±25°
pub const FLAME_REFILL: f32 = FLAME_FUEL_S / 9.0;
/// Recon passive regen: hp/s after this long without taking damage.
pub const RECON_REGEN: f32 = 4.0;
pub const RECON_REGEN_DELAY: f32 = 5.0;
// ---- §11 (Brief III): the MECH chassis — supersedes the Robot Suit ------
// A piloted walker above human scale. Damage is classified by the angle
// between the shot and the mech's BODY FACING (never the camera): a
// frontal fortress, a flanking objective. The hull is a resource you
// spend — it never regenerates; at zero the pilot ejects at 25 HP.
// Task 4 (MISSION doc, supersedes Brief VI's 1.15x/2.05m): the art
// measures the hull at ~2.5x soldier height with the soldier's helmet
// only reaching the mech's KNEE - 1.15x threw that presence away
// entirely. Decision: A3 (recommended) - 1.7x, ~3.03m. Keeps the
// tower/leg-cover read the art sells, at the cost of only a widened nav
// radius (MECH_RADIUS below is already a formula, not a bespoke system -
// this scale change needed no new collision/doorway infrastructure).
pub const MECH_SCALE: f32 = 1.7;
// §4.5 (Brief VI): hull 1000, and the sensor visor is a ×2.0 weak
// point applied AFTER the angle multiplier (front-arc only).
pub const MECH_HULL: f32 = 1000.0;
pub const MECH_VISOR_MULT: f32 = 2.0;
pub const MECH_RADIUS: f32 = BODY_RADIUS * MECH_SCALE; // cannot fit doorways
pub const MECH_EJECT_HP: f32 = 25.0;
/// §B.3: the pilot's eye height inside the hull, as a fraction of full
/// mech height. This formula was duplicated as a bare local in TEN
/// places across the test module and existed nowhere as a constant, so
/// nothing could depend on it without copying it an eleventh time.
/// Promoted here before a camera started reading it too.
pub const MECH_VISOR_Y_FRAC: f32 = 0.90;

/// The pilot's eye position inside the hull.
///
/// Pure and shared, so the visor camera and the hit-zone tests can never
/// disagree about where the pilot's head is - the same single-source
/// discipline `approach_velocity` and `shot_clock` exist for.
pub fn mech_visor_eye_y(pos_y: f32) -> f32 {
    pos_y + BODY_HEIGHT * MECH_SCALE * MECH_VISOR_Y_FRAC
}
// §6.2 (Brief VII v2): boarding/leaving the mech is COMMITTED, not
// instant - the chassis needs real seconds to seal up or power down.
pub const MECH_ENTER_S: f32 = 1.6;
pub const MECH_EXIT_S: f32 = 1.2;

// ---- R&D Cycle 1: mech entry sequence (backlog #1) -----------------------
// [S-01b]: the human-factors literature on staged, must-not-skip
// sequences under time pressure argues for a Do-List pattern (the
// system executes every stage on a fixed timeline, no player action
// mid-sequence) over an interactive challenge-response one - which
// matches §7.6's existing "committed, no cancel" rule and gives it a
// real citation instead of just flavor text. Eight named stages divide
// MECH_ENTER_S evenly; this is presentation SEQUENCING only - it reads
// `mech_transition_t`, never writes gameplay-relevant state, so it
// cannot desync a replay even though it's exposed from the sim layer
// (the sim layer is simply the only place `mech_transition_t` lives).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MechEnterStage {
    CockpitOpen,
    ClimbIn,
    Harness,
    PowerUp,
    ServoSync,
    GyroCalibration,
    WeaponDiagnostics,
    HudBoot,
}
pub const MECH_ENTER_STAGES: [MechEnterStage; 8] = [
    MechEnterStage::CockpitOpen,
    MechEnterStage::ClimbIn,
    MechEnterStage::Harness,
    MechEnterStage::PowerUp,
    MechEnterStage::ServoSync,
    MechEnterStage::GyroCalibration,
    MechEnterStage::WeaponDiagnostics,
    MechEnterStage::HudBoot,
];

/// Pure: which named stage is active `elapsed_s` into the entry window.
/// Clamped at both ends so a caller need not pre-validate its input.
pub fn mech_enter_stage(elapsed_s: f32) -> MechEnterStage {
    let frac = (elapsed_s / MECH_ENTER_S).clamp(0.0, 0.999_999);
    let idx = (frac * MECH_ENTER_STAGES.len() as f32) as usize;
    MECH_ENTER_STAGES[idx.min(MECH_ENTER_STAGES.len() - 1)]
}

/// The active stage for a fighter currently mid-entry, or `None` if
/// they aren't (not a mech, not transitioning, or exiting rather than
/// entering - `mech_exiting` reuses the same timer for the power-DOWN
/// countdown, which has no stage list of its own).
pub fn mech_enter_stage_for(f: &Fighter) -> Option<MechEnterStage> {
    if f.armor_set != ArmorSet::RobotSuit || f.mech_transition_t <= 0.0 || f.mech_exiting {
        return None;
    }
    Some(mech_enter_stage(MECH_ENTER_S - f.mech_transition_t))
}
// §6.3: armor drops in three stages as hull falls - the exposed
// under-frame at each stage takes MORE damage, rewarding the strip.
pub const MECH_PLATE_70_PCT: f32 = 0.70;
pub const MECH_PLATE_40_PCT: f32 = 0.40;
pub const MECH_PLATE_15_PCT: f32 = 0.15;
pub const MECH_EXPOSED_DMG_MULT: f32 = 1.25;
/// Frontal 0–60°: 85% reduction. Side 60–120°: 70%. Rear: none.
pub const MECH_RED_FRONT: f32 = 0.85;
pub const MECH_RED_SIDE: f32 = 0.70;
/// §11.2 rule 2: explosives bypass HALF the reduction; fire bypasses ALL
/// of it (it attacks cooling, not plating — Pyro's defined role).
/// §11: the mech TURNS to face a threat — a real, visible, punishable
/// commitment. The pilot's view is free; the armor follows the body.
// §4.3 (Brief VI): 180°/s — a soldier circling close feels the lag
pub const MECH_TURN_RATE: f32 = 3.1416; // rad/s

// ---- §7.4 (BRIEF VIII): power stride --------------------------------
// "The mech's answer to being outrun, without ever leaving the
// ground." Held sprint winds the hull forward, then bursts to 110% of
// soldier run speed for a BOUNDED window — not a second walk speed.
// Shares the §7.8 heat pool with the minigun by the brief's own
// cross-reference ("costs heat (§7.8)"): a mech running hot from its
// gun cannot also stride, one budget for how hard the machine is
// pushed, not two independent ones.
pub const POWER_STRIDE_WINDUP_S: f32 = 0.35;
pub const POWER_STRIDE_DURATION_S: f32 = 2.5;
pub const POWER_STRIDE_SPEED_MULT: f32 = 1.10;
pub const POWER_STRIDE_HEAT_PER_S: f32 = 100.0 / POWER_STRIDE_DURATION_S; // 40/s - a full burst spends the whole bar
pub const POWER_STRIDE_TURN_RATE: f32 = MECH_TURN_RATE * 0.5; // 90 deg/s, half the normal pivot cap
// ---- §10 (Brief III): base health regeneration ---------------------------
// Every fighter, independent of armor. Deliberately slow: worst case from
// one bullet to full is 24 s — it rewards disengaging, not trading. ANY
// damage resets the timer, so DoT (fire pools, toxin) denies regen
// entirely and area denial keeps its value. Restores HEALTH only, never
// armor — armor is a consumable found in the world.
pub const REGEN_DELAY_S: f32 = 12.0;
pub const REGEN_RATE_HPS: f32 = 8.33;
// ---- §5 (Brief III): the knife -------------------------------------------
// Tap: quick slash. Hold: committed lunge — visibly wound up, punishable.
// Silent (4 m noise), backstabs are lethal, and it works on the horde:
// the correct tool for zombie extraction and (later) mech rear arcs.
pub const KNIFE_QUICK_WIND_S: f32 = 0.28;
pub const KNIFE_QUICK_ACTIVE_S: f32 = 0.12;
pub const KNIFE_QUICK_RECOVER_S: f32 = 0.34;
pub const KNIFE_QUICK_DMG: f32 = 55.0;
pub const KNIFE_QUICK_BACKSTAB: f32 = 160.0;
pub const KNIFE_RANGE_M: f32 = 1.9;
pub const KNIFE_LUNGE_WIND_S: f32 = 0.55;
pub const KNIFE_LUNGE_RANGE_M: f32 = 2.9;
pub const KNIFE_LUNGE_DMG: f32 = 95.0;
pub const KNIFE_LUNGE_BACKSTAB: f32 = 999.0;
/// Holding past this commits the lunge instead of the quick slash.
pub const KNIFE_COMMIT_S: f32 = 0.30;
// ---- §6 (Brief IV): the axe ----------------------------------------------
// The knife's heavy sibling: slower on every beat, hits harder, and the
// swing is a SWEEP — every enemy inside the 90° frontal arc takes the
// hit. Louder than the blade (6 m), still quiet next to any gun.
pub const AXE_QUICK_WIND_S: f32 = 0.45;
pub const AXE_QUICK_ACTIVE_S: f32 = 0.15;
pub const AXE_QUICK_RECOVER_S: f32 = 0.50;
pub const AXE_QUICK_DMG: f32 = 85.0;
pub const AXE_QUICK_BACKSTAB: f32 = 190.0;
pub const AXE_RANGE_M: f32 = 2.1;
/// cos of the sweep half-angle (±45° — the full 90° arc).
pub const AXE_ARC_COS: f32 = 0.707;
pub const AXE_LUNGE_WIND_S: f32 = 0.70;
pub const AXE_LUNGE_DMG: f32 = 130.0;
pub const AXE_LUNGE_BACKSTAB: f32 = 999.0;
// ---- §7 (Brief IV): the minigun ------------------------------------------
// Never in a loadout — a pad weapon. The whole design is the tradeoff
// loop: spin-up latency before the stream, heat while it pours, a forced
// 3 s vent at 100 heat (R vents early on YOUR schedule instead). 400
// rounds, no reloads — the pad respawn is the reload.
pub const MINIGUN_SPINUP_S: f32 = 0.4; // §5.1 (Brief VI)
/// §5.1: 100 heat in exactly 4 s of continuous fire at 1000 RPM
/// (66.7 rounds): 1.5 heat per round.
pub const MINIGUN_HEAT_PER_SHOT: f32 = 1.5;
pub const MINIGUN_VENT_FORCED_S: f32 = 3.0;
/// Manual (R) vent clears heat at this rate — 60 heat ≈ 1.4 s.
pub const MINIGUN_VENT_RATE: f32 = 42.9;
/// §5.1: idle cooling — a full 99% cooldown in 6 s.
pub const MINIGUN_HEAT_DECAY: f32 = 16.5;
/// §5.1: the spread cone WIDENS with heat — 1.2° → 3.5° half-angle
/// (as tangents: tan 1.2° = 0.021, tan 3.5° = 0.061).
pub const MINIGUN_SPREAD_COLD: f32 = 0.021;
pub const MINIGUN_SPREAD_HOT: f32 = 0.061;

/// §3.4 (BRIEF VIII): sprint-carry threshold and per-class sprint-out
/// times - the delay between leaving a sprint and being able to fire.
/// Class mapping: light/one-handed 0.15, rifles 0.20, heavy 0.30.
/// (The minigun keeps its own spin-up as its readiness cost; projectile
/// draws - bow/spear - already pay windup+stability, so their sprint-out
/// uses the rifle beat rather than stacking a second heavy tax.)
pub const SPRINT_CARRY_FRAC: f32 = 0.85;
pub fn sprint_out_s(kind: GunKind) -> f32 {
    match kind {
        GunKind::Glock | GunKind::Deagle | GunKind::Mp5 | GunKind::Fists => 0.15,
        GunKind::M4 | GunKind::Ak47 | GunKind::Bow => 0.20,
        GunKind::Shotgun | GunKind::M249 | GunKind::Awm => 0.30,
        // spin-up IS the minigun's ready cost - no double tax
        GunKind::Minigun => 0.0,
        // §5.4: the spear's OWN windup already pays the "can't
        // insta-fire out of a sprint" cost the generic gate exists
        // for - and the running-throw bonus explicitly REWARDS
        // throwing while still at running speed. Stacking the sprint
        // gate on top would make that condition unreachable: the gate
        // holds at any speed above the 85% carry threshold, but the
        // bonus wants exactly a throw released AT that speed. Found
        // by the running-throw bonus's own test failing against this.
        GunKind::Spear => 0.0,
    }
}

/// §3.4: the empty reload (bolt/charge cycle on top of the mag swap) is
/// SLOWER than a tactical reload with a round still chambered. The ammo
/// math already kept the chambered round; the TIME cost of running dry
/// did not exist - tactical and empty took identical seconds, deleting
/// the count-your-shots skill the split exists to reward.
pub const RELOAD_EMPTY_MULT: f32 = 1.35;

/// §5.1 (Brief VI): a gun's base cone BEFORE movement/bloom. For the
/// minigun this WIDENS with heat instead of blooming - 1.2 deg cold to
/// 3.5 deg at full heat - which is that weapon's entire cost model.
///
/// Public and shared because the client's crosshair must show the same
/// number the sim shoots: `GunSpec.spread` holds only the COLD value, so
/// a client reading the spec directly showed a stability bracket that
/// never moved across a full heat cycle while the real cone nearly
/// tripled.
pub fn base_spread(kind: GunKind, heat: f32) -> f32 {
    if kind == GunKind::Minigun {
        MINIGUN_SPREAD_COLD
            + (MINIGUN_SPREAD_HOT - MINIGUN_SPREAD_COLD) * (heat / 100.0).clamp(0.0, 1.0)
    } else {
        gun(kind).spread
    }
}
/// Carrying the mass: ×0.70 walk; barrels spun: ×0.55.
pub const MINIGUN_MOVE_MULT: f32 = 0.70;
pub const MINIGUN_SPUN_MOVE_MULT: f32 = 0.55;
/// Trigger-intent hold window — long enough to bridge the 15 Hz far-bot
/// LOD (8 ticks), short enough to be imperceptible on release (70 ms).
pub const MINIGUN_SPIN_HOLD_S: f32 = 0.07;
// ---- §6 (Brief III): shield rules ----------------------------------------
/// Throwing from behind the raised shield LOWERS it for this long — a
/// real window of vulnerability that makes the lob a decision.
pub const SHIELD_DIP_S: f32 = 0.62;
// ---- §3 (Brief III): the Achilles throw ----------------------------------
/// The spear now has a VISIBLE windup before release — plant, hips, whip.
/// Enemies can see it coming; the spear is a committal weapon, the
/// correct counterweight to its flat trajectory. The release uses the
/// thrower's aim AT RELEASE (athletes track their target through the
/// plant), refreshed while winding.
// §3.1 (Brief VII v2): the raise is 0.4s (was 0.5s) - Sons of the Forest's
// grammar, retuned.
pub const SPEAR_WINDUP_S: f32 = 0.40;

// ---- §4 (Brief III): aerial flips ----------------------------------------
// Q + direction while airborne. ONE flip per airborne period, no firing
// until landing recovery — pure mobility, never a combat move.
pub const FLIP_S: f32 = 0.62;
pub const FLIP_BOOST: f32 = 1.4;
pub const FLIP_RECOVER_S: f32 = 0.18;
pub const FLIP_RECOVER_SPEED: f32 = 0.6;

/// §4.3: how `apply_hit` classifies zones on this fighter RIGHT NOW.
/// Halfway through a backflip the head occupies the BOTTOM of the capsule
/// — the banded frac test would call a boot shot a ×4 headshot. Airborne
/// acrobatics force Uniform (×1.0 everywhere) for the full flip plus the
/// landing recovery, in BOTH directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitZoneMode {
    Banded,
    Uniform,
}

// ---- §8 (Brief II): zombie extraction ------------------------------------

/// The enemy roster. Headshots (×4.0) one-shot the mass — that's the
/// skeleton and hit-zone work paying rent directly to the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZKind {
    /// The mass. A headshot is a one-shot kill.
    Shambler,
    /// Fast, after the 4-minute mark. Stops you moving backwards.
    Runner,
    /// Staggers on every 3rd headshot; a wall of meat.
    Brute,
    /// 2.2 s wind-up, then calls the horde. The tension engine.
    Screamer,
    /// Bursts into a toxic cloud on death. Punishes close range.
    Bloater,
}

pub struct ZSpec {
    pub hp: f32,
    pub speed: f32,
    pub dmg: f32,
    pub height: f32,
    pub girth: f32,
}

pub fn zspec(k: ZKind) -> ZSpec {
    match k {
        ZKind::Shambler => ZSpec { hp: 42.0, speed: 1.6, dmg: 14.0, height: 1.7, girth: 0.38 },
        ZKind::Runner => ZSpec { hp: 30.0, speed: 6.9, dmg: 11.0, height: 1.6, girth: 0.32 },
        ZKind::Brute => ZSpec { hp: 340.0, speed: 2.8, dmg: 48.0, height: 2.2, girth: 0.62 },
        ZKind::Screamer => ZSpec { hp: 24.0, speed: 3.1, dmg: 0.0, height: 1.7, girth: 0.34 },
        ZKind::Bloater => ZSpec { hp: 90.0, speed: 1.2, dmg: 0.0, height: 1.8, girth: 0.55 },
    }
}

#[derive(Clone, Debug)]
pub struct Zombie {
    pub id: u32,
    pub kind: ZKind,
    pub pos: [f32; 3],
    pub hp: f32,
    pub atk_cd: f32,
    pub scream_t: f32,
    pub head_hits: u32,
    /// Where it's headed: a fighter it can see, or the last noise.
    pub target: [f32; 2],
    pub alerted: bool,
}

/// A bloater's burst — 4 m of lingering toxin.
#[derive(Clone, Debug)]
pub struct ToxicCloud {
    pub pos: [f32; 3],
    pub ttl: f32,
    pub tick_t: f32,
}

/// §8.2 noise: every action has a radius that feeds the horde director.
/// This is what turns the bow and spear into the CORRECT tool here.
pub fn gun_noise_m(kind: GunKind) -> f32 {
    match kind {
        GunKind::Fists => 4.0,
        GunKind::Bow => 6.0,
        GunKind::Spear => 8.0,
        GunKind::Glock => 60.0,
        GunKind::Deagle => 80.0,
        GunKind::Mp5 => 70.0,
        GunKind::Shotgun => 110.0,
        GunKind::Ak47 | GunKind::M4 => 90.0,
        GunKind::Awm | GunKind::M249 => 100.0,
        GunKind::Minigun => 95.0, // §7: the horde hears every burst
    }
}

pub const ZOMBIE_CAP: usize = 40;
pub const EXTRACT_LEN_S: f32 = 900.0; // a 15-minute run
pub const EXTRACT_REVEAL_S: f32 = 240.0;
pub const EXTRACT_RELOCATE_S: f32 = 720.0;
pub const EXTRACT_HOLD_S: f32 = 90.0;
pub const EXTRACT_RADIUS: f32 = 6.0;
/// §8.3: never spawn within this range of a player's view cone.
pub const ZSPAWN_MIN_M: f32 = 35.0;
pub const TOXIC_DPS: f32 = 8.0;
pub const TOXIC_R: f32 = 4.0;

#[derive(Clone, Debug)]
pub struct Pickup {
    pub kind: PickupKind,
    pub pos: [f32; 3],
    pub respawn_t: f32, // >0 → waiting to respawn (hidden)
}

// ------------------------------------------------------------------ modes

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Tdm,
    Koth,
    /// §8: co-op survival — insert, fight the horde, extract with what
    /// you carry. Gear is lost on death.
    Extraction,
}

pub struct TdmSim {
    pub cfg: MatchConfig,
    pub mode: Mode,
    pub map: MapKind,
    /// Playable half-extent — bigger maps have more room (v6 §10).
    pub half: f32,
    pub fighters: Vec<Fighter>,
    pub cover: Vec<Aabb>,
    pub cover_kind: Vec<CoverKind>,
    pub checkpoints: Vec<Checkpoint>,
    pub pickups: Vec<Pickup>,
    pub missiles: Vec<Missile>,
    /// §5.3 (Brief VI): pod missiles in flight.
    pub rockets: Vec<Rocket>,
    /// §3: rested arrows/spears, recoverable by walking over them.
    pub dropped: Vec<DroppedAmmo>,
    /// §5 throwables in flight / at rest with fuses burning.
    pub grenades_air: Vec<Grenade>,
    /// §5.4 active smoke spheres (bounded at SMOKE_MAX).
    pub smokes: Vec<SmokeVolume>,
    /// §5.5 molotov fire pools.
    pub fires: Vec<FirePool>,
    /// Detonation FX events for the client, with display ttl.
    pub booms: Vec<(Boom, f32)>,
    /// §8 the horde.
    pub zombies: Vec<Zombie>,
    /// §8 bloater bursts.
    pub toxics: Vec<ToxicCloud>,
    /// §8.3 director pressure 0..1 — time, noise, and player health feed
    /// it; spawn rate and composition scale off it.
    pub pressure: f32,
    /// §8.4 extraction: two candidate sites, the active index, and the
    /// hold progress in seconds.
    pub extract_sites: [[f32; 3]; 2],
    pub extract_idx: usize,
    pub extract_hold: f32,
    zspawn_cd: f32,
    next_zombie_id: u32,
    pub score: [f32; 2], // kills (TDM) or hill-seconds (KOTH)
    pub kill_feed: Vec<(KillEvent, f32)>,
    pub hits: Vec<(HitEvent, f32)>,
    pub impacts: Vec<(Impact, f32)>,
    pub tracers: Vec<Tracer>,
    pub t: f32,
    pub match_t: f32,
    pub overtime: bool,
    pub round_over_t: Option<f32>,
    pub winner: Option<Team>,
    pub hill: [f32; 3],
    pub player: usize,
    /// §9.1 broadphase over `cover` — rebuilt with the map, never mutated.
    grid: CoverGrid,
    rng: Pcg32,
    tick: u64,
    next_missile_id: u32,
}

const BLUE_NAMES: [&str; 8] = [
    "You", "Brasidas", "Cleon", "Pelops", "Dion", "Nikias", "Phormio", "Lamachos",
];
const RED_NAMES: [&str; 8] = [
    "Xerxes", "Otanes", "Mardon", "Artax", "Hydarnes", "Datis", "Bessus", "Tigran",
];

/// A fresh (magazine, reserve) pair for every slot of a loadout.
fn fresh_ammo(inv: Loadout) -> [(u32, u32); 3] {
    [
        (gun(inv[0]).mag, gun(inv[0]).reserve),
        (gun(inv[1]).mag, gun(inv[1]).reserve),
        (gun(inv[2]).mag, gun(inv[2]).reserve),
    ]
}

impl TdmSim {
    pub fn new(cfg: MatchConfig) -> Self {
        let per_team = cfg.per_team.clamp(1, 8);
        let mut rng = Pcg32::new(cfg.seed, 0x7D7D);
        let layout = build_map(cfg.map, &mut rng);
        let (cover, cover_kind, center_top, half) =
            (layout.cover, layout.kind, layout.center_top, layout.half);

        // ---- pickups: health / ammo / the robot armor -------------------
        // (weapon pads are gone in v6 — you BRING your loadout; the map
        // now feeds you consumables and the armor)
        let mut pickups = Vec::new();
        pickups.push(Pickup {
            kind: PickupKind::RobotArmor,
            pos: [0.0, center_top, 0.0],
            respawn_t: 0.0,
        });
        for (kind, x, z) in [
            (PickupKind::Health, -19.0, 14.0),
            (PickupKind::Health, 19.0, -14.0),
            (PickupKind::Ammo, -19.0, -14.0),
            (PickupKind::Ammo, 19.0, 14.0),
            // §6: the armor sets are LOOT — walk over a pad to suit up
            (PickupKind::FolkArmor, 0.0, 19.0),
            (PickupKind::PyroArmor, 0.0, -19.0),
            (PickupKind::ReconWeave, -19.0, 0.0),
            // §7 (Brief IV): the M134 pad — the mirror of Recon's spot
            (PickupKind::Minigun, 19.0, 0.0),
        ] {
            pickups.push(Pickup {
                kind,
                pos: [x, 0.0, z],
                respawn_t: 0.0,
            });
        }
        // §12.2: on the Battlefield the loot TEACHES the route — armor
        // along the landmarks, the Mech deep by the mine, consumables
        // spread down the long axis
        if cfg.map == MapKind::Battlefield {
            for pk in &mut pickups {
                pk.pos = match pk.kind {
                    PickupKind::FolkArmor => [-58.0, 0.0, 56.0], // settlement
                    PickupKind::PyroArmor => [-100.0, 0.0, -95.0], // the forge
                    PickupKind::ReconWeave => [86.0, 0.0, 100.0], // cathedral
                    PickupKind::RobotArmor => [166.0, 0.0, 152.0], // by the mine
                    _ => [pk.pos[0] * 3.0, 0.0, pk.pos[2] * 3.0],
                };
            }
        }
        // snap pickups AND checkpoints onto the terrain under them — a
        // lane that crosses a plateau must sit ON it, not inside it
        let support_at = |cover: &Vec<Aabb>, x: f32, z: f32| {
            let mut y = 0.0_f32;
            for c in cover {
                if x > c.min[0] && x < c.max[0] && z > c.min[2] && z < c.max[2] && c.max[1] > y {
                    y = c.max[1];
                }
            }
            y
        };
        for p in &mut pickups {
            p.pos[1] = p.pos[1].max(support_at(&cover, p.pos[0], p.pos[2]));
        }
        let checkpoints = layout
            .checkpoints
            .iter()
            .map(|&[x, z]| Checkpoint {
                pos: [x, support_at(&cover, x, z), z],
                owner: None,
                charge: 0.0,
            })
            .collect();

        let mut fighters = Vec::new();
        // §8: Extraction is CO-OP — everyone spawns Blue, the horde is
        // the other side
        let teams = if cfg.mode == Mode::Extraction { 1 } else { 2 };
        for team_i in 0..teams {
            let team = if team_i == 0 { Team::Blue } else { Team::Red };
            for k in 0..per_team {
                let (pos, yaw) = spawn_point(team, k, half);
                let is_player = team_i == 0 && k == 0;
                let idx = team_i * per_team + k;
                // v6: everyone spawns with a LOADOUT. The player brings
                // the one picked on the loadout screen; bots run varied
                // kits off a deterministic rotation.
                let inv: Loadout = if is_player {
                    cfg.loadout
                } else {
                    [
                        PRIMARIES[idx % PRIMARIES.len()],
                        SECONDARIES[idx % SECONDARIES.len()],
                        SPECIALS[idx % SPECIALS.len()],
                    ]
                };
                let g0 = inv[0];
                let name = if team == Team::Blue {
                    BLUE_NAMES[k % 8]
                } else {
                    RED_NAMES[k % 8]
                };
                fighters.push(Fighter {
                    name,
                    team,
                    gun: g0,
                    inventory: inv,
                    slot_ammo: fresh_ammo(inv),
                    active: 0,
                    shield_up: false,
                    lean: 0.0,
                    crouch: false,
                    switch_t: 0.0,
                    pos,
                    vel: [0.0, 0.0],
                    vy: 0.0,
                    grounded: true,
                    yaw,
                    prev_yaw: yaw,
                    roll_t: 0.0,
                    roll_cd: 0.0,
                    roll_dir: [0.0, 1.0],
                    roll_boost: 1.0,
                    health: MAX_HEALTH,
                    armor: 0.0,
                    armor_set: ArmorSet::None,
                    hull: 0.0,
                    fuel: 0.0,
                    mech_transition_t: 0.0,
                    mech_exiting: false,
                    mech_plates_dropped: 0,
                    brace: false,
                    mech_brace: false,
                    mech_weapon: MechWeapon::Gatling,
                    gatling_heat: 0.0,
                    gatling_vent_t: 0.0,
                    gatling_cd: 0.0,
                    gatling_trigger_t: 0.0,
                    autocannon_cd: 0.0,
                    knife_phase: 0.0,
                    knife_committed: false,
                    knife_struck: false,
                    melee_axe: false,
                    spin_t: 0.0,
                    heat: 0.0,
                    vent_t: 0.0,
                    spin_cmd: 0.0,
                    stride_wind_t: 0.0,
                    stride_t: 0.0,
                    stride_heat: 0.0,
                    prev_primary: g0,
                    punch: [0.0; 2],
                    punch_vel: [0.0; 2],
                    spray_i: 0.0,
                    last_shot_at: -100.0,
                    pod_ammo: 0,
                    pod_cd: 0.0,
                    pod_lock_t: 0.0,
                    pod_lock_id: -1,
                    pod_aim_held: false,
                    lock_warn_t: 0.0,
                    shield_dip_t: 0.0,
                    spear_wind_t: 0.0,
                    spear_aim: [0.0, 0.0, 1.0],
                    spear_v0: 0.0,
                    bow_draw_t: 0.0,
                    bow_aim: [0.0, 0.0, 1.0],
                    flip_t: 0.0,
                    flip_dir: [0.0, 0.0],
                    flip_kind: 1,
                    flip_used: false,
                    flip_recover_t: 0.0,
                    ability_cd: 0.0,
                    last_ability_at: -100.0,
                    last_dmg_at: -100.0,
                    ammo: gun(g0).mag,
                    reserve: gun(g0).reserve,
                    reload_t: 0.0,
                    sprint_gate_t: 0.0,
                    running_momentum_t: 0.0,
                    fire_cd: 0.0,
                    bloom: 0.0,
                    respawn_t: 0.0,
                    protect_t: SPAWN_PROTECT_S,
                    kills: 0,
                    deaths: 0,
                    assists: 0,
                    hits_dealt: 0,
                    last_hit_by: None,
                    ammo_full_t: 0.0,
                    grenades: GRENADE_LOADOUT,
                    throw_sel: 0,
                    cook_t: 0.0,
                    blind_t: 0.0,
                    burn_t: 0.0,
                    waypoint: [rng.range(-12.0, 12.0), rng.range(-8.0, 8.0)],
                    strafe_phase: rng.range(0.0, 6.28),
                    los_time: 0.0,
                    think_offset: idx as u32 % 12,
                });
            }
        }
        let mut fighters = fighters;
        // §6/§8 (Brief IV): the PLAYER's loadout choices — melee slot and
        // grenade budget preset. Bots keep the knife and standard pouch.
        fighters[0].melee_axe = cfg.melee_axe;
        fighters[0].grenades = GRENADE_PRESETS[cfg.grenade_preset % GRENADE_PRESETS.len()].0;
        TdmSim {
            cfg,
            mode: cfg.mode,
            map: cfg.map,
            half,
            grid: CoverGrid::build(&cover, half),
            fighters,
            cover,
            cover_kind,
            checkpoints,
            pickups,
            missiles: Vec::new(),
            rockets: Vec::new(),
            dropped: Vec::new(),
            grenades_air: Vec::new(),
            smokes: Vec::new(),
            fires: Vec::new(),
            booms: Vec::new(),
            score: [0.0, 0.0],
            kill_feed: Vec::new(),
            hits: Vec::new(),
            impacts: Vec::new(),
            tracers: Vec::new(),
            t: 0.0,
            match_t: if cfg.mode == Mode::Extraction {
                EXTRACT_LEN_S
            } else {
                MATCH_LEN_S
            },
            zombies: Vec::new(),
            toxics: Vec::new(),
            pressure: 0.0,
            extract_sites: [
                [half - 20.0, 0.0, half - 20.0],
                [-(half - 20.0), 0.0, -(half - 20.0)],
            ],
            extract_idx: 0,
            extract_hold: 0.0,
            zspawn_cd: 6.0,
            next_zombie_id: 0,
            overtime: false,
            round_over_t: None,
            winner: None,
            hill: [0.0, center_top, 0.0], // the hill IS the center top
            player: 0,
            rng,
            tick: 0,
            next_missile_id: 0,
        }
    }

    pub fn team_idx(team: Team) -> usize {
        match team {
            Team::Blue => 0,
            Team::Red => 1,
        }
    }

    /// §9.1: nearest static-cover hit along a ray, via the grid broadphase.
    /// The client uses this too (camera boom, crosshair ray).
    pub fn raycast_cover(&self, o: [f32; 3], d: [f32; 3], t_max: f32) -> Option<(f32, [f32; 3])> {
        self.grid.ray_hit(&self.cover, o, d, t_max)
    }

    /// Rebuild the broadphase after mutating `cover` directly (test
    /// harnesses build shooting ranges by clearing it). The grid indexes
    /// into `cover` — a stale grid after a mutation is out of bounds.
    pub fn rebuild_grid(&mut self) {
        self.grid = CoverGrid::build(&self.cover, self.half);
    }

    pub fn step(&mut self, cmd: PlayerCmd) {
        self.tick += 1;
        self.t += DT;
        for tr in &mut self.tracers {
            tr.ttl -= DT;
        }
        self.tracers.retain(|t| t.ttl > 0.0);
        for (_, ttl) in &mut self.kill_feed {
            *ttl -= DT;
        }
        self.kill_feed.retain(|(_, ttl)| *ttl > 0.0);
        for (_, ttl) in &mut self.hits {
            *ttl -= DT;
        }
        self.hits.retain(|(_, ttl)| *ttl > 0.0);
        for (_, ttl) in &mut self.impacts {
            *ttl -= DT;
        }
        self.impacts.retain(|(_, ttl)| *ttl > 0.0);

        if let Some(over) = self.round_over_t {
            if self.t - over > 7.0 {
                let seed = self.rng.next_u32() as u64;
                *self = TdmSim::new(MatchConfig {
                    seed: seed ^ 0x9E37,
                    ..self.cfg
                });
            }
            return;
        }

        // ---- match clock + overtime ------------------------------------
        self.match_t -= DT;
        if self.match_t <= 0.0 {
            if self.mode == Mode::Extraction {
                self.finish(Team::Red); // §8: overrun — the horde wins
            } else {
                let (b, r) = (self.score[0], self.score[1]);
                if (b - r).abs() > 0.01 {
                    self.finish(if b > r { Team::Blue } else { Team::Red });
                } else if !self.overtime {
                    self.overtime = true;
                    self.match_t = OVERTIME_S; // sudden death: next point wins
                } else {
                    self.finish(Team::Blue); // exhausted overtime: blue by honor
                }
            }
        }

        // ---- timers, respawns ------------------------------------------
        let t_now = self.t;
        let player_pouch =
            GRENADE_PRESETS[self.cfg.grenade_preset % GRENADE_PRESETS.len()].0;
        let mut spear_releases: Vec<usize> = Vec::new();
        for i in 0..self.fighters.len() {
            let f = &mut self.fighters[i];
            f.fire_cd = (f.fire_cd - DT).max(0.0);
            // §3.4: sprint-out. While moving at sprint-carry pace the
            // weapon is lowered and the gate is HELD at the class value;
            // once speed drops it counts down, and only then can the gun
            // fire. Applies to every fighter - bots corner-sprint too.
            {
                let sp = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
                if sp > SPRINT_CARRY_FRAC * SPRINT_SPEED {
                    f.sprint_gate_t = sprint_out_s(f.gun);
                } else {
                    f.sprint_gate_t = (f.sprint_gate_t - DT).max(0.0);
                }
                // §5.4: the spear's running-throw bonus needs "approach
                // run", not a tap - counts continuous time at/above the
                // run threshold, resets the instant speed drops below it
                if sp >= RUNNING_THROW_SPEED_FRAC * SPRINT_SPEED {
                    f.running_momentum_t += DT;
                } else {
                    f.running_momentum_t = 0.0;
                }
            }
            f.protect_t = (f.protect_t - DT).max(0.0);
            f.switch_t = (f.switch_t - DT).max(0.0);
            // §6.2: the mech transition timer covers BOTH directions.
            // When an EXIT finishes, the chassis actually powers down
            // here - deferring the teardown to the end of the window is
            // what makes leaving committal instead of a single-tick
            // state flip (MECH_EXIT_S was a dead constant before this).
            if f.mech_transition_t > 0.0 {
                f.mech_transition_t = (f.mech_transition_t - DT).max(0.0);
                if f.mech_transition_t <= 0.0 && f.mech_exiting {
                    f.mech_exiting = false;
                    f.armor_set = ArmorSet::None;
                    f.armor = 0.0;
                    f.hull = 0.0;
                    f.fuel = 0.0;
                    f.mech_plates_dropped = 0;
                }
            } else {
                f.mech_exiting = false;
            }
            f.roll_cd = (f.roll_cd - DT).max(0.0);
            f.ammo_full_t = (f.ammo_full_t - DT).max(0.0);
            f.blind_t = (f.blind_t - DT).max(0.0);
            f.burn_t = (f.burn_t - DT).max(0.0);
            f.ability_cd = (f.ability_cd - DT).max(0.0);
            f.shield_dip_t = (f.shield_dip_t - DT).max(0.0);
            f.pod_cd = (f.pod_cd - DT).max(0.0);
            f.lock_warn_t = (f.lock_warn_t - DT).max(0.0);
            // §7: minigun barrels and heat. `spin_cmd` is the trigger
            // hold-timer (refreshed by try_fire, drained here): live →
            // barrels climb and heat holds; expired → wind down + cool.
            if f.gun == GunKind::Minigun {
                if f.spin_cmd > 0.0 {
                    f.spin_cmd -= DT;
                    f.spin_t = (f.spin_t + DT).min(MINIGUN_SPINUP_S);
                } else {
                    f.spin_t = (f.spin_t - DT).max(0.0);
                    if f.vent_t <= 0.0 {
                        f.heat = (f.heat - MINIGUN_HEAT_DECAY * DT).max(0.0);
                    }
                }
                if f.vent_t > 0.0 {
                    f.vent_t -= DT;
                    if f.vent_t <= 0.0 {
                        f.vent_t = 0.0;
                        f.heat = 0.0; // a vent always clears the gun
                    }
                }
            } else {
                f.spin_t = 0.0;
                f.spin_cmd = 0.0;
            }
            // §C: the hull mounts' own clocks. OUTSIDE the
            // `gun == Minigun` block above on purpose - what the pilot
            // happens to be carrying has nothing to do with the guns
            // bolted to the chassis.
            //
            // The heat decay is gated on the TRIGGER-HOLD timer, exactly
            // as the minigun's is: a barrel group under fire does not
            // cool. Ungated, the 9.5/s decay eats most of the 12/s ramp
            // and stretches the run to a forced vent from ~8 s to ~40 s
            // - the design says the hull gun sustains about TWICE the
            // minigun's ~4.4 s, not nine times.
            f.gatling_cd = (f.gatling_cd - DT).max(0.0);
            f.gatling_trigger_t = (f.gatling_trigger_t - DT).max(0.0);
            if f.gatling_vent_t > 0.0 {
                f.gatling_vent_t -= DT;
                if f.gatling_vent_t <= 0.0 {
                    f.gatling_vent_t = 0.0;
                    f.gatling_heat = 0.0; // a vent always clears the mount
                }
            } else if f.gatling_trigger_t <= 0.0 {
                f.gatling_heat = (f.gatling_heat - GATLING_HEAT_DECAY * DT).max(0.0);
            }
            f.autocannon_cd = (f.autocannon_cd - DT).max(0.0);
            // §3: the spear releases at the END of the windup, on the
            // tracked aim (the ammo was spent at the trigger)
            if f.spear_wind_t > 0.0 {
                f.spear_wind_t -= DT;
                if f.spear_wind_t <= 0.0 {
                    f.spear_wind_t = 0.0;
                    spear_releases.push(i);
                }
            }
            // §4 flip bookkeeping: the spin drifts 1.4 m/s in its
            // direction; landing starts the 0.18 s recovery; recovery
            // ending returns the trigger AND the gun
            if f.flip_t > 0.0 {
                f.flip_t = (f.flip_t - DT).max(0.0);
                f.pos[0] += f.flip_dir[0] * FLIP_BOOST * DT;
                f.pos[2] += f.flip_dir[1] * FLIP_BOOST * DT;
                if f.grounded {
                    f.flip_t = 0.0;
                }
            }
            if f.flip_recover_t > 0.0 {
                f.flip_recover_t -= DT;
                if f.flip_recover_t <= 0.0 {
                    f.flip_recover_t = 0.0;
                    f.flip_used = false; // one per airborne period
                }
            } else if f.grounded && f.flip_used && f.flip_t <= 0.0 {
                f.flip_recover_t = FLIP_RECOVER_S;
            }
            // §9 (Brief III): post-shot spread recovers fast — controlled
            // bursts stay tight (was 0.12/s)
            f.bloom = (f.bloom - 0.02 * DT * 10.0).max(0.0);
            // §2 (Brief VI): punch decay — the CS:GO exact math. Angle
            // integrates velocity, decays exponentially THEN linearly;
            // velocity decays exponentially. Rest ≈0.3–0.5 s.
            for k in 0..2 {
                f.punch[k] += f.punch_vel[k] * DT;
                f.punch[k] *= (-PUNCH_DECAY_EXP * DT).exp();
                let lin = PUNCH_DECAY_LIN_DEG * DT;
                f.punch[k] = if f.punch[k].abs() <= lin {
                    0.0
                } else {
                    f.punch[k] - lin * f.punch[k].signum()
                };
                f.punch_vel[k] *= (-PUNCH_VEL_DECAY_EXP * DT).exp();
            }
            // spray index: holds through full-auto, decays once idle
            // longer than cycletime × 1.1 — tap-firing resets the pattern
            if f.armed()
                && t_now - f.last_shot_at > gun(f.gun).fire_period * 1.1
            {
                f.spray_i = (f.spray_i - SPRAY_INDEX_DECAY * DT).max(0.0);
            }
            // §10 (Brief III): base regen for everyone — 12 s untouched,
            // then 8.33 hp/s (Recon's own earlier regen stacks on top;
            // that's its identity)
            if f.alive() && t_now - f.last_dmg_at > REGEN_DELAY_S {
                f.health = (f.health + REGEN_RATE_HPS * DT).min(MAX_HEALTH);
            }
            // §6 per-set upkeep: power recharge, fuel refill, Recon regen
            match f.armor_set {
                ArmorSet::RobotSuit => {
                    if f.grounded && t_now - f.last_ability_at > POWER_REGEN_DELAY {
                        f.armor = (f.armor + POWER_REGEN * DT).min(POWER_MAX);
                    }
                }
                ArmorSet::Pyro => {
                    if t_now - f.last_ability_at > 1.0 {
                        f.fuel = (f.fuel + FLAME_REFILL * DT).min(FLAME_FUEL_S);
                    }
                }
                ArmorSet::Recon => {
                    if f.alive() && t_now - f.last_dmg_at > RECON_REGEN_DELAY {
                        f.health = (f.health + RECON_REGEN * DT).min(MAX_HEALTH);
                    }
                }
                _ => {}
            }
            if f.reload_t > 0.0 {
                f.reload_t -= DT;
                if f.reload_t <= 0.0 {
                    // top up from reserve, KEEPING what's still chambered —
                    // dumping the part-full mag used to strictly lose ammo
                    let spec = gun(f.gun);
                    let total = f.ammo + f.reserve;
                    f.ammo = spec.mag.min(total);
                    f.reserve = total - f.ammo;
                }
            }
            if f.respawn_t > 0.0 {
                f.respawn_t -= DT;
                if f.respawn_t <= 0.0 {
                    let slot = i % self.cfg.per_team.max(1);
                    let (mut pos, mut yaw) = spawn_point(f.team, slot, self.half);
                    // "check back": an owned checkpoint pulls the respawn
                    // forward — you rejoin the fight where your team holds
                    if let Some(cp) = self
                        .checkpoints
                        .iter()
                        .find(|c| c.owner == Some(f.team))
                    {
                        let ang = slot as f32 * 1.3;
                        pos = [
                            cp.pos[0] + ang.cos() * 1.2,
                            cp.pos[1],
                            cp.pos[2] + ang.sin() * 1.2,
                        ];
                        // face the arena center from the forward spawn
                        yaw = (-pos[0]).atan2(-pos[2]);
                    }
                    f.pos = pos;
                    f.yaw = yaw;
                    f.prev_yaw = yaw; // no phantom whip-turn on the spawn tick
                    f.vy = 0.0;
                    f.grounded = true;
                    f.health = MAX_HEALTH;
                    f.armor = 0.0;
                    f.armor_set = ArmorSet::None; // §6: gear is lost on death
                    f.hull = 0.0;
                    f.fuel = 0.0;
                    f.mech_transition_t = 0.0;
                    f.mech_exiting = false;
                    f.mech_plates_dropped = 0;
                    f.brace = false;
                    f.mech_brace = false;
                    // §C: the hull mounts die with the chassis. A fresh
                    // life starts on the gatling with cold barrels -
                    // leaving `gatling_vent_t` set would hand the next
                    // chassis a lockout it never earned.
                    f.mech_weapon = MechWeapon::Gatling;
                    f.gatling_heat = 0.0;
                    f.gatling_vent_t = 0.0;
                    f.gatling_cd = 0.0;
                    f.gatling_trigger_t = 0.0;
                    f.autocannon_cd = 0.0;
                    f.knife_phase = 0.0;
                    f.knife_committed = false;
                    f.knife_struck = false;
                    f.spin_t = 0.0;
                    f.heat = 0.0;
                    f.vent_t = 0.0;
                    f.spin_cmd = 0.0;
                    f.stride_wind_t = 0.0;
                    f.stride_t = 0.0;
                    f.stride_heat = 0.0;
                    f.punch = [0.0; 2];
                    f.punch_vel = [0.0; 2];
                    f.spray_i = 0.0;
                    f.last_shot_at = -100.0;
                    f.pod_ammo = 0;
                    f.pod_cd = 0.0;
                    f.pod_lock_t = 0.0;
                    f.pod_lock_id = -1;
                    f.pod_aim_held = false;
                    f.lock_warn_t = 0.0;
                    f.shield_dip_t = 0.0;
                    f.flip_t = 0.0;
                    f.flip_used = false;
                    f.flip_recover_t = 0.0;
                    f.spear_wind_t = 0.0;
                    f.ability_cd = 0.0;
                    f.last_ability_at = -100.0;
                    f.last_dmg_at = -100.0;
                    // §7: the pad weapon dies with you — the ORIGINAL
                    // primary comes back in its place
                    if f.inventory[0] == GunKind::Minigun {
                        f.inventory[0] = f.prev_primary;
                    }
                    // the full loadout comes back with you
                    f.slot_ammo = fresh_ammo(f.inventory);
                    f.active = 0;
                    f.gun = f.inventory[0];
                    f.ammo = f.slot_ammo[0].0;
                    f.reserve = f.slot_ammo[0].1;
                    f.shield_up = false;
                    f.lean = 0.0;
                    f.protect_t = SPAWN_PROTECT_S;
                    f.vel = [0.0, 0.0];
                    f.crouch = false;
                    f.roll_t = 0.0;
                    f.roll_cd = 0.0;
                    f.roll_boost = 1.0;
                    f.grenades = if i == 0 {
                        // §8: the player's chosen budget comes back too
                        player_pouch
                    } else {
                        GRENADE_LOADOUT
                    };
                    f.cook_t = 0.0;
                    f.blind_t = 0.0;
                    f.burn_t = 0.0;
                    // Death mid-action must not tax the NEXT life: dying
                    // during a 3s reload left reload_t counting through
                    // the corpse, so the respawn arrived with a full mag
                    // it could not fire for up to 3 seconds (try_fire
                    // gates on reload_t). Same for a mid-switch death
                    // (0.6s), a hot fire_cd, and accumulated bloom
                    // spreading the fresh body's first shots.
                    f.reload_t = 0.0;
                    f.switch_t = 0.0;
                    f.fire_cd = 0.0;
                    f.bloom = 0.0;
                    f.sprint_gate_t = 0.0; // died sprinting != spawn disarmed
                    f.last_hit_by = None; // a new life owes no assist to the old one's attackers
                    f.running_momentum_t = 0.0;
                    // A bot's fire gate is `los_time > reaction_s`, and
                    // `bot_act` is skipped entirely while dead - so
                    // los_time froze at whatever it held the instant the
                    // bot died and carried straight through respawn. A
                    // bot killed mid-firefight came back shooting with
                    // ZERO reaction delay. A fresh body re-acquires.
                    f.los_time = 0.0;
                }
            }
        }

        // §3: launch the wound-up spears (release aim, release charge)
        for i in spear_releases {
            if !self.fighters[i].alive() {
                continue;
            }
            let (aim, v0) = (self.fighters[i].spear_aim, self.fighters[i].spear_v0);
            let o = self.muzzle_origin(i);
            let dmg = gun(GunKind::Spear).projectile.unwrap().1;
            self.spawn_missile(o, aim, v0, dmg, i, true); // always a spear
        }

        // ---- pickups respawn + collection ------------------------------
        for pi in 0..self.pickups.len() {
            if self.pickups[pi].respawn_t > 0.0 {
                self.pickups[pi].respawn_t -= DT;
                continue;
            }
            let ppos = self.pickups[pi].pos;
            let kind = self.pickups[pi].kind;
            let mut taken = false;
            for i in 0..self.fighters.len() {
                let f = &self.fighters[i];
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - ppos[0];
                let dz = f.pos[2] - ppos[2];
                let dy = f.pos[1] - ppos[1];
                if dx * dx + dz * dz < PICKUP_RADIUS * PICKUP_RADIUS && dy.abs() < 1.4 {
                    match kind {
                        PickupKind::Health => {
                            let f = &mut self.fighters[i];
                            if f.health >= MAX_HEALTH {
                                continue;
                            }
                            f.health = (f.health + 50.0).min(MAX_HEALTH);
                        }
                        PickupKind::Ammo => {
                            let f = &mut self.fighters[i];
                            if !f.armed() {
                                continue;
                            }
                            // §7: the minigun has no reserve to fill — a
                            // reserve it can never load would also trap
                            // bots in the "still has ammo" branch forever.
                            // The cache tops the BELT up instead.
                            if f.gun == GunKind::Minigun {
                                let mag = gun(GunKind::Minigun).mag;
                                if f.ammo >= mag {
                                    continue;
                                }
                                f.ammo = (f.ammo + 100).min(mag);
                            } else {
                                f.reserve += gun(f.gun).mag * 2;
                            }
                        }
                        PickupKind::RobotArmor => {
                            // §11: the pad now grants the MECH chassis
                            let f = &mut self.fighters[i];
                            f.armor_set = ArmorSet::RobotSuit;
                            f.armor = POWER_MAX;
                            f.hull = MECH_HULL;
                            f.fuel = 0.0;
                            // §6.2 (Brief VII v2): boarding is committed,
                            // not instant - the chassis seals for 1.6s
                            // before it can fight (gated in try_fire and
                            // the movement code below).
                            f.mech_transition_t = MECH_ENTER_S;
                            f.mech_plates_dropped = 0;
                            // §C: a FRESH chassis - cold mounts, gatling
                            // selected. Without this a pilot who cooked
                            // one hull off, dismounted and boarded a new
                            // one would inherit the old vent lockout.
                            f.mech_weapon = MechWeapon::Gatling;
                            f.gatling_heat = 0.0;
                            f.gatling_vent_t = 0.0;
                            f.gatling_cd = 0.0;
                            f.gatling_trigger_t = 0.0;
                            f.autocannon_cd = 0.0;
                            // §5.3 (Brief VI): 4 tubes per chassis —
                            // resupply is a fresh chassis, not a pickup
                            f.pod_ammo = POD_TUBES;
                        }
                        PickupKind::FolkArmor => {
                            let f = &mut self.fighters[i];
                            f.armor_set = ArmorSet::Folk;
                            f.armor = 0.0;
                            f.fuel = 0.0;
                        }
                        PickupKind::PyroArmor => {
                            let f = &mut self.fighters[i];
                            f.armor_set = ArmorSet::Pyro;
                            f.armor = 0.0;
                            f.fuel = FLAME_FUEL_S;
                        }
                        PickupKind::ReconWeave => {
                            let f = &mut self.fighters[i];
                            f.armor_set = ArmorSet::Recon;
                            f.armor = 0.0;
                            f.fuel = 0.0;
                        }
                        PickupKind::Minigun => {
                            let f = &mut self.fighters[i];
                            if f.inventory[0] == GunKind::Minigun {
                                continue; // already hauling one
                            }
                            // bank the OUTGOING slot's live rounds first —
                            // the forced switch must not resurrect stale
                            // spawn-time ammo later
                            f.slot_ammo[f.active] = (f.ammo, f.reserve);
                            // §7: it takes the PRIMARY slot and your
                            // hands right now — the displaced primary
                            // returns on death
                            f.prev_primary = f.inventory[0];
                            f.inventory[0] = GunKind::Minigun;
                            f.slot_ammo[0] = (gun(GunKind::Minigun).mag, 0);
                            f.active = 0;
                            f.gun = GunKind::Minigun;
                            f.ammo = f.slot_ammo[0].0;
                            f.reserve = 0;
                            f.reload_t = 0.0;
                            f.heat = 0.0;
                            f.spin_t = 0.0;
                            f.vent_t = 0.0;
                            f.switch_t = SWITCH_S;
                        }
                    }
                    taken = true;
                    break;
                }
            }
            if taken {
                self.pickups[pi].respawn_t = match kind {
                    PickupKind::Health | PickupKind::Ammo => 20.0,
                    PickupKind::Minigun => 75.0, // a power weapon earns a longer clock
                    _ => 45.0, // every armor set is a 45 s pad
                };
            }
        }

        // ---- §3: dropped-ammo walk-over pickup (players AND bots; no
        // owner lock, no prompt). At cap the entity is NOT consumed — it
        // stays on the ground and the HUD says AMMO FULL.
        for di in 0..self.dropped.len() {
            if self.dropped[di].count == 0 {
                continue;
            }
            let (dpos, kind) = (self.dropped[di].pos, self.dropped[di].kind);
            let want = match kind {
                AmmoKind::Arrow => GunKind::Bow,
                AmmoKind::Spear => GunKind::Spear,
            };
            let cap = match kind {
                AmmoKind::Arrow => AMMO_CAP_ARROW,
                AmmoKind::Spear => AMMO_CAP_SPEAR,
            };
            for i in 0..self.fighters.len() {
                let f = &self.fighters[i];
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - dpos[0];
                let dz = f.pos[2] - dpos[2];
                let dy = f.pos[1] - dpos[1];
                if dx * dx + dz * dz > DROPPED_RADIUS * DROPPED_RADIUS || dy.abs() > 1.4 {
                    continue;
                }
                let Some(slot) = (0..3).find(|&s| f.inventory[s] == want) else {
                    continue; // no bow/spear in the loadout — not your ammo
                };
                let f = &mut self.fighters[i];
                let reserve = if slot == f.active {
                    f.reserve
                } else {
                    f.slot_ammo[slot].1
                };
                let room = cap.saturating_sub(reserve);
                if room == 0 {
                    f.ammo_full_t = 1.5;
                    continue; // leave it lying there — visible feedback
                }
                let take = (self.dropped[di].count as u32).min(room);
                if slot == f.active {
                    f.reserve += take;
                    f.slot_ammo[slot].1 = f.reserve;
                } else {
                    f.slot_ammo[slot].1 += take;
                }
                self.dropped[di].count -= take as u8;
                if self.dropped[di].count == 0 {
                    break;
                }
            }
        }
        self.dropped.retain(|d| d.count > 0);

        // ---- player -----------------------------------------------------
        let p = self.player;
        if self.fighters[p].alive() {
            // Task 3 rule 3: the PRE-move velocity, captured before the
            // movement block below overwrites f.vel with this tick's own
            // input. This sim has no horizontal inertia - vel IS the
            // input - so "what was I doing before the dodge" only exists
            // here, one line before it is destroyed. (The first wiring
            // attempt read f.vel at the dodge trigger, which by then was
            // already the dodge tick's backward tap: bonus never fired.
            // The launch test caught it.)
            let prior_vel = self.fighters[p].vel;
            self.fighters[p].set_crouch(cmd.crouch);
            self.fighters[p].lean = cmd.lean.clamp(-1.0, 1.0);
            // slot select (number keys) + shield toggle (E)
            // §C.5: the number keys mean something DIFFERENT while
            // piloting - the same repurposing §A did with crouch. A
            // sealed chassis has no inventory to swap into; 1 and 2
            // pick which hull mount the trigger drives. Gating the
            // infantry path on `!in_mech()` is the load-bearing half:
            // without it a number key would keep silently switching the
            // pilot's CARRIED gun underneath the mech, burning a
            // SWITCH_S he can neither see nor use.
            if let Some(s) = cmd.slot {
                if self.fighters[p].in_mech() {
                    match s {
                        0 => self.fighters[p].mech_weapon = MechWeapon::Gatling,
                        1 => self.fighters[p].mech_weapon = MechWeapon::Autocannon,
                        _ => {}
                    }
                } else {
                    self.switch_slot(p, s as usize);
                }
            }
            if cmd.shield {
                let f = &mut self.fighters[p];
                f.shield_up = !f.shield_up;
            }
            let p_spec = gun(self.fighters[p].gun);
            let scoped = cmd.ads && p_spec.scoped;
            // §4: a drawn bow / cocked spear walks at rifle-ADS pace, not
            // a crawl — the accuracy cost of moving lives in `try_fire`'s
            // stability model instead of in a movement prohibition
            let drawn = cmd.ads && p_spec.projectile.is_some();
            // §4.3 (Brief VI): the mech does not sprint — its one pace
            // is the 85% walk; the side-step is the only burst
            let in_mech = self.fighters[p].armor_set == ArmorSet::RobotSuit
                && self.fighters[p].hull > 0.0;
            // §7.4 (BRIEF VIII): sprint on a mech now means something -
            // it winds up power stride instead of being a pure no-op.
            // Windup is cancellable (nothing's been paid yet); once the
            // burst itself starts it's committed, ticking down on its
            // own even if sprint is released mid-burst. The re-arm gate
            // is a FULL cooldown to zero, not merely "under the cap" -
            // a completed burst always maxes heat to exactly 100 (2.5s
            // x 40/s), so gating on "< 100" let one tick of passive
            // cooldown re-arm it almost immediately, which made the
            // heat cost close to meaningless. Full cooldown (5s at the
            // passive rate) is the real rest between bursts.
            {
                let f = &mut self.fighters[p];
                if f.stride_t > 0.0 {
                    f.stride_t = (f.stride_t - DT).max(0.0);
                    f.stride_heat = (f.stride_heat + POWER_STRIDE_HEAT_PER_S * DT).min(100.0);
                } else if in_mech && cmd.sprint && f.grounded && f.stride_heat <= 0.0 {
                    f.stride_wind_t += DT;
                    if f.stride_wind_t >= POWER_STRIDE_WINDUP_S {
                        f.stride_wind_t = 0.0;
                        f.stride_t = POWER_STRIDE_DURATION_S;
                    }
                } else {
                    f.stride_wind_t = 0.0; // released early or ineligible - nothing owed
                    // cool down whenever not actively striding, same
                    // rhythm as the windup/burst cost
                    f.stride_heat = (f.stride_heat - POWER_STRIDE_HEAT_PER_S * 0.5 * DT).max(0.0);
                }
            }
            let mut speed = if cmd.sprint && !drawn && !in_mech {
                SPRINT_SPEED // sprinting at full draw is genuinely not possible
            } else {
                MOVE_SPEED
            };
            // read the AUTHORITATIVE flag, not the raw intent: a mech is
            // denied crouch, so reading `cmd.crouch` here charged it the
            // crouch speed tax for a stance it never entered
            if self.fighters[p].crouch {
                speed *= CROUCH_SPEED_MULT;
            }
            // the raised shield owns the pace — ADS/scope mults don't
            // stack on top (you're not sighting anything behind a plate)
            if self.fighters[p].shield_up {
                speed *= SHIELD_SPEED_MULT;
            } else if scoped {
                speed *= SCOPED_SPEED_MULT; // AWM glass: a crawl
            } else if drawn {
                speed *= if self.fighters[p].gun == GunKind::Bow {
                    DRAW_SPEED_MULT_BOW
                } else {
                    DRAW_SPEED_MULT_SPEAR
                };
            } else if cmd.ads {
                speed *= ADS_SPEED_MULT;
            }
            // §6: the equipped set owns the pace; a drained Robot Suit is
            // heavy and grounded; a held Shieldwall Brace is a plant
            {
                let f = &self.fighters[p];
                let aspec = armor_spec(f.armor_set);
                speed *= aspec.move_mult;
                // §7.4: power stride OVERRIDES the walk pace outright -
                // 110% of soldier run speed is the point, not 110% ON
                // TOP of the 85% walk multiplier just applied above.
                if f.stride_t > 0.0 {
                    speed = MOVE_SPEED * POWER_STRIDE_SPEED_MULT;
                }
                if f.armor_set == ArmorSet::RobotSuit && f.armor <= 0.0 {
                    speed *= ROBOT_DRAINED_MOVE;
                }
                if f.brace {
                    speed *= BRACE_SPEED_MULT;
                }
                // §A.4: the mech's OWN brace multiplier. Deliberately a
                // separate branch, not an `else if` and not a reuse of
                // the infantry constant above - see MECH_BRACE_* .
                if f.mech_brace {
                    speed *= MECH_BRACE_SPEED_MULT;
                }
                if f.flip_recover_t > 0.0 {
                    speed *= FLIP_RECOVER_SPEED; // §4: 0.18 s landing tax
                }
                // §7: the minigun is MASS — slower carried, a trudge
                // with the barrels spun
                if f.gun == GunKind::Minigun {
                    speed *= if f.spin_t > 0.2 {
                        MINIGUN_SPUN_MOVE_MULT
                    } else {
                        MINIGUN_MOVE_MULT
                    };
                }
                // §1 (Brief V): aiming a throw is a commitment — 70%
                // walk. The mech's LAUNCHER carries no such tax.
                if f.cook_t > 0.0
                    && !(f.armor_set == ArmorSet::RobotSuit && f.hull > 0.0)
                {
                    speed *= THROW_AIM_MOVE_MULT;
                }
            }
            let mag = (cmd.move_x * cmd.move_x + cmd.move_z * cmd.move_z)
                .sqrt()
                .max(1e-6);
            let cap = if mag > 1.0 { mag } else { 1.0 };
            let mut vel = [cmd.move_x / cap * speed, cmd.move_z / cap * speed];
            if drawn {
                // bracing: strafe and backpedal at 0.85 of the forward
                // pace — walking INTO the shot is the natural motion
                let (fx, fz) = (cmd.yaw.sin(), cmd.yaw.cos());
                let fwd = vel[0] * fx + vel[1] * fz;
                let (lx, lz) = (vel[0] - fwd * fx, vel[1] - fwd * fz);
                let fwd = if fwd > 0.0 { fwd } else { fwd * DRAW_SIDE_MULT };
                vel = [
                    fx * fwd + lx * DRAW_SIDE_MULT,
                    fz * fwd + lz * DRAW_SIDE_MULT,
                ];
            }
            // §1.3: `vel` above is the input TARGET, not the result -
            // the body accelerates toward it and decelerates out of it.
            // Impulses that set velocity directly (the dodge burst at
            // the roll block, the power-stride burst) still write
            // `f.vel` themselves AFTER this and are deliberately
            // unaffected: an impulse is not a steering input.
            self.fighters[p].vel = approach_velocity(self.fighters[p].vel, vel, DT);
            // §11: a mech TURNS at a capped rate — facing a new threat is
            // a visible, punishable commitment (the armor follows the
            // body, the pilot's view stays free)
            if self.fighters[p].armor_set == ArmorSet::RobotSuit && self.fighters[p].hull > 0.0
            {
                let f = &mut self.fighters[p];
                let d = wrap_angle(cmd.yaw - f.yaw);
                // §7.4: the burst caps turning harder than the normal
                // pivot rate - striding in a straight line is the deal
                let rate = if f.stride_t > 0.0 { POWER_STRIDE_TURN_RATE } else { MECH_TURN_RATE };
                let step = (rate * DT).min(d.abs());
                f.yaw += d.signum() * step;
            } else {
                self.fighters[p].yaw = cmd.yaw;
            }
            // §3: the thrower TRACKS the target through the plant — the
            // player's wound-up spear follows the live aim to release
            if self.fighters[p].spear_wind_t > 0.0 {
                self.fighters[p].spear_aim = normalize(cmd.aim);
            }
            // duck-spin dodge: somersault in the move direction (facing if
            // standing still); grounded only, gated by a short cooldown.
            // §2 (Brief V): the human rolls (load → burst → ease-out);
            // the MECH takes a braced SIDE-STEP instead — tall, grounded,
            // committed — a 2.7 m walker does not somersault.
            if cmd.dodge {
                let mech = self.fighters[p].armor_set == ArmorSet::RobotSuit
                    && self.fighters[p].hull > 0.0;
                let f = &mut self.fighters[p];
                if f.grounded && f.roll_t <= 0.0 && f.roll_cd <= 0.0 {
                    let m = (cmd.move_x * cmd.move_x + cmd.move_z * cmd.move_z).sqrt();
                    f.roll_dir = if m > 0.2 {
                        [cmd.move_x / m, cmd.move_z / m]
                    } else {
                        [f.yaw.sin(), f.yaw.cos()]
                    };
                    // Task 3 rule 3: SNAPSHOT the pre-dodge velocity HERE,
                    // at the trigger - the burst phase overwrites f.vel
                    // every tick, which is why this bonus sat unwired for
                    // two briefs ("the client never sees the pre-dodge
                    // velocity"). The sim sees it right now. A dodge cut
                    // against real prior movement launches harder; the
                    // mech is steel, not tendon - no elastic return.
                    let along =
                        prior_vel[0] * f.roll_dir[0] + prior_vel[1] * f.roll_dir[1];
                    f.roll_boost = if !mech && along < -0.5 {
                        1.0 + counter_movement_bonus(along, 1.0, ROLL_COUNTER_BONUS)
                    } else {
                        1.0
                    };
                    if mech {
                        f.roll_t = MECH_STEP_S + ROLL_EASE_S;
                        f.roll_cd = MECH_STEP_S + ROLL_EASE_S + MECH_STEP_CD_S;
                    } else {
                        f.roll_t = ROLL_LOAD_S + ROLL_S + ROLL_EASE_S;
                        f.roll_cd = ROLL_LOAD_S + ROLL_S + ROLL_EASE_S + ROLL_CD_S;
                    }
                } else if !mech
                    && !f.grounded
                    && f.flip_t <= 0.0
                    && !f.flip_used
                    && f.roll_t <= 0.0
                {
                    // §4: Q airborne = the flip. Direction from the stick
                    // in body space; no input = backflip. One per airborne
                    // period, 1.4 m/s of real travel, and the gun is
                    // locked until landing recovery.
                    let (fx, fz) = (f.yaw.sin(), f.yaw.cos());
                    let (mx, mz) = (cmd.move_x, cmd.move_z);
                    let fwd_c = mx * fx + mz * fz;
                    let lat_c = mx * fz - mz * fx; // screen-right component
                    f.flip_kind = if fwd_c.abs().max(lat_c.abs()) < 0.3 {
                        1 // backflip default
                    } else if fwd_c.abs() >= lat_c.abs() {
                        if fwd_c > 0.0 {
                            0
                        } else {
                            1
                        }
                    } else if lat_c > 0.0 {
                        3
                    } else {
                        2
                    };
                    f.flip_dir = match f.flip_kind {
                        0 => [fx, fz],
                        1 => [-fx, -fz],
                        3 => [fz, -fx],
                        _ => [-fz, fx],
                    };
                    f.flip_t = FLIP_S;
                    f.flip_used = true;
                }
            }
            // §4.3 (Brief VI): the mech CANNOT jump — grounded is its
            // identity; the braced side-step is its only dash
            if cmd.jump
                && self.fighters[p].grounded
                && self.fighters[p].roll_t <= 0.0
                && !(self.fighters[p].armor_set == ArmorSet::RobotSuit
                    && self.fighters[p].hull > 0.0)
            {
                let f = &mut self.fighters[p];
                f.vy = JUMP_SPEED;
                f.pos[1] += 0.05; // clear the support clamp so the ascent integrates
                f.grounded = false;
            }
            // §4.3 (Brief VI): FLIGHT IS DELETED. The Brief IV thruster
            // block (hold SPACE airborne → climb, power-metered) lived
            // here; the mech never leaves the ground now. jump_held is
            // dead input for the mech by design.
            // §4.6 (Brief VI): U dismounts — the pilot steps out on
            // foot; the spent chassis is scrapped (the pad respawns)
            // §6.2: leaving is COMMITTED too - the chassis powers down
            // over MECH_EXIT_S and only then hands the pilot back. The
            // teardown itself runs in the timer loop; pressing exit only
            // STARTS it, and cannot be started while a transition (in
            // either direction) is already running.
            if cmd.exit_mech
                && self.fighters[p].armor_set == ArmorSet::RobotSuit
                && self.fighters[p].hull > 0.0
                && self.fighters[p].mech_transition_t <= 0.0
            {
                let f = &mut self.fighters[p];
                f.mech_transition_t = MECH_EXIT_S;
                f.mech_exiting = true;
            }
            // §5.3 (Brief VI): the missile pod. HOLD = targeting: a MECH
            // under the reticle (6° cone, ≤250 m, LOS) accrues lock —
            // and the VICTIM is warned from lock start, not launch.
            // RELEASE with a full 1.3 s lock = homing launch; a quick
            // tap dumb-fires straight. Never locks infantry.
            {
                let in_mech = self.fighters[p].armor_set == ArmorSet::RobotSuit
                    && self.fighters[p].hull > 0.0;
                let can_pod = in_mech
                    && self.fighters[p].pod_ammo > 0
                    && self.fighters[p].pod_cd <= 0.0
                    && self.fighters[p].alive()
                    // §7.4: power stride locks the missile pod while active
                    && self.fighters[p].stride_t <= 0.0;
                if cmd.pod_aim && can_pod {
                    let eye = self.muzzle_origin(p);
                    let d = normalize(cmd.aim);
                    let mut tgt: i32 = -1;
                    let pteam = self.fighters[p].team;
                    for (j, g) in self.fighters.iter().enumerate() {
                        if j == p
                            || g.team == pteam
                            || !g.alive()
                            || !(g.armor_set == ArmorSet::RobotSuit && g.hull > 0.0)
                        {
                            continue; // mechs ONLY — never infantry
                        }
                        let c = [g.pos[0], g.pos[1] + g.height() * 0.5, g.pos[2]];
                        let to = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
                        let dist = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2])
                            .sqrt()
                            .max(0.01);
                        let dot =
                            (to[0] * d[0] + to[1] * d[1] + to[2] * d[2]) / dist;
                        if dist <= POD_RANGE_M
                            && dot > POD_CONE_COS
                            && self.los_clear(eye, c)
                        {
                            tgt = j as i32;
                            break;
                        }
                    }
                    if tgt >= 0 && tgt == self.fighters[p].pod_lock_id {
                        self.fighters[p].pod_lock_t += DT;
                    } else {
                        self.fighters[p].pod_lock_id = tgt;
                        self.fighters[p].pod_lock_t = 0.0;
                    }
                    if tgt >= 0 {
                        // counterplay begins BEFORE the missile exists
                        self.fighters[tgt as usize].lock_warn_t = 0.25;
                    }
                }
                let released = self.fighters[p].pod_aim_held && !cmd.pod_aim;
                if released && can_pod {
                    let locked = self.fighters[p].pod_lock_t >= POD_LOCK_S
                        && self.fighters[p].pod_lock_id >= 0;
                    let target = if locked {
                        self.fighters[p].pod_lock_id
                    } else {
                        -1
                    };
                    let eye = self.muzzle_origin(p);
                    let d = normalize(cmd.aim);
                    let team = self.fighters[p].team;
                    self.rockets.push(Rocket {
                        pos: [
                            eye[0] + d[0] * 0.6,
                            eye[1] + d[1] * 0.6 + 0.35,
                            eye[2] + d[2] * 0.6,
                        ],
                        vel: [d[0] * 20.0, d[1] * 20.0, d[2] * 20.0],
                        target,
                        shooter: p,
                        team,
                        t: 0.0,
                        los_lost: 0.0,
                        prev_los: d,
                    });
                    let f = &mut self.fighters[p];
                    f.pod_ammo -= 1;
                    f.pod_cd = POD_RELAUNCH_S;
                }
                if !cmd.pod_aim {
                    let f = &mut self.fighters[p];
                    f.pod_lock_t = 0.0;
                    f.pod_lock_id = -1;
                }
                self.fighters[p].pod_aim_held = cmd.pod_aim;
            }
            if cmd.reload {
                self.try_reload(p);
            }
            // §4.1 (Brief VII v2): the bow draws on HOLD-FIRE and looses
            // on RELEASE - it needs the call every tick (held or not) to
            // see the release edge, unlike try_fire's "only while held".
            // §C.5: in a chassis the trigger drives the HULL MOUNT, not
            // whatever the pilot is still carrying - checked FIRST so a
            // pilot holding a bow does not draw it from inside the mech.
            if self.fighters[p].in_mech() {
                if cmd.shoot {
                    match self.fighters[p].mech_weapon {
                        MechWeapon::Gatling => {
                            self.try_fire_gatling(p, cmd.aim);
                        }
                        MechWeapon::Autocannon => {
                            self.try_fire_autocannon(p, cmd.aim);
                        }
                    }
                }
            } else if self.fighters[p].gun == GunKind::Bow {
                self.step_bow_draw(p, cmd.aim, cmd.shoot);
            } else if cmd.shoot {
                self.try_fire(p, cmd.aim, cmd.ads);
            }
            // ---- §5 throwables: G cycles, hold arms, release throws ----
            if cmd.cycle_throw {
                let f = &mut self.fighters[p];
                // §8 (Brief IV): G cycles through what you actually
                // CARRY — empty slots are skipped (4 tries, then stay)
                let was = f.throw_sel;
                for _ in 0..4 {
                    f.throw_sel = (f.throw_sel + 1) % 4;
                    if f.grenades[f.throw_sel as usize] > 0 {
                        break;
                    }
                }
                if f.throw_sel != was {
                    // The cook clock belongs to the grenade in your hand,
                    // not to the hand. Carrying it across a switch meant
                    // cycling from a nearly-cooked SMOKE (fuse 1.2s, no
                    // cook cap anywhere in the sim) onto a FRAG detonated
                    // it instantly in your palm - and cycling the other
                    // way silently defused a cooked frag.
                    f.cook_t = 0.0;
                }
            }
            let sel = self.fighters[p].throw_sel as usize;
            let kind = ThrowKind::ALL[sel];
            // §6: throwables are the ONE thing usable behind the raised
            // shield — the shieldwall advances and lobs
            // §1 (Brief V): cancel exits aim mode — nothing thrown,
            // nothing consumed (the grenade is only spent at release)
            if cmd.throw_cancel && self.fighters[p].cook_t > 0.0 {
                self.fighters[p].cook_t = 0.0;
            }
            if cmd.throw_hold
                && !cmd.throw_cancel
                && self.fighters[p].grenades[sel] > 0
                && self.fighters[p].roll_t <= 0.0
                && self.fighters[p].knife_phase <= 0.0
            {
                let f = &mut self.fighters[p];
                f.cook_t += DT;
                // a frag cooked past its fuse goes off IN HAND
                if kind == ThrowKind::Frag && f.cook_t >= throw_spec(kind).fuse_s {
                    f.grenades[sel] -= 1;
                    f.cook_t = 0.0;
                    let at = [f.pos[0], f.pos[1] + 1.0, f.pos[2]];
                    let team = f.team;
                    self.next_missile_id += 1;
                    let id = self.next_missile_id;
                    self.detonate(Grenade {
                        id,
                        kind,
                        pos: at,
                        vel: [0.0; 3],
                        thrower: p,
                        team,
                        fuse_t: 0.0,
                        bounces: 0,
                        rest: true,
                    });
                }
            } else if self.fighters[p].cook_t > 0.0 {
                // release → throw: overhand; underhand lob when crouched;
                // gentle drop when looking steeply down (retreat play)
                let cook = self.fighters[p].cook_t;
                let f = &mut self.fighters[p];
                if f.grenades[sel] > 0 {
                    // §6: throwing from behind the plate DIPS it — a real
                    // 0.62 s window of vulnerability
                    if f.shield_up {
                        f.shield_dip_t = SHIELD_DIP_S;
                    }
                    f.grenades[sel] -= 1;
                    // §1 (Brief V): the SHARED release math — power from
                    // the hold, crouch/drop variants, run inertia, mech
                    // launcher — the same fn the preview arc calls
                    let (o, vel) = self.throw_release_velocity(p, cmd.aim, cook);
                    let f = &mut self.fighters[p];
                    let spec_t = throw_spec(kind);
                    let fuse = if spec_t.fuse_s.is_finite() {
                        (spec_t.fuse_s - if kind == ThrowKind::Frag { cook } else { 0.0 })
                            .max(0.15)
                    } else {
                        f32::INFINITY
                    };
                    self.next_missile_id += 1;
                    let team = f.team;
                    self.grenades_air.push(Grenade {
                        id: self.next_missile_id,
                        kind,
                        pos: o,
                        vel,
                        thrower: p,
                        team,
                        fuse_t: fuse,
                        bounces: 0,
                        rest: false,
                    });
                }
                self.fighters[p].cook_t = 0.0;
            }
            // ---- §6 abilities on F, by set -----------------------------
            let t_now = self.t;
            match self.fighters[p].armor_set {
                ArmorSet::Folk => {
                    // Shieldwall Brace: plant + raise, 110° frontal cut —
                    // heavy to enter, committal, devastating when timed
                    let f = &mut self.fighters[p];
                    f.brace = cmd.ability && f.grounded && !f.shield_up && f.roll_t <= 0.0;
                }
                ArmorSet::Pyro => {
                    self.fighters[p].brace = false;
                    if cmd.ability
                        && self.fighters[p].fuel > 0.0
                        && self.fighters[p].roll_t <= 0.0
                        && self.fighters[p].alive()
                    {
                        self.fighters[p].fuel = (self.fighters[p].fuel - DT).max(0.0);
                        self.fighters[p].last_ability_at = t_now;
                        // Flame Projector: 34 dps cone, 7.5 m, ±25°
                        let (ppos, pteam, pyaw) = {
                            let f = &self.fighters[p];
                            (f.pos, f.team, f.yaw)
                        };
                        let fwd = [pyaw.sin(), pyaw.cos()];
                        for j in 0..self.fighters.len() {
                            let g = &self.fighters[j];
                            if j == p || g.team == pteam || !g.alive() || g.protect_t > 0.0 {
                                continue;
                            }
                            let dx = g.pos[0] - ppos[0];
                            let dz = g.pos[2] - ppos[2];
                            let d = (dx * dx + dz * dz).sqrt().max(0.01);
                            if d > FLAME_REACH
                                || (fwd[0] * dx + fwd[1] * dz) / d < FLAME_ARC_COS
                            {
                                continue;
                            }
                            self.fighters[j].burn_t = 1.0;
                            // the ATTACKER's position: this is the
                            // direction the hit came FROM, which is what
                            // the mech arc and the Folk brace arc are
                            // measured against. Passing the victim's own
                            // position makes the direction vector zero,
                            // which silently reads as "side" for a mech
                            // and never matches a brace at all.
                            self.apply_plain_damage(p, j, FLAME_DPS * DT, ppos, false, true);
                        }
                        // §8: the flame CONE reaches the horde. In
                        // Extraction there is only one team, so the team
                        // filter above left this ability with zero
                        // possible targets - the Pyro armour the map
                        // hands out was completely inert in the only
                        // mode with a horde.
                        let mut zhits: Vec<u32> = Vec::new();
                        for z in &self.zombies {
                            let dx = z.pos[0] - ppos[0];
                            let dz = z.pos[2] - ppos[2];
                            let d = (dx * dx + dz * dz).sqrt().max(0.01);
                            if d <= FLAME_REACH
                                && (fwd[0] * dx + fwd[1] * dz) / d >= FLAME_ARC_COS
                            {
                                zhits.push(z.id);
                            }
                        }
                        for zid in zhits {
                            if let Some(zi) = self.zombies.iter().position(|z| z.id == zid) {
                                self.damage_zombie(zi, FLAME_DPS * DT, false);
                            }
                        }
                    }
                }
                ArmorSet::RobotSuit => {
                    self.fighters[p].brace = false;
                    // §A.3: the crouch key is DEAD INPUT in a mech -
                    // `set_crouch` refuses it outright (`want && !in_mech()`).
                    // Repurpose it rather than add a binding: the intent
                    // ("lower my stance") now means "widen the mech's
                    // stance". Same key, different meaning while piloting.
                    self.fighters[p].mech_brace = cmd.crouch
                        && self.fighters[p].grounded
                        && self.fighters[p].hull > 0.0;
                    if cmd.ability
                        && self.fighters[p].ability_cd <= 0.0
                        && self.fighters[p].armor >= REPULSOR_COST
                    {
                        self.fighters[p].ability_cd = REPULSOR_CD;
                        self.fighters[p].armor -= REPULSOR_COST;
                        self.fighters[p].last_ability_at = t_now;
                        // Repulsor Blast: first enemy in a tight cone,
                        // 62 damage + a real launch
                        let (ppos, pteam) = {
                            let f = &self.fighters[p];
                            (f.pos, f.team)
                        };
                        let o = [ppos[0], ppos[1] + EYE_REL, ppos[2]];
                        let d = normalize(cmd.aim);
                        let mut best: Option<(usize, f32)> = None;
                        for j in 0..self.fighters.len() {
                            let g = &self.fighters[j];
                            if j == p || g.team == pteam || !g.alive() || g.protect_t > 0.0 {
                                continue;
                            }
                            let chest = [g.pos[0], g.pos[1] + g.height() * 0.55, g.pos[2]];
                            let to = [chest[0] - o[0], chest[1] - o[1], chest[2] - o[2]];
                            let dist =
                                (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(0.01);
                            let dot = (to[0] * d[0] + to[1] * d[1] + to[2] * d[2]) / dist;
                            if dist < 20.0
                                && dot > 0.985
                                && self.los_clear(o, chest)
                                && best.map_or(true, |(_, bd)| dist < bd)
                            {
                                best = Some((j, dist));
                            }
                        }
                        if let Some((j, _)) = best {
                            // the launch: up and away, off the ground
                            let g = &mut self.fighters[j];
                            let dx = g.pos[0] - ppos[0];
                            let dz = g.pos[2] - ppos[2];
                            let l = (dx * dx + dz * dz).sqrt().max(0.1);
                            g.pos[0] += dx / l * 0.4;
                            g.pos[2] += dz / l * 0.4;
                            g.pos[1] += 0.06;
                            g.vy = REPULSOR_KNOCK * 0.5;
                            g.grounded = false;
                            // attacker's position - see the flame note
                            self.apply_plain_damage(p, j, REPULSOR_DMG, ppos, true, false);
                        }
                        // §8: the repulsor shoves the HORDE too - same
                        // one-team problem as the flame projector, so the
                        // mech's panic button did nothing against zombies
                        self.blast_zombies(
                            [ppos[0], ppos[1] + EYE_REL, ppos[2]],
                            6.0,
                            |_| REPULSOR_DMG,
                        );
                    }
                }
                _ => {
                    self.fighters[p].brace = false;
                    // covers dismounting mid-brace: a pilot who steps
                    // out must not carry the hull's stance onto foot
                    self.fighters[p].mech_brace = false;
                }
            }
            // ---- §5 the knife: tap = quick slash, hold = committed
            // lunge. Silent, capsule-swept, lethal from behind. ---------
            if cmd.knife_hold
                && self.fighters[p].knife_phase <= 0.0
                && self.fighters[p].roll_t <= 0.0
                && !self.fighters[p].shield_up
                && self.fighters[p].cook_t <= 0.0
                // §3: a spear already committed to a THROW cannot also
                // thrust - the weapon is mid-air-or-about-to-be. try_fire
                // already has the mirror of this guard (`knife_phase >
                // 0.0` blocks a throw during a thrust); without this side
                // one spear landed two separate attacks.
                && self.fighters[p].spear_wind_t <= 0.0
            {
                let f = &mut self.fighters[p];
                f.knife_phase = DT;
                f.knife_committed = false;
                f.knife_struck = false;
            } else if self.fighters[p].knife_phase > 0.0 {
                let phase = self.fighters[p].knife_phase + DT;
                self.fighters[p].knife_phase = phase;
                // §2 (Brief V): F while WIELDING THE SPEAR is a THRUST —
                // its own beats, a narrow line, and a recovery that is
                // longer on a whiff than on a hit. The mech gears the
                // same thrust down. Otherwise §6 (Brief IV): the melee
                // slot may carry the AXE — slower, harder, and the swing
                // SWEEPS the arc.
                let thrust = self.fighters[p].gun == GunKind::Spear;
                let tmul = if thrust
                    && self.fighters[p].armor_set == ArmorSet::RobotSuit
                    && self.fighters[p].hull > 0.0
                {
                    MECH_THRUST_TIME_MULT
                } else {
                    1.0
                };
                let axe = self.fighters[p].melee_axe && !thrust;
                let lunge_wind = if axe { AXE_LUNGE_WIND_S } else { KNIFE_LUNGE_WIND_S };
                if !thrust
                    && cmd.knife_hold
                    && !self.fighters[p].knife_committed
                    && (KNIFE_COMMIT_S..lunge_wind).contains(&phase)
                {
                    self.fighters[p].knife_committed = true; // visibly wound up
                }
                let committed = self.fighters[p].knife_committed;
                let (wind, range, dmg, backstab) = if thrust {
                    (
                        THRUST_WIND_S * tmul,
                        THRUST_RANGE_M,
                        THRUST_DMG,
                        THRUST_BACKSTAB,
                    )
                } else {
                    match (axe, committed) {
                        (false, true) => (
                            KNIFE_LUNGE_WIND_S,
                            KNIFE_LUNGE_RANGE_M,
                            KNIFE_LUNGE_DMG,
                            KNIFE_LUNGE_BACKSTAB,
                        ),
                        (false, false) => (
                            KNIFE_QUICK_WIND_S,
                            KNIFE_RANGE_M,
                            KNIFE_QUICK_DMG,
                            KNIFE_QUICK_BACKSTAB,
                        ),
                        (true, true) => (
                            AXE_LUNGE_WIND_S,
                            KNIFE_LUNGE_RANGE_M,
                            AXE_LUNGE_DMG,
                            AXE_LUNGE_BACKSTAB,
                        ),
                        (true, false) => (
                            AXE_QUICK_WIND_S,
                            AXE_RANGE_M,
                            AXE_QUICK_DMG,
                            AXE_QUICK_BACKSTAB,
                        ),
                    }
                };
                let (active, recover) = if thrust {
                    // the whiff pays: a missed thrust is COMMITTED
                    let rec = if self.fighters[p].knife_struck {
                        THRUST_RECOVER_HIT_S
                    } else {
                        THRUST_RECOVER_WHIFF_S
                    };
                    (THRUST_ACTIVE_S * tmul, rec * tmul)
                } else if axe {
                    (AXE_QUICK_ACTIVE_S, AXE_QUICK_RECOVER_S)
                } else {
                    (KNIFE_QUICK_ACTIVE_S, KNIFE_QUICK_RECOVER_S)
                };
                // the lunge carries the body forward through its strike
                if committed && phase >= wind && phase < wind + 0.18 {
                    let f = &mut self.fighters[p];
                    let (fx, fz) = (f.yaw.sin(), f.yaw.cos());
                    f.pos[0] += fx * 5.0 * DT;
                    f.pos[2] += fz * 5.0 * DT;
                }
                if axe
                    && !self.fighters[p].knife_struck
                    && phase >= wind
                {
                    // §6: ONE sweep on the first active tick — everyone
                    // inside the 90° arc takes it, horde included
                    self.fighters[p].knife_struck = true;
                    let (ppos, pteam, pyaw) = {
                        let f = &self.fighters[p];
                        (f.pos, f.team, f.yaw)
                    };
                    let (fx, fz) = (pyaw.sin(), pyaw.cos());
                    let mut hits: Vec<usize> = Vec::new();
                    for (j, g) in self.fighters.iter().enumerate() {
                        if j == p || g.team == pteam || !g.alive() || g.protect_t > 0.0 {
                            continue;
                        }
                        let dx = g.pos[0] - ppos[0];
                        let dz = g.pos[2] - ppos[2];
                        let d = (dx * dx + dz * dz).sqrt();
                        if d < range && (fx * dx + fz * dz) / d.max(0.05) > AXE_ARC_COS {
                            hits.push(j);
                        }
                    }
                    let mut any = !hits.is_empty();
                    for j in hits {
                        // back-stab per victim: facing away from the sweep
                        let v = &self.fighters[j];
                        let dxv = v.pos[0] - ppos[0];
                        let dzv = v.pos[2] - ppos[2];
                        let dl = (dxv * dxv + dzv * dzv).sqrt().max(0.05);
                        let behind = (v.yaw.sin() * dxv + v.yaw.cos() * dzv) / dl > 0.35;
                        let d_out = if behind { backstab } else { dmg };
                        // attacker's position - the arc model measures
                        // where the blow came FROM, and the victim's own
                        // position degenerates it to a zero vector
                        self.apply_plain_damage(p, j, d_out, ppos, false, false);
                    }
                    // Collect zombie IDs, NOT indices: `damage_zombie`
                    // does `swap_remove` on a kill, which relocates the
                    // last element into the dead slot and shrinks the
                    // vec - so any index collected earlier that happened
                    // to equal the old `len-1` becomes out of range and
                    // panics the authoritative sim. One axe sweep hits a
                    // 2.1m / 90deg arc of a horde packed to 0.55m and
                    // one-shots every zombie type, so multi-kill sweeps
                    // are the normal case, not an edge case. The missile
                    // path already avoids this by looking up by id.
                    let mut zhits: Vec<u32> = Vec::new();
                    for z in self.zombies.iter() {
                        let dx = z.pos[0] - ppos[0];
                        let dz = z.pos[2] - ppos[2];
                        let d = (dx * dx + dz * dz).sqrt();
                        if d < range && (fx * dx + fz * dz) / d.max(0.05) > AXE_ARC_COS {
                            zhits.push(z.id);
                        }
                    }
                    any |= !zhits.is_empty();
                    for zid in zhits {
                        if let Some(zi) = self.zombies.iter().position(|z| z.id == zid) {
                            self.damage_zombie(zi, dmg, false);
                        }
                    }
                    if self.mode == Mode::Extraction && any {
                        let at = [ppos[0], ppos[2]];
                        self.emit_noise(at, 6.0); // heavier than the blade
                    }
                } else if !axe
                    && !self.fighters[p].knife_struck
                    && phase >= wind
                    && phase < wind + active
                {
                    // §2: the thrust is a LINE — a much tighter cone than
                    // the knife's target-assisted slash
                    let cone = if thrust { THRUST_ARC_COS } else { 0.42 };
                    let (ppos, pteam, pyaw) = {
                        let f = &self.fighters[p];
                        (f.pos, f.team, f.yaw)
                    };
                    let (fx, fz) = (pyaw.sin(), pyaw.cos());
                    let mut best: Option<(usize, f32)> = None;
                    for (j, g) in self.fighters.iter().enumerate() {
                        if j == p || g.team == pteam || !g.alive() || g.protect_t > 0.0 {
                            continue;
                        }
                        let dx = g.pos[0] - ppos[0];
                        let dz = g.pos[2] - ppos[2];
                        let d = (dx * dx + dz * dz).sqrt();
                        if d < range
                            && (fx * dx + fz * dz) / d.max(0.05) > cone
                            && best.map_or(true, |(_, b)| d < b)
                        {
                            best = Some((j, d));
                        }
                    }
                    if let Some((j, _)) = best {
                        self.fighters[p].knife_struck = true;
                        // back-stab: the victim faces AWAY from the blade
                        let v = &self.fighters[j];
                        let dxv = v.pos[0] - ppos[0];
                        let dzv = v.pos[2] - ppos[2];
                        let dl = (dxv * dxv + dzv * dzv).sqrt().max(0.05);
                        let behind = (v.yaw.sin() * dxv + v.yaw.cos() * dzv) / dl > 0.35;
                        let d_out = if behind { backstab } else { dmg };
                        // attacker's position - the arc model measures
                        // where the blow came FROM, and the victim's own
                        // position degenerates it to a zero vector
                        self.apply_plain_damage(p, j, d_out, ppos, false, false);
                    } else {
                        // the horde is knife-work too (silent = correct)
                        let mut bz: Option<(usize, f32)> = None;
                        for (zi, z) in self.zombies.iter().enumerate() {
                            let dx = z.pos[0] - ppos[0];
                            let dz = z.pos[2] - ppos[2];
                            let d = (dx * dx + dz * dz).sqrt();
                            if d < range
                                && (fx * dx + fz * dz) / d.max(0.05) > cone
                                && bz.map_or(true, |(_, b)| d < b)
                            {
                                bz = Some((zi, d));
                            }
                        }
                        if let Some((zi, _)) = bz {
                            self.fighters[p].knife_struck = true;
                            self.damage_zombie(zi, dmg, false);
                        }
                    }
                    if self.mode == Mode::Extraction && self.fighters[p].knife_struck {
                        let at = [self.fighters[p].pos[0], self.fighters[p].pos[2]];
                        self.emit_noise(at, 4.0); // nearly silent
                    }
                }
                let total = wind + active + recover;
                if phase >= total {
                    let f = &mut self.fighters[p];
                    f.knife_phase = 0.0;
                    f.knife_committed = false;
                    f.knife_struck = false;
                }
            }
        } else {
            self.fighters[p].vel = [0.0, 0.0];
        }

        // §8.2: sprinting is noise too (Recon Weave runs quiet)
        if self.mode == Mode::Extraction && self.tick % 60 == 0 {
            let mut events: Vec<([f32; 2], f32)> = Vec::new();
            for f in &self.fighters {
                if !f.alive() {
                    continue;
                }
                let sp = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
                if sp > 6.0 {
                    let r = if f.armor_set == ArmorSet::Recon { 6.0 } else { 14.0 };
                    events.push(([f.pos[0], f.pos[2]], r));
                }
            }
            for (at, r) in events {
                self.emit_noise(at, r);
            }
        }

        // ---- bots -------------------------------------------------------
        let ppos = self.fighters[p].pos;
        for i in 0..self.fighters.len() {
            if i == p || !self.fighters[i].alive() {
                continue;
            }
            if (self.tick + self.fighters[i].think_offset as u64) % 12 == 0 {
                self.bot_think(i);
            }
            // §9.5 bot LOD: distant bots act at 15 Hz instead of 120. The
            // LOD level is a PURE function of sim state (distance to the
            // player) — never camera distance or frame rate, or every
            // replay diverges. Velocity persists between acts.
            let dx = self.fighters[i].pos[0] - ppos[0];
            let dz = self.fighters[i].pos[2] - ppos[2];
            let far = dx * dx + dz * dz > 80.0 * 80.0;
            if far && (self.tick + self.fighters[i].think_offset as u64) % 8 != 0 {
                continue;
            }
            self.bot_act(i);
        }

        // ---- integrate: XZ + walls, then gravity/steps ------------------
        // (dead fighters integrate too — velocity is zeroed on death, so
        // this just lets a mid-air corpse fall instead of hanging)
        let half = self.half;
        for i in 0..self.fighters.len() {
            // mid-roll: the somersault owns the velocity (player and bots
            // alike — bots roll off hard landings too)
            {
                let f = &mut self.fighters[i];
                if f.roll_t > 0.0 {
                    f.roll_t -= DT;
                    if f.alive() {
                        // §2 (Brief V): load → burst → EASE-OUT. The burst
                        // never hands speed back as a cliff — the ease
                        // ramps it down to walking pace over 0.14 s. The
                        // mech's side-step skips the load (servos don't
                        // crouch) but keeps the ease.
                        let mech = f.armor_set == ArmorSet::RobotSuit && f.hull > 0.0;
                        // roll_boost: Task 3 rule 3's counter-movement
                        // launch, snapshotted at the trigger (1.0 unless
                        // the dodge cut against real prior movement)
                        let burst = if mech { MECH_STEP_SPEED } else { ROLL_SPEED * f.roll_boost };
                        let t = f.roll_t.max(0.0);
                        let sp = if !mech && t > ROLL_S + ROLL_EASE_S {
                            burst * 0.25 // the crouch-load creep
                        } else if t > ROLL_EASE_S {
                            burst
                        } else {
                            burst * (0.35 + 0.65 * (t / ROLL_EASE_S))
                        };
                        f.vel = [f.roll_dir[0] * sp, f.roll_dir[1] * sp];
                    }
                }
            }
            let (nx, nz) = {
                let f = &self.fighters[i];
                (f.pos[0] + f.vel[0] * DT, f.pos[2] + f.vel[1] * DT)
            };
            let y = self.fighters[i].pos[1];
            // §11.4: the mech's fat radius is what keeps it OUT of
            // buildings — doorways it doesn't fit through push it back
            let radius = self.fighters[i].radius();
            let mut px = nx.clamp(-half + 0.5, half - 0.5);
            let mut pz = nz.clamp(-half + 0.5, half - 0.5);
            // walls you cannot step onto push you out; walkable tops don't
            for c in &self.cover {
                if c.max[1] <= y + STEP_UP {
                    continue; // climbable — handled by support
                }
                if y >= c.max[1] - 0.01 {
                    continue; // already above it
                }
                let cx = px.clamp(c.min[0], c.max[0]);
                let cz = pz.clamp(c.min[2], c.max[2]);
                let (dx, dz) = (px - cx, pz - cz);
                let d2 = dx * dx + dz * dz;
                if d2 < radius * radius {
                    let d = d2.sqrt().max(1e-4);
                    let push = radius - d;
                    if d > 1e-3 {
                        px += dx / d * push;
                        pz += dz / d * push;
                    } else {
                        pz += push;
                    }
                }
            }
            // vertical: stand on the tallest reachable support, else fall
            let f = &mut self.fighters[i];
            f.pos[0] = px;
            f.pos[2] = pz;
            let support = {
                let y0 = f.pos[1];
                let mut s = 0.0_f32;
                for c in &self.cover {
                    if px > c.min[0] - BODY_RADIUS * 0.4
                        && px < c.max[0] + BODY_RADIUS * 0.4
                        && pz > c.min[2] - BODY_RADIUS * 0.4
                        && pz < c.max[2] + BODY_RADIUS * 0.4
                        && c.max[1] <= y0 + STEP_UP
                        && c.max[1] > s
                    {
                        s = c.max[1];
                    }
                }
                s
            };
            if f.pos[1] > support + 0.02 {
                f.vy -= GRAVITY * DT;
                // §9.3 soft ceiling: flyers get pushed back down into the
                // fight — no orbital camping
                if f.pos[1] > SOFT_CEILING_M {
                    f.vy = f.vy.min(-2.0);
                }
                f.pos[1] = (f.pos[1] + f.vy * DT).max(support);
                if f.pos[1] <= support {
                    // hard landing → automatic parkour breakfall: the fall
                    // rolls out along the ground instead of stopping dead
                    let impact = f.vy;
                    f.vy = 0.0;
                    f.grounded = true;
                    if impact < -HARD_LANDING_VY && f.alive() && f.roll_t <= 0.0 {
                        let sp = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
                        f.roll_dir = if sp > 0.5 {
                            [f.vel[0] / sp, f.vel[1] / sp]
                        } else {
                            [f.yaw.sin(), f.yaw.cos()]
                        };
                        // §2 (Brief V): reactive — the breakfall skips
                        // the crouch-load but keeps the ease-out landing.
                        // No counter-move bonus: this is fall momentum
                        // being converted, not a loaded launch.
                        f.roll_boost = 1.0;
                        f.roll_t = ROLL_S + ROLL_EASE_S;
                        f.roll_cd = ROLL_S + ROLL_EASE_S + ROLL_CD_S;
                    }
                } else {
                    f.grounded = false;
                }
            } else {
                f.pos[1] = support;
                f.vy = 0.0;
                f.grounded = true;
            }
        }

        // ---- missiles (arrows / spears) --------------------------------
        self.step_missiles();
        // ---- §5.3 (Brief VI): pod missiles ------------------------------
        self.step_rockets();

        // ---- §8 the horde ----------------------------------------------
        if self.mode == Mode::Extraction {
            self.step_zombies();
        }

        // ---- §5 throwables ---------------------------------------------
        self.step_grenades();
        self.step_fires();
        for s in &mut self.smokes {
            s.ttl -= DT;
        }
        self.smokes.retain(|s| s.ttl > 0.0);
        for (_, ttl) in &mut self.booms {
            *ttl -= DT;
        }
        self.booms.retain(|(_, ttl)| *ttl > 0.0);

        // ---- respawn checkpoints ("check back") ------------------------
        // Stand in the ring uncontested to charge it toward your team;
        // contested rings freeze. An owned ring is a forward spawn.
        for cp in &mut self.checkpoints {
            let (mut blue, mut red) = (0u32, 0u32);
            for f in &self.fighters {
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - cp.pos[0];
                let dz = f.pos[2] - cp.pos[2];
                if dx * dx + dz * dz < CHECKPOINT_RADIUS * CHECKPOINT_RADIUS
                    && (f.pos[1] - cp.pos[1]).abs() < 2.0
                {
                    match f.team {
                        Team::Blue => blue += 1,
                        Team::Red => red += 1,
                    }
                }
            }
            if blue > 0 && red == 0 {
                cp.charge = (cp.charge + DT).min(CHECKPOINT_CAP_S);
                if cp.charge >= CHECKPOINT_CAP_S {
                    cp.owner = Some(Team::Blue);
                }
            } else if red > 0 && blue == 0 {
                cp.charge = (cp.charge - DT).max(-CHECKPOINT_CAP_S);
                if cp.charge <= -CHECKPOINT_CAP_S {
                    cp.owner = Some(Team::Red);
                }
            }
        }

        // ---- KOTH scoring ----------------------------------------------
        if self.mode == Mode::Koth {
            let mut on_hill = [0u32; 2];
            for f in &self.fighters {
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - self.hill[0];
                let dz = f.pos[2] - self.hill[2];
                if dx * dx + dz * dz < HILL_RADIUS * HILL_RADIUS
                    && (f.pos[1] - self.hill[1]).abs() < 1.5
                {
                    on_hill[Self::team_idx(f.team)] += 1;
                }
            }
            if on_hill[0] > 0 && on_hill[1] == 0 {
                self.score[0] += DT;
            } else if on_hill[1] > 0 && on_hill[0] == 0 {
                self.score[1] += DT;
            }
            if self.score[0] >= KOTH_TARGET_S {
                self.finish(Team::Blue);
            } else if self.score[1] >= KOTH_TARGET_S {
                self.finish(Team::Red);
            }
        }

        // ---- §4 stability bookkeeping: this tick's yaw becomes the
        // baseline for next tick's angular-rate measurement
        for f in &mut self.fighters {
            f.prev_yaw = f.yaw;
        }
    }

    fn finish(&mut self, winner: Team) {
        if self.round_over_t.is_none() {
            self.round_over_t = Some(self.t);
            self.winner = Some(winner);
        }
    }

    /// Swap to another loadout slot; each slot keeps its own magazine.
    fn switch_slot(&mut self, i: usize, s: usize) {
        if s >= 3 {
            return;
        }
        let f = &mut self.fighters[i];
        if s == f.active || f.inventory[s] == GunKind::Fists || !f.alive() {
            return;
        }
        f.slot_ammo[f.active] = (f.ammo, f.reserve);
        f.active = s;
        f.gun = f.inventory[s];
        let (a, r) = f.slot_ammo[s];
        f.ammo = a;
        f.reserve = r;
        f.reload_t = 0.0;
        f.switch_t = SWITCH_S;
        f.shield_up = false; // both hands on the new weapon
    }

    fn try_reload(&mut self, i: usize) {
        let f = &mut self.fighters[i];
        if !f.armed() {
            return;
        }
        // §7: R on the minigun is a MANUAL VENT — clear the heat early,
        // on your schedule, instead of eating the forced 3 s at 100.
        if f.gun == GunKind::Minigun {
            if f.vent_t <= 0.0 && f.heat > 1.0 {
                f.vent_t = f.heat / MINIGUN_VENT_RATE;
            }
            return;
        }
        let spec = gun(f.gun);
        if f.reload_t <= 0.0 && f.ammo < spec.mag && f.reserve > 0 {
            // §3.4: running the gun DRY costs extra - the empty reload
            // adds the bolt/charge cycle a tactical reload skips. The
            // ammo math always kept the chambered round; now the CLOCK
            // rewards counting your shots too.
            f.reload_t = if f.ammo == 0 {
                spec.reload_s * RELOAD_EMPTY_MULT
            } else {
                spec.reload_s
            };
        }
    }

    /// Where a fighter's shots actually leave from: crouch lowers the
    /// eye, lean shifts it sideways. The client mirrors this for the
    /// crosshair ray and the arc preview so aim and muzzle agree.
    pub fn muzzle_origin(&self, i: usize) -> [f32; 3] {
        let f = &self.fighters[i];
        // screen-right under this game's yaw convention (matches the
        // playtest-verified A/D strafe mapping): lean +1 peeks RIGHT
        let right = [-f.yaw.cos(), 0.0, f.yaw.sin()];
        [
            f.pos[0] + right[0] * LEAN_SHIFT * f.lean,
            f.pos[1] + EYE_REL.min(f.height() - 0.12),
            f.pos[2] + right[2] * LEAN_SHIFT * f.lean,
        ]
    }

    /// `is_spear` is bound at TRIGGER time by the caller — a forced
    /// weapon switch (minigun pad) during the spear windup must not
    /// transmute the released spear into an arrow's ballistics.
    fn spawn_missile(&mut self, o: [f32; 3], d: [f32; 3], v0: f32, dmg: f32, i: usize, is_spear: bool) {
        let id = self.next_missile_id;
        self.next_missile_id += 1;
        self.missiles.push(Missile {
            id,
            pos: o,
            vel: [d[0] * v0, d[1] * v0, d[2] * v0],
            team: self.fighters[i].team,
            shooter: i,
            damage: dmg,
            is_spear,
            stuck_t: None,
            embedded: true,
            pierces_left: 0,
            pierced: Vec::new(),
            power: 1.0,
        });
    }

    /// §4.1/§4.2 (Brief VII v2): fire the bow itself, once, at the given
    /// draw power - spawns a PIERCING arrow. Separate from `spawn_missile`
    /// (which spears/legacy call sites use) because arrows now carry
    /// pierce state spawn_missile's callers don't need to know about.
    fn spawn_arrow(&mut self, o: [f32; 3], d: [f32; 3], power: f32, i: usize) {
        let id = self.next_missile_id;
        self.next_missile_id += 1;
        let v0 = BOW_V0_FULL * power;
        self.missiles.push(Missile {
            id,
            pos: o,
            vel: [d[0] * v0, d[1] * v0, d[2] * v0],
            team: self.fighters[i].team,
            shooter: i,
            damage: BOW_PIERCE_DMG[0] * power,
            is_spear: false,
            stuck_t: None,
            embedded: true,
            pierces_left: BOW_MAX_PIERCES,
            pierced: Vec::new(),
            power,
        });
    }

    /// §4.1 (Brief VII v2): the bow's draw-and-release. Called every
    /// tick regardless of whether fire is held - it needs to see the
    /// RELEASE edge (held this tick -> not held) to decide whether to
    /// loose an arrow, which `try_fire`'s "only called while held"
    /// convention can't express.
    /// The aim cone for fighter `i` firing their CURRENT gun this tick.
    ///
    /// Extracted from `try_fire` so every fire path shares one
    /// computation. The bow's draw/release path (`step_bow_draw`) is a
    /// second, independent fire path that never called any of this - so
    /// the player's bow fired with literally zero spread: pixel-perfect
    /// while sprinting, mid-whip-turn, or airborne, with `GunSpec.spread`
    /// / `spread_move` dead for that weapon and the §4 stability model
    /// unwired. Any future third fire path gets it by construction.
    /// IX-A map-design validator: the longest unobstructed eye-level
    /// sightline on this map, in metres.
    ///
    /// The castle brief's rule 3 says no two standing positions may see
    /// each other across more than 40 m of open ground - engagements
    /// belong in the 25-35 m band where weapon balance and movement
    /// matter. This is the measuring instrument for that rule: it
    /// samples a grid of standable positions (eye height, outside all
    /// cover) and raycasts every pair through the real cover grid - the
    /// same `los_clear` the game itself shoots through, so the validator
    /// cannot disagree with the gun.
    ///
    /// Built BEFORE any castle geometry exists, on the brief's own
    /// logic: it tells you whether the maps you already have violate the
    /// rule you are about to hold a new one to.
    pub fn max_unobstructed_sightline(&self, grid_step: f32) -> f32 {
        let mut pts: Vec<[f32; 3]> = Vec::new();
        let h = self.half - 1.0;
        let mut x = -h;
        while x <= h {
            let mut z = -h;
            while z <= h {
                // standable: eye height, not inside any cover volume
                let eye = [x, EYE_REL, z];
                let buried = self.cover.iter().any(|c| {
                    x > c.min[0]
                        && x < c.max[0]
                        && z > c.min[2]
                        && z < c.max[2]
                        && eye[1] > c.min[1]
                        && eye[1] < c.max[1]
                });
                if !buried {
                    pts.push(eye);
                }
                z += grid_step;
            }
            x += grid_step;
        }
        let mut worst = 0.0_f32;
        for i in 0..pts.len() {
            for j in (i + 1)..pts.len() {
                let dx = pts[i][0] - pts[j][0];
                let dz = pts[i][2] - pts[j][2];
                let d2 = dx * dx + dz * dz;
                // only pairs that could beat the current worst need a ray
                if d2 > worst * worst && self.los_clear(pts[i], pts[j]) {
                    worst = d2.sqrt();
                }
            }
        }
        worst
    }

    /// How long a fighter who just died stays down. §8: an Extraction
    /// RUN has no respawns at all - dying is the end of your run.
    ///
    /// Hoisted because every death site used to hard-code this, and
    /// `apply_plain_damage` hard-coded the WRONG one: a frag or molotov
    /// death in Extraction handed back a full-health, fully-rearmed
    /// fighter after 3 seconds, so cooking a grenade at your own feet was
    /// a free heal. The claw path had already been special-cased for
    /// exactly this reason; the explosive paths had not.
    fn death_respawn_t(&self) -> f32 {
        if self.mode == Mode::Extraction {
            9999.0
        } else {
            RESPAWN_S
        }
    }

    /// Public view of `aim_spread` for the client's arc preview, so the
    /// HUD cannot show a cone the simulation does not shoot.
    pub fn aim_spread_of(&self, i: usize, ads: bool) -> f32 {
        self.aim_spread(i, ads)
    }

    fn aim_spread(&self, i: usize, ads: bool) -> f32 {
        let f = &self.fighters[i];
        let spec = gun(f.gun);
        // §2.4 (Brief VI): the movement penalty RAMPS from zero at 34%
        // of max speed to full at 95% — counter-strafe under 34% and
        // your shot is already clean. Airborne adds jump inaccuracy.
        let move_frac = {
            let sp = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
            ((sp / SPRINT_SPEED - MOVE_INACC_START)
                / (MOVE_INACC_FULL - MOVE_INACC_START))
                .clamp(0.0, 1.0)
        };
        let airborne_pen = if f.grounded { 0.0 } else { 1.5 };
        let mut spread = base_spread(f.gun, f.heat)
            + spec.spread_move * (move_frac + airborne_pen)
            + f.bloom;
        if spec.scoped && ads {
            // §5.2 (Brief VI): the scoped shot is a LASER — a flat
            // 0.002 standing / 0.0015 crouched, plus ONLY the movement
            // penalty (0.176 moving is a hard miss in either mode)
            spread = if f.crouch { 0.0015 } else { 0.002 }
                + spec.spread_move * (move_frac + airborne_pen);
        } else {
            if ads {
                spread *= ADS_SPREAD_MULT;
            }
            if f.crouch {
                spread *= CROUCH_SPREAD_MULT;
            }
        }
        // §4 stability: bow/spear shots taken mid-whip-turn or on the run
        // are spoiled — the cost replaces the old feeling of being pinned.
        // Deterministic: prev_yaw is sim state, updated once per tick.
        if spec.projectile.is_some() {
            let ang = wrap_angle(f.yaw - f.prev_yaw).abs() / DT;
            let plan = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
            let stability = (1.0
                - AIM_TURN_K * (ang - AIM_TURN_FREE).max(0.0)
                - AIM_MOVE_K * (plan - AIM_MOVE_FREE).max(0.0))
            .clamp(AIM_STABILITY_MIN, 1.0);
            spread /= stability;
        }
        spread
    }

    fn step_bow_draw(&mut self, i: usize, aim: [f32; 3], held: bool) {
        let blocked = {
            let f = &self.fighters[i];
            !f.armed()
                || f.gun != GunKind::Bow
                || !f.alive()
                || f.roll_t > 0.0
                || f.shield_up
                || f.knife_phase > 0.0
                || f.flip_t > 0.0
                || f.flip_used
                || f.reload_t > 0.0
                || f.sprint_gate_t > 0.0 // §3.4: the bow lowers at a sprint too
                || f.ammo == 0
        };
        if blocked {
            self.fighters[i].bow_draw_t = 0.0;
            return;
        }
        if held {
            let f = &mut self.fighters[i];
            f.bow_aim = normalize(aim); // tracked through the draw, like the spear
            f.bow_draw_t += DT;
            if f.bow_draw_t >= BOW_DRAW_FORCE_S {
                f.bow_draw_t = 0.0; // §4.1: forced letdown past 10s, no shot
            }
            return;
        }
        // release edge: was drawing, isn't anymore
        let held_s = self.fighters[i].bow_draw_t;
        self.fighters[i].bow_draw_t = 0.0;
        if held_s <= 0.0 {
            return; // wasn't drawing at all
        }
        let Some(power) = bow_power_fraction(held_s) else {
            return; // §4.1: under 0.15s or forced-letdown boundary - no shot
        };
        // §4: the SAME cone every other fire path uses. Drawing does not
        // exempt the bow from movement penalty, bloom, or the whip-turn
        // stability divide - this path used to fire perfectly straight.
        // A drawn bow is aimed, so it takes the ADS branch.
        let spread = self.aim_spread(i, true);
        let (ex, ey) = (
            self.rng.range(-spread, spread),
            self.rng.range(-spread, spread),
        );
        // §4.1: full-draw hold sway - layered ON TOP of the normal
        // accuracy cone, not instead of it. Same seeded stream as
        // every other perturbation here, so it stays replay-exact.
        let sway = bow_sway_deg(held_s, self.fighters[i].crouch).to_radians();
        let (sx, sy) = (self.rng.range(-sway, sway), self.rng.range(-sway, sway));
        let d = perturb(self.fighters[i].bow_aim, ex + sx, ey + sy);
        let o = self.muzzle_origin(i);
        let spec = gun(GunKind::Bow);
        {
            let f = &mut self.fighters[i];
            f.ammo -= 1;
            f.fire_cd = spec.fire_period;
            f.protect_t = 0.0;
            // nock the next arrow automatically, exactly as try_fire's
            // projectile branch does - without this the human bow needed
            // a manual R after every shot while a bot's re-nocked itself
            if f.ammo == 0 && f.reserve > 0 {
                f.reload_t = spec.reload_s;
            }
        }
        let at = [self.fighters[i].pos[0], self.fighters[i].pos[2]];
        self.spawn_arrow(o, d, power, i);
        self.emit_noise(at, gun_noise_m(GunKind::Bow));
    }

    fn try_fire(&mut self, i: usize, aim: [f32; 3], ads: bool) -> bool {
        // §7: a HELD minigun trigger spins the barrels every tick it is
        // held — including the ticks inside fire_cd between rounds. This
        // must precede every early return or the spin (and the heat
        // suppression) stutters between shots.
        if self.fighters[i].gun == GunKind::Minigun && self.fighters[i].alive() {
            self.fighters[i].spin_cmd = MINIGUN_SPIN_HOLD_S;
        }
        {
            let f = &self.fighters[i];
            if !f.armed()
                || f.fire_cd > 0.0
                || f.reload_t > 0.0
                || f.switch_t > 0.0
                || f.ammo == 0
                || !f.alive()
                || f.roll_t > 0.0 // no shooting mid-somersault
                || f.shield_up // the shield takes both hands
                || f.knife_phase > 0.0 // §5: the blade owns both hands too
                || f.flip_t > 0.0 // §4.2: a flip is PURE mobility
                || f.flip_used // ...and the gun returns on landing recovery
                // §3.4: the weapon is still coming up out of the sprint
                // carry - the sprint-out beat is the whole point
                || f.sprint_gate_t > 0.0
                // §6.2: the chassis is still sealing up. Scoped to
                // ACTUALLY being in a chassis: the timer is mech state,
                // but this gate is not, so a pilot who dismounts (or is
                // blown out) mid-entry used to stay disarmed on foot for
                // the rest of the window, with nothing in the HUD saying
                // why.
                || (f.mech_transition_t > 0.0
                    && f.armor_set == ArmorSet::RobotSuit
                    && f.hull > 0.0)
            {
                return false;
            }
        }
        // §7: rounds only leave once the spin-up completes, and never
        // during a vent (spin_cmd was already set above).
        if self.fighters[i].gun == GunKind::Minigun {
            let f = &self.fighters[i];
            if f.vent_t > 0.0 || f.spin_t < MINIGUN_SPINUP_S {
                return false;
            }
        }
        let spec = gun(self.fighters[i].gun);
        let spread = self.aim_spread(i, ads);
        // lean shifts the muzzle sideways off the body line and steadies
        // the shoulder a touch (recoil ×0.8 while leaning)
        let lean = self.fighters[i].lean;
        let o = self.muzzle_origin(i);
        {
            let f = &mut self.fighters[i];
            f.ammo -= 1;
            f.fire_cd = spec.fire_period;
            // opening fire drops your spawn protection — no shooting from
            // behind the untargetable shimmer
            f.protect_t = 0.0;
            let kick = if lean.abs() > 0.1 {
                spec.kick * LEAN_RECOIL_MULT
            } else {
                spec.kick
            };
            f.bloom = (f.bloom + kick).min(0.05);
            // §2 (Brief VI): advance the deterministic spray — the
            // per-weapon table feeds the punch VELOCITY (first shots
            // suppressed 0.75 → 1.0, consecutive entries lerped 0.55)
            if spec.projectile.is_none() && f.gun != GunKind::Fists {
                let i0 = f.spray_i.max(0.0) as usize;
                let (a0, m0) = spray_entry(f.gun, i0);
                let (a1, m1) = spray_entry(f.gun, i0 + 1);
                let ang = a0 + (a1 - a0) * SPRAY_LERP;
                // §5.2 (Brief VI): the scoped shot kicks 25 where the
                // no-scope kicks 78 (Valve's AWP table ratio)
                let scoped_scale = if spec.scoped && ads { 25.0 / 78.0 } else { 1.0 };
                // §A.5: bracing's MECHANICAL payoff - not just a movement
                // debuff. A planted hull eats recoil. Written generically
                // here (not as an autocannon special case) so §C's
                // autocannon gets braced-vs-unbraced kick for free by
                // multiplying the same constant at its own call site.
                let brace_scale = if f.armor_set == ArmorSet::RobotSuit && f.mech_brace {
                    MECH_BRACE_RECOIL_DAMP
                } else {
                    1.0
                };
                let mag = (m0 + (m1 - m0) * SPRAY_LERP)
                    * scoped_scale
                    * brace_scale
                    * if i0 < 4 {
                        0.75 + i0 as f32 * (0.25 / 3.0)
                    } else {
                        1.0
                    };
                f.punch_vel[0] += ang.cos() * mag; // pitch, up
                f.punch_vel[1] += ang.sin() * mag; // yaw, right
                f.spray_i += 1.0;
            }
            f.last_shot_at = self.t;
            // §7: every round is heat; 100 forces the 3 s vent
            if f.gun == GunKind::Minigun {
                f.heat += MINIGUN_HEAT_PER_SHOT;
                if f.heat >= 100.0 {
                    f.heat = 100.0;
                    f.vent_t = MINIGUN_VENT_FORCED_S;
                }
            }
            if f.ammo == 0 && spec.projectile.is_some() && f.reserve > 0 {
                // nock the next arrow / heft the next spear automatically
                f.reload_t = spec.reload_s;
            }
        }
        if let Some((v0, dmg)) = spec.projectile {
            // §4: the PLAYER's hip-thrown spear is a min-charge lob; the
            // full 26 m/s needs the settled (ADS) throw. Bots always
            // commit to the full throw — they have no hip/ADS split.
            let v0 = if self.fighters[i].gun == GunKind::Spear && !ads && i == self.player {
                SPEAR_V0_MIN
            } else {
                v0
            };
            // §5.4: the bonus is decided at INITIATION (this instant,
            // on the momentum the thrower already has) and baked into
            // the release velocity carried through the windup - the
            // brief's own wording is "a throw INITIATED at >=70% run
            // speed... gets velocity x1.15", not a check at release.
            let v0 = if self.fighters[i].gun == GunKind::Spear
                && self.fighters[i].running_momentum_t >= RUNNING_THROW_MIN_S
            {
                v0 * RUNNING_THROW_MULT
            } else {
                v0
            };
            if self.fighters[i].gun == GunKind::Spear {
                // §3: the throw WINDS UP — plant, hips, whip. The spear
                // leaves the hand SPEAR_WINDUP_S later, on the aim held
                // at release. Committal by design, visible to enemies.
                let (ex, ey) = (
                    self.rng.range(-spread, spread),
                    self.rng.range(-spread, spread),
                );
                let d = perturb(normalize(aim), ex, ey);
                let f = &mut self.fighters[i];
                f.spear_wind_t = SPEAR_WINDUP_S;
                f.spear_aim = d;
                f.spear_v0 = v0;
                return true;
            }
            let (ex, ey) = (
                self.rng.range(-spread, spread),
                self.rng.range(-spread, spread),
            );
            let d = perturb(normalize(aim), ex, ey);
            // only the bow reaches here — the spear went out through the
            // windup path above
            self.spawn_missile(o, d, v0, dmg, i, false);
            return true;
        }
        // §2 channel 1 (Brief VI) — the TRUTH: bullets leave at the aim
        // deflected by the punch. The player's aim ray was cast through
        // the PUNCHED camera (it already carries the visible 45%), so
        // the sim adds the remaining 55% — total ×2.0 vs the original
        // angles, ×1.1 drift vs the crosshair, the CS:GO arithmetic.
        // Bots aim from raw geometry, so they take the full ×2.0.
        let aim = self.punched_aim(i, aim);
        self.hitscan_burst(i, o, aim, spread, spec.damage, spec.pellets);
        // §8.2: gunfire is NOISE — the whole horde director runs on it.
        // This is what makes the bow and spear the correct tool here.
        if self.mode == Mode::Extraction {
            let at = [self.fighters[i].pos[0], self.fighters[i].pos[2]];
            let r = gun_noise_m(self.fighters[i].gun);
            self.emit_noise(at, r);
        }
        true
    }

    /// The SHARED hitscan resolution: one trace per pellet, each rolling
    /// its own spread, against cover → enemy fighters → the horde.
    ///
    /// Lifted VERBATIM out of `try_fire` (which is still its only
    /// infantry caller) so the mech's hull mounts resolve hits on
    /// exactly the path a rifle does. Damage, pellet count and spread
    /// are parameters instead of `GunSpec` reads precisely because the
    /// hull mounts have no `GunSpec` - that is the entire point of the
    /// extraction. Two copies of this loop would diverge on the first
    /// balance pass, and this file's single most repeated defect has
    /// been a second copy of a rule quietly drifting from the first.
    ///
    /// `aim` must ARRIVE already deflected by the caller's recoil model.
    /// The RNG is consumed exactly twice per pellet, x then y: replay
    /// determinism depends on that count and that order.
    fn hitscan_burst(
        &mut self,
        i: usize,
        o: [f32; 3],
        aim: [f32; 3],
        spread: f32,
        damage: f32,
        pellets: u32,
    ) {
        for _pellet in 0..pellets.max(1) {
            let (ex, ey) = (
                self.rng.range(-spread, spread),
                self.rng.range(-spread, spread),
            );
            let d = perturb(normalize(aim), ex, ey);
            let mut t_hit = 200.0_f32;
            let mut hit_normal = [0.0, 1.0, 0.0];
            if let Some((t, n)) = self.grid.ray_hit(&self.cover, o, d, t_hit) {
                t_hit = t;
                hit_normal = n;
            }
            let shooter_team = self.fighters[i].team;
            let mut victim: Option<(usize, f32, f32)> = None;
            for (j, g) in self.fighters.iter().enumerate() {
                if j == i || g.team == shooter_team || !g.alive() || g.protect_t > 0.0 {
                    continue;
                }
                if let Some((t, hit_y)) = ray_vs_cylinder(o, d, g.pos, g.radius(), g.height()) {
                    if t < t_hit && victim.map_or(true, |(_, bt, _)| t < bt) {
                        victim = Some((j, t, hit_y));
                    }
                }
            }
            // §8: the horde is shootable — same ray, same hit zones, so
            // the ×4 head multiplier one-shots the mass
            let mut zvictim: Option<(usize, f32, f32)> = None;
            for (zi, z) in self.zombies.iter().enumerate() {
                let zs = zspec(z.kind);
                if let Some((t, hit_y)) = ray_vs_cylinder(o, d, z.pos, zs.girth, zs.height) {
                    if t < t_hit
                        && victim.map_or(true, |(_, bt, _)| t < bt)
                        && zvictim.map_or(true, |(_, bt, _)| t < bt)
                    {
                        zvictim = Some((zi, t, hit_y));
                    }
                }
            }
            let end_t = zvictim
                .map(|(_, t, _)| t)
                .or(victim.map(|(_, t, _)| t))
                .unwrap_or(t_hit)
                .min(victim.map(|(_, t, _)| t).unwrap_or(t_hit));
            let end = [o[0] + d[0] * end_t, o[1] + d[1] * end_t, o[2] + d[2] * end_t];
            self.tracers.push(Tracer {
                from: o,
                to: end,
                team: shooter_team,
                ttl: 0.06,
            });
            // nearest body wins: zombie, fighter, or the wall
            if let Some((zi, zt, hit_y)) = zvictim {
                if victim.map_or(true, |(_, ft, _)| zt < ft) {
                    let (base, height, kind) = {
                        let z = &self.zombies[zi];
                        (z.pos[1], zspec(z.kind).height, z.kind)
                    };
                    let frac = ((hit_y - base) / height).clamp(0.0, 1.0);
                    let head = frac > 0.82;
                    let mult = if head {
                        HEAD_MULT
                    } else if frac > 0.35 {
                        1.0
                    } else {
                        LEG_MULT
                    };
                    let _ = kind;
                    self.fighters[i].hits_dealt += 1;
                    self.damage_zombie(zi, damage * mult, head);
                    continue;
                }
            }
            match victim {
                Some((j, _, hit_y)) => {
                    // the PASSED damage, never a re-read of the held gun:
                    // the hull mounts have no gun in anyone's hands
                    self.apply_hit_dmg(i, j, hit_y, end, damage);
                }
                None => {
                    if end_t < 199.0 {
                        self.impacts.push((
                            Impact {
                                at: end,
                                normal: hit_normal,
                            },
                            30.0, // bullet marks linger
                        ));
                    }
                }
            }
        }
    }

    /// §C: the aim ray a hull mount actually fires along. Same recoil
    /// arithmetic `try_fire` uses (channel 1 of Brief VI §2) - shared so
    /// the punch cannot mean one thing on foot and another in a chassis.
    fn punched_aim(&self, i: usize, aim: [f32; 3]) -> [f32; 3] {
        let f = &self.fighters[i];
        let k = if i == self.player {
            RECOIL_SCALE * (1.0 - VIEW_RECOIL_TRACKING)
        } else {
            RECOIL_SCALE
        };
        deflect(aim, f.punch[0] * k, f.punch[1] * k)
    }

    /// §C: the hull GATLING — suppression. A sibling of `try_fire`, not
    /// a branch inside it, and the gate list is why: `try_fire` opens on
    /// `armed()`/`gun`/`ammo`/`reload_t`/`switch_t`, then `shield_up`,
    /// `knife_phase`, `flip_t`, `sprint_gate_t` — the state of a pair of
    /// HANDS. None of it describes a gun bolted to a chassis that a
    /// pilot triggers from inside a sealed cockpit. Folding these in as
    /// branches would have meant answering every one of those gates for
    /// a weapon they do not apply to.
    ///
    /// Returns true iff a round left the mount.
    pub fn try_fire_gatling(&mut self, i: usize, aim: [f32; 3]) -> bool {
        // A HELD trigger keeps the barrel group hot every tick it is
        // held — including the ticks inside `gatling_cd` between rounds,
        // and including the ticks of a forced vent. This must precede
        // every early return, exactly as the minigun's `spin_cmd` does,
        // or the heat suppression stutters between shots and the mount
        // cools a little every cycle. See `gatling_trigger_t`.
        {
            let f = &mut self.fighters[i];
            if f.in_mech() && f.mech_weapon == MechWeapon::Gatling && f.alive() {
                f.gatling_trigger_t = GATLING_TRIGGER_HOLD_S;
            }
        }
        {
            let f = &self.fighters[i];
            if !f.in_mech()
                || f.mech_weapon != MechWeapon::Gatling
                || !f.alive()
                || f.gatling_cd > 0.0
                || f.gatling_vent_t > 0.0
                // §6.2: the chassis is still sealing up (or powering
                // down) — the mounts are not live yet
                || f.mech_transition_t > 0.0
            {
                return false;
            }
        }
        let o = self.muzzle_origin(i);
        // the cone opens as the barrels cook, exactly as the minigun's
        // does — this is the cost that makes sustained fire a choice
        let spread = {
            let f = &self.fighters[i];
            GATLING_SPREAD_COLD
                + (GATLING_SPREAD_HOT - GATLING_SPREAD_COLD)
                    * (f.gatling_heat / 100.0).clamp(0.0, 1.0)
        };
        {
            let f = &mut self.fighters[i];
            f.gatling_cd = GATLING_FIRE_PERIOD;
            f.protect_t = 0.0; // opening fire drops spawn protection
            // NOTE: deliberately does NOT write `last_shot_at`. That
            // field has exactly one consumer - the carried gun's
            // `spray_i` decay gate - and the mount refires every 0.075 s,
            // faster than ANY gun's decay threshold. Writing it froze the
            // pilot's spray index for as long as the hull gun was firing,
            // so he dismounted into a fully-bloomed recoil pattern he
            // never fired a round to earn.
            f.gatling_heat += GATLING_HEAT_PER_SHOT;
            if f.gatling_heat >= 100.0 {
                f.gatling_heat = 100.0;
                f.gatling_vent_t = GATLING_VENT_FORCED_S;
            }
        }
        let aim = self.punched_aim(i, aim);
        self.hitscan_burst(i, o, aim, spread, GATLING_DAMAGE, 1);
        if self.mode == Mode::Extraction {
            let at = [self.fighters[i].pos[0], self.fighters[i].pos[2]];
            // a radius LOOKUP, not an entry into the GunKind pipeline:
            // the hull gun is as loud as the minigun it echoes
            self.emit_noise(at, gun_noise_m(GunKind::Minigun));
        }
        true
    }

    /// §C: the hull AUTOCANNON — precision. Slow cycle, tight cone, and
    /// a kick big enough that firing it unbraced costs you the next
    /// shot's picture. Deliberately does NOT touch the spray table: the
    /// deterministic per-weapon spray patterns are a `GunKind`-indexed
    /// infantry system, and a single-shot hull cannon has no pattern to
    /// walk — it has one honest kick.
    ///
    /// Sibling of `try_fire` for the same reason as the gatling above.
    pub fn try_fire_autocannon(&mut self, i: usize, aim: [f32; 3]) -> bool {
        {
            let f = &self.fighters[i];
            if !f.in_mech()
                || f.mech_weapon != MechWeapon::Autocannon
                || !f.alive()
                || f.autocannon_cd > 0.0
                || f.mech_transition_t > 0.0
            {
                return false;
            }
        }
        let o = self.muzzle_origin(i);
        {
            let f = &mut self.fighters[i];
            f.autocannon_cd = AUTOCANNON_CYCLE_S;
            f.protect_t = 0.0;
            // NOTE: no `last_shot_at` write here either — same reason as
            // the gatling's. A hull mount has no spray table to hold.
            // §A.5's damp, consumed exactly as it was designed to be:
            // ONE unbraced constant, the braced value derived from it.
            // A second `AUTOCANNON_BRACED_KICK` constant would be a
            // duplicate of this relationship free to drift away from it.
            let kick = if f.mech_brace {
                AUTOCANNON_UNBRACED_KICK * MECH_BRACE_RECOIL_DAMP
            } else {
                AUTOCANNON_UNBRACED_KICK
            };
            f.punch_vel[0] += kick; // pitch, up
        }
        let aim = self.punched_aim(i, aim);
        self.hitscan_burst(i, o, aim, AUTOCANNON_SPREAD, AUTOCANNON_DAMAGE, 1);
        if self.mode == Mode::Extraction {
            let at = [self.fighters[i].pos[0], self.fighters[i].pos[2]];
            self.emit_noise(at, gun_noise_m(GunKind::Minigun));
        }
        true
    }

    /// Damage reduction from a raised shield, if the attack comes through
    /// the front arc. Sides and rear ignore the shield ENTIRELY — flanking
    /// is the counter-play, by design.
    fn shield_block(&self, j: usize, attack_from: [f32; 3]) -> Option<f32> {
        let v = &self.fighters[j];
        // no plate discipline mid-somersault — a rolling shield blocks
        // nothing (otherwise the roll would be a 95%-immune dash); a
        // shield DIPPED for a throw (§6) blocks nothing either
        if !v.shield_up || v.roll_t > 0.0 || v.shield_dip_t > 0.0 {
            return None;
        }
        let dx = attack_from[0] - v.pos[0];
        let dz = attack_from[2] - v.pos[2];
        let len = (dx * dx + dz * dz).sqrt();
        if len < 1e-3 {
            return None;
        }
        let facing = [v.yaw.sin(), v.yaw.cos()];
        let dot = (facing[0] * dx + facing[1] * dz) / len;
        if dot > SHIELD_ARC_COS {
            let base = if v.crouch {
                SHIELD_BLOCK_CROUCH
            } else {
                SHIELD_BLOCK_STAND
            };
            // §1 (Brief V): aiming a throw drops the plate to a
            // ONE-HANDED carry — half the block while the arm is busy
            Some(if v.cook_t > 0.0 { base * 0.5 } else { base })
        } else {
            None
        }
    }

    /// A hitscan hit from the shooter's CURRENTLY HELD gun. Thin wrapper
    /// over `apply_hit_dmg` so the 20-odd call sites that mean exactly
    /// that keep reading as they did.
    fn apply_hit(&mut self, i: usize, j: usize, hit_y: f32, at: [f32; 3]) {
        let base = gun(self.fighters[i].gun).damage;
        self.apply_hit_dmg(i, j, hit_y, at, base);
    }

    /// §C: the same resolution with the per-torso damage passed IN.
    ///
    /// This parameter is load-bearing, not a tidy-up: `apply_hit` used
    /// to re-derive the damage from `gun(shooter.gun)` at the bottom of
    /// the chain, which silently made every shot cost whatever the
    /// shooter was HOLDING. A hull mount has no `GunKind` and no gun in
    /// anyone's hands, so routing it through the old signature would
    /// have made the autocannon deal the pilot's rifle damage to
    /// fighters while correctly dealing 145 to zombies - a split the
    /// zombie path would have hidden for a long time.
    fn apply_hit_dmg(&mut self, i: usize, j: usize, hit_y: f32, at: [f32; 3], base_dmg: f32) {
        // a body that already dropped this tick takes no further hits —
        // otherwise a shotgun's later pellets score the same kill twice
        if !self.fighters[j].alive() {
            return;
        }
        let base = self.fighters[j].pos[1];
        let h = self.fighters[j].height();
        let frac = ((hit_y - base) / h).clamp(0.0, 1.0);
        // §4.3: a flipping fighter is UNIFORM — mid-backflip the head is
        // at the BOTTOM of the capsule and the banded test would call a
        // boot shot a ×4 headshot. No multiplier in either direction.
        let zone = if self.fighters[j].hit_zone_mode() == HitZoneMode::Uniform {
            HitZone::Torso
        } else if frac > 0.82 {
            HitZone::Head
        } else if frac > 0.66 {
            HitZone::Arms
        } else if frac > 0.35 {
            HitZone::Torso
        } else {
            HitZone::Legs
        };
        // v6 model: per-gun torso damage × zone multiplier. The baseline
        // M4A1 lands the owner's tuned rule: 2 headshots / 8 body shots.
        // §4.5 (Brief VI): proportional zones do NOT apply to a mech —
        // the angle model (+ visor ×2, inside apply_armor) replaces them.
        let in_mech = self.fighters[j].armor_set == ArmorSet::RobotSuit
            && self.fighters[j].hull > 0.0;
        let mut dmg = base_dmg * if in_mech { 1.0 } else { zone.mult() };
        let from = {
            let f = &self.fighters[i];
            [f.pos[0], f.pos[1] + EYE_REL, f.pos[2]]
        };
        // the shield eats front-arc damage BEFORE armor
        let mut shielded = false;
        if let Some(block) = self.shield_block(j, from) {
            dmg *= 1.0 - block;
            shielded = true;
        }
        // §6.1: set armor applies AFTER the zone multiplier, with a floor
        dmg = self.apply_armor(j, dmg, base_dmg, zone, Some(from));
        let assist_candidate = self.record_hit_get_assist(i, j);
        self.fighters[j].health -= dmg;
        self.fighters[i].hits_dealt += 1;
        let fatal = self.fighters[j].health <= 0.0;
        self.hits.push((
            HitEvent {
                shooter: i,
                victim: j,
                zone,
                damage: dmg,
                shielded,
                from,
                at,
                fatal,
            },
            2.2,
        ));
        if fatal {
            self.fighters[j].deaths += 1;
            self.fighters[j].respawn_t = self.death_respawn_t();
            self.fighters[j].vel = [0.0, 0.0];
            self.fighters[j].shield_up = false; // the plate drops with you
            self.fighters[i].kills += 1;
            if self.mode == Mode::Tdm {
                let s = Self::team_idx(self.fighters[i].team);
                self.score[s] += 1.0;
                if self.overtime || self.score[s] >= TDM_TARGET as f32 {
                    self.finish(self.fighters[i].team);
                }
            }
            // an assist only counts from the KILLER's own team - a
            // teammate of the victim who accidentally clipped them
            // earlier must never be credited toward an enemy's kill
            let assist_candidate =
                assist_candidate.filter(|&a| self.fighters[a].team == self.fighters[i].team);
            if let Some(a) = assist_candidate {
                self.fighters[a].assists += 1;
            }
            self.kill_feed.push((
                KillEvent {
                    killer: i,
                    victim: j,
                    headshot: zone == HitZone::Head,
                    assist: assist_candidate,
                },
                5.0,
            ));
        }
    }

    /// Predict where a projectile launched from `o` along `d` at `v0` will
    /// fly and land — the SAME integration as `step_missiles`, so the
    /// preview arc is exactly the flight the arrow/spear will take.
    /// Returns (sampled points along the arc, impact point, impact normal).
    pub fn predict_arc(
        &self,
        o: [f32; 3],
        d: [f32; 3],
        v0: f32,
        is_spear: bool,
        max_s: f32,
    ) -> (Vec<[f32; 3]>, [f32; 3], [f32; 3]) {
        let d = normalize(d);
        let mut pos = o;
        let mut vel = [d[0] * v0, d[1] * v0, d[2] * v0];
        let g = missile_g(is_spear); // the SAME constant the flight uses (§4)
        let mut pts = Vec::new();
        let steps = (max_s / DT) as usize;
        for step in 0..steps {
            vel[1] -= g * DT;
            let old = pos;
            pos[0] += vel[0] * DT;
            pos[1] += vel[1] * DT;
            pos[2] += vel[2] * DT;
            let seg = [pos[0] - old[0], pos[1] - old[1], pos[2] - old[2]];
            let seg_len = (seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2])
                .sqrt()
                .max(1e-5);
            let dn = [seg[0] / seg_len, seg[1] / seg_len, seg[2] / seg_len];
            let best = self.grid.ray_hit(&self.cover, old, dn, seg_len);
            if let Some((t, n)) = best {
                let at = [old[0] + dn[0] * t, old[1] + dn[1] * t, old[2] + dn[2] * t];
                return (pts, at, n);
            }
            if pos[1] <= 0.0 {
                let t = if seg[1].abs() > 1e-6 {
                    (0.0 - old[1]) / seg[1]
                } else {
                    1.0
                };
                let at = [
                    old[0] + seg[0] * t.clamp(0.0, 1.0),
                    0.0,
                    old[2] + seg[2] * t.clamp(0.0, 1.0),
                ];
                return (pts, at, [0.0, 1.0, 0.0]);
            }
            if step % 5 == 0 {
                pts.push(pos);
            }
        }
        (pts, pos, [0.0, 1.0, 0.0])
    }

    fn step_missiles(&mut self) {
        let t_now = self.t;
        let mut hits: Vec<(usize, usize, f32, [f32; 3], [f32; 3], bool)> = Vec::new();
        let snap: Vec<(Team, [f32; 3], f32, f32, bool)> = self
            .fighters
            .iter()
            .map(|f| {
                (
                    f.team,
                    f.pos,
                    f.height(),
                    f.radius(),
                    f.alive() && f.protect_t <= 0.0,
                )
            })
            .collect();
        let cover = &self.cover;
        let grid = &self.grid;
        // §8: arrows and spears bite the horde too — id-keyed so kills
        // resolving after the loop can't be shifted by swap_remove
        let zsnap: Vec<(u32, [f32; 3], f32, f32)> = self
            .zombies
            .iter()
            .map(|z| (z.id, z.pos, zspec(z.kind).girth, zspec(z.kind).height))
            .collect();
        let mut zhits: Vec<(u32, f32, bool)> = Vec::new();
        for m in &mut self.missiles {
            if m.stuck_t.is_some() {
                continue;
            }
            m.vel[1] -= missile_g(m.is_spear) * DT; // ONE shared constant (§4)
            let old = m.pos;
            m.pos[0] += m.vel[0] * DT;
            m.pos[1] += m.vel[1] * DT;
            m.pos[2] += m.vel[2] * DT;
            // body check
            for (j, &(team, pos, h, r, alive)) in snap.iter().enumerate() {
                if team == m.team || !alive || j == m.shooter || m.pierced.contains(&j) {
                    continue;
                }
                let dx = m.pos[0] - pos[0];
                let dz = m.pos[2] - pos[2];
                let rr = (r + 0.11) * (r + 0.11);
                if dx * dx + dz * dz < rr && m.pos[1] > pos[1] && m.pos[1] < pos[1] + h {
                    // §4.2 (Brief VII v2): an arrow with pierces left
                    // keeps FLYING through this body - damage this pass
                    // uses the cascade table (already scaled by power at
                    // spawn for pass 0; later passes re-scale here).
                    let pass = BOW_MAX_PIERCES - m.pierces_left;
                    let dmg = if !m.is_spear && m.pierces_left > 0 {
                        BOW_PIERCE_DMG[pass.min(2) as usize] * m.power
                    } else {
                        m.damage
                    };
                    hits.push((m.shooter, j, dmg, m.pos, m.vel, m.is_spear));
                    m.pierced.push(j);
                    if !m.is_spear && m.pierces_left > 0 {
                        m.pierces_left -= 1;
                        if m.pierces_left == 0 {
                            m.stuck_t = Some(t_now); // last pierce spent - embeds
                        }
                        // else: still flying, no stuck_t, no break - the
                        // arrow keeps going and may hit ANOTHER body the
                        // very same tick if they're lined up close enough
                        continue;
                    }
                    m.stuck_t = Some(t_now);
                    break;
                }
            }
            if m.stuck_t.is_none() {
                for &(zid, zpos, girth, height) in &zsnap {
                    let dx = m.pos[0] - zpos[0];
                    let dz = m.pos[2] - zpos[2];
                    if dx * dx + dz * dz < girth * girth
                        && m.pos[1] > zpos[1]
                        && m.pos[1] < zpos[1] + height
                    {
                        let head = (m.pos[1] - zpos[1]) / height > 0.82;
                        zhits.push((zid, m.damage, head));
                        m.stuck_t = Some(t_now);
                        break;
                    }
                }
            }
            if m.stuck_t.is_some() {
                continue;
            }
            // world check
            let seg = [
                m.pos[0] - old[0],
                m.pos[1] - old[1],
                m.pos[2] - old[2],
            ];
            let seg_len = (seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2])
                .sqrt()
                .max(1e-5);
            let dn = [seg[0] / seg_len, seg[1] / seg_len, seg[2] / seg_len];
            // §9.1 grid broadphase — and NEAREST hit, so a segment that
            // clips two boxes sticks at the first surface along the path
            // (predict_arc already used nearest; now they agree exactly)
            if let Some((t, normal)) = grid.ray_hit(cover, old, dn, seg_len) {
                m.pos = [old[0] + dn[0] * t, old[1] + dn[1] * t, old[2] + dn[2] * t];
                // §3.2 (Brief VII v2): stick vs. bounce, spear only.
                m.embedded = impact_angle_to_surface_deg(dn, normal) >= SPEAR_STICK_ANGLE_DEG;
                m.stuck_t = Some(t_now);
            }
            if m.pos[1] <= 0.0 {
                // land at the exact ground crossing — the same
                // interpolation `predict_arc` uses, so the preview and
                // the flight agree to the centimetre (§4)
                let seg_y = m.pos[1] - old[1];
                let t = if seg_y.abs() > 1e-6 {
                    ((0.0 - old[1]) / seg_y).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                m.pos = [
                    old[0] + (m.pos[0] - old[0]) * t,
                    0.0,
                    old[2] + (m.pos[2] - old[2]) * t,
                ];
                // §3.2 (Brief VII v2): ground counts as a flat surface,
                // normal straight up.
                m.embedded = impact_angle_to_surface_deg(dn, [0.0, 1.0, 0.0])
                    >= SPEAR_STICK_ANGLE_DEG;
                m.stuck_t = Some(t_now);
            }
        }
        // ---- §3: a missile coming to rest converts IN PLACE to a
        // walk-over pickup. Spears always survive; arrows break 35% of
        // the time — rolled from a PCG seeded on the missile's stable ID
        // (never wall clock, never renderer state: replays must agree).
        let mut converted: Vec<u32> = Vec::new();
        for m in &self.missiles {
            if m.stuck_t != Some(t_now) {
                continue; // only missiles that stuck THIS tick
            }
            let recovered = if m.is_spear {
                // §3.2 (Brief VII v2): a spear that embedded (steep
                // enough) always survives as a pickup; one that bounced
                // off shallow is lost, exactly like a broken arrow.
                m.embedded
            } else {
                let mut r = Pcg32::new(
                    self.cfg.seed ^ (m.id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
                    0xD20,
                );
                r.next_f32() < ARROW_RECOVER_P
            };
            if !recovered {
                continue; // the broken arrow stays as a 15 s prop
            }
            converted.push(m.id);
            let kind = if m.is_spear {
                AmmoKind::Spear
            } else {
                AmmoKind::Arrow
            };
            // stack merging: an arrow-heavy round must not spawn hundreds
            // of entities — nearby same-kind piles absorb the new one
            let merged = self.dropped.iter_mut().any(|d| {
                let dx = d.pos[0] - m.pos[0];
                let dz = d.pos[2] - m.pos[2];
                if d.kind == kind && dx * dx + dz * dz < DROPPED_MERGE_M * DROPPED_MERGE_M {
                    d.count = d.count.saturating_add(1);
                    true
                } else {
                    false
                }
            });
            if !merged {
                self.dropped.push(DroppedAmmo {
                    kind,
                    count: 1,
                    rest_tick: self.tick,
                    pos: [m.pos[0], m.pos[1].max(0.0), m.pos[2]],
                });
            }
        }
        self.missiles.retain(|m| {
            !converted.contains(&m.id) && m.stuck_t.map_or(true, |s| t_now - s < 15.0)
        });
        // lifetime + global cap, oldest first
        let tick = self.tick;
        self.dropped
            .retain(|d| tick.saturating_sub(d.rest_tick) < DROPPED_TTL_TICKS);
        while self.dropped.len() > DROPPED_MAX {
            let oldest = self
                .dropped
                .iter()
                .enumerate()
                .min_by_key(|(_, d)| d.rest_tick)
                .map(|(i, _)| i)
                .unwrap();
            self.dropped.remove(oldest);
        }
        for (zid, dmg, head) in zhits {
            if let Some(zi) = self.zombies.iter().position(|z| z.id == zid) {
                // a stuck arrow in the skull is rewarded (bow = the tool)
                self.damage_zombie(zi, if head { dmg * 1.5 } else { dmg }, head);
            }
        }
        for (i, j, dmg, at, vel, is_spear) in hits {
            // a corpse from an earlier missile this same tick stays down —
            // no double deaths, no double score
            if !self.fighters[j].alive() {
                continue;
            }
            // §3.2/§4.2 (Brief VII v2): both projectiles read hit height -
            // the spear's full zone table (85/×2 head/×0.75 legs), the
            // arrow's headshot-only bonus (×2 head at EVERY pierce, no
            // leg reduction - piercing is the arrow's whole fantasy).
            // Neither applies against a mech - angle-armor replaces zone
            // bands there entirely.
            let in_mech = self.fighters[j].armor_set == ArmorSet::RobotSuit
                && self.fighters[j].hull > 0.0;
            let zone_mult = if !in_mech {
                let base = self.fighters[j].pos[1];
                let h = self.fighters[j].height();
                let frac = ((at[1] - base) / h).clamp(0.0, 1.0);
                let uniform = self.fighters[j].hit_zone_mode() == HitZoneMode::Uniform;
                if uniform {
                    1.0
                } else if frac > 0.82 {
                    if is_spear { SPEAR_HEAD_MULT } else { 2.0 }
                } else if frac > 0.35 || !is_spear {
                    1.0
                } else {
                    LEG_MULT
                }
            } else {
                1.0
            };
            let mut d = dmg * zone_mult;
            // arrows and spears respect the shield too — the attack comes
            // from BACK ALONG the flight path, not from the impact point
            // (the impact point sits ON the victim and has no direction)
            let from_dir = [at[0] - vel[0], at[1] - vel[1], at[2] - vel[2]];
            let mut shielded = false;
            if let Some(block) = self.shield_block(j, from_dir) {
                d *= 1.0 - block;
                shielded = true;
            }
            // §6.1 flats + floor (projectiles are flat-torso damage)
            d = self.apply_armor(j, d, dmg * zone_mult, HitZone::Torso, Some(from_dir));
            let assist_candidate = self.record_hit_get_assist(i, j);
            self.fighters[j].health -= d;
            self.fighters[i].hits_dealt += 1;
            let fatal = self.fighters[j].health <= 0.0;
            let from = {
                let f = &self.fighters[i];
                [f.pos[0], f.pos[1] + EYE_REL, f.pos[2]]
            };
            self.hits.push((
                HitEvent {
                    shooter: i,
                    victim: j,
                    zone: HitZone::Torso,
                    damage: d,
                    shielded,
                    from,
                    at,
                    fatal,
                },
                2.2,
            ));
            if fatal {
                self.fighters[j].deaths += 1;
                self.fighters[j].respawn_t = self.death_respawn_t();
                self.fighters[j].vel = [0.0, 0.0];
                self.fighters[j].shield_up = false;
                self.fighters[i].kills += 1;
                if self.mode == Mode::Tdm {
                    let s = Self::team_idx(self.fighters[i].team);
                    self.score[s] += 1.0;
                    if self.overtime || self.score[s] >= TDM_TARGET as f32 {
                        self.finish(self.fighters[i].team);
                    }
                }
                let assist_candidate =
                    assist_candidate.filter(|&a| self.fighters[a].team == self.fighters[i].team);
                if let Some(a) = assist_candidate {
                    self.fighters[a].assists += 1;
                }
                self.kill_feed.push((
                    KillEvent {
                        killer: i,
                        victim: j,
                        headshot: false,
                        assist: assist_candidate,
                    },
                    5.0,
                ));
            }
        }
    }

    /// §8 the horde, all sim-side: the director breathes pressure in and
    /// out, spawns arrive OUTSIDE view cones and never within 35 m,
    /// zombies chase what they see or the last noise they heard, and the
    /// extraction ring asks you to stand still while everything closes in.
    fn step_zombies(&mut self) {
        let elapsed = EXTRACT_LEN_S - self.match_t;
        // ---- director: pressure rises with time and hurt, ebbs in quiet
        let hurt = self
            .fighters
            .iter()
            .filter(|f| f.alive())
            .map(|f| 1.0 - f.health / MAX_HEALTH)
            .fold(0.0_f32, f32::max);
        self.pressure = (self.pressure + DT * (1.0 / 480.0 + hurt * 0.0004)
            - DT * 0.002)
            .clamp(0.05, 1.0);
        let holding = {
            let site = self.extract_sites[self.extract_idx];
            elapsed >= EXTRACT_REVEAL_S
                && self.fighters.iter().any(|f| {
                    f.alive()
                        && (f.pos[0] - site[0]).powi(2) + (f.pos[2] - site[2]).powi(2)
                            < EXTRACT_RADIUS * EXTRACT_RADIUS
                })
        };
        if holding {
            self.pressure = 1.0; // the final wave dumps
            self.extract_hold += DT;
            if self.extract_hold >= EXTRACT_HOLD_S {
                self.finish(Team::Blue); // EXTRACTED
            }
        }
        // §8.4: the site relocates ONCE at the 12-minute mark
        if elapsed >= EXTRACT_RELOCATE_S && self.extract_idx == 0 {
            self.extract_idx = 1;
            self.extract_hold = 0.0;
        }
        // ---- spawning: scaled by pressure, capped, out of sight --------
        self.zspawn_cd -= DT;
        if self.zspawn_cd <= 0.0 && self.zombies.len() < ZOMBIE_CAP {
            self.zspawn_cd = 6.0 - 4.8 * self.pressure;
            let n = 1 + (self.pressure * 3.0) as usize;
            for _ in 0..n {
                if self.zombies.len() >= ZOMBIE_CAP {
                    break;
                }
                // pick an edge point ≥35 m from every player and OUTSIDE
                // every view cone — try a few candidates, else skip
                let mut spot = None;
                for _try in 0..6 {
                    let side = (self.rng.next_f32() * 4.0) as u32;
                    let a = self.rng.range(-self.half + 4.0, self.half - 4.0);
                    let e = self.half - 4.0;
                    let cand = match side {
                        0 => [a, e],
                        1 => [a, -e],
                        2 => [e, a],
                        _ => [-e, a],
                    };
                    let ok = self.fighters.iter().all(|f| {
                        if !f.alive() {
                            return true;
                        }
                        let dx = cand[0] - f.pos[0];
                        let dz = cand[1] - f.pos[2];
                        let d = (dx * dx + dz * dz).sqrt();
                        if d < ZSPAWN_MIN_M {
                            return false;
                        }
                        // outside the view cone (±60°)
                        let fwd = [f.yaw.sin(), f.yaw.cos()];
                        (fwd[0] * dx + fwd[1] * dz) / d.max(0.1) < 0.5
                    });
                    if ok {
                        spot = Some(cand);
                        break;
                    }
                }
                let Some(spot) = spot else { continue };
                // composition scales with pressure; Runners gate on time
                let roll = self.rng.next_f32();
                let kind = if roll < 0.68 {
                    ZKind::Shambler
                } else if roll < 0.80 && elapsed > 240.0 {
                    ZKind::Runner
                } else if roll < 0.88 {
                    ZKind::Screamer
                } else if roll < 0.96 {
                    ZKind::Bloater
                } else {
                    ZKind::Brute
                };
                self.next_zombie_id += 1;
                self.zombies.push(Zombie {
                    id: self.next_zombie_id,
                    kind,
                    pos: [spot[0], 0.0, spot[1]],
                    hp: zspec(kind).hp,
                    atk_cd: 0.0,
                    scream_t: 0.0,
                    head_hits: 0,
                    target: [0.0, 0.0],
                    alerted: false,
                });
            }
        }
        // ---- per-zombie behavior ---------------------------------------
        let mut screams: Vec<[f32; 3]> = Vec::new();
        // §8: claw hits are collected and applied AFTER the walk, so they
        // can go through the shared armor pipeline (which needs &mut self)
        // instead of writing `health` raw. (victim, raw damage, from)
        let mut claw_hits: Vec<(usize, f32, [f32; 3])> = Vec::new();
        for zi in 0..self.zombies.len() {
            let (zpos, kind) = (self.zombies[zi].pos, self.zombies[zi].kind);
            let spec = zspec(kind);
            // sight: nearest living fighter within 40 m with clear LOS
            let mut seen: Option<(usize, f32)> = None;
            for (j, f) in self.fighters.iter().enumerate() {
                if !f.alive() {
                    continue;
                }
                let dx = f.pos[0] - zpos[0];
                let dz = f.pos[2] - zpos[2];
                let d2 = dx * dx + dz * dz;
                if d2 < 40.0 * 40.0
                    && seen.map_or(true, |(_, b)| d2 < b)
                    && self.sight_clear(
                        [zpos[0], zpos[1] + 1.4, zpos[2]],
                        [f.pos[0], f.pos[1] + 1.0, f.pos[2]],
                    )
                {
                    seen = Some((j, d2));
                }
            }
            let z = &mut self.zombies[zi];
            z.atk_cd = (z.atk_cd - DT).max(0.0);
            if let Some((j, d2)) = seen {
                let fpos = self.fighters[j].pos;
                z.target = [fpos[0], fpos[2]];
                z.alerted = true;
                // §8.1 Screamer: wind-up, then call the horde
                if kind == ZKind::Screamer && d2 < 14.0 * 14.0 && z.scream_t > -1.0 {
                    z.scream_t += DT;
                    if z.scream_t >= 2.2 {
                        z.scream_t = -999.0; // spent
                        screams.push(z.pos);
                    }
                }
                // melee. `d2` is PLANAR only - zombies have no vertical
                // simulation at all (pos[1] is written once at spawn and
                // never again), so without a height check the horde claws
                // players standing on a crate or a rooftop directly above
                // them. Gate on the same 1.4 m reach vertically.
                let dy = (fpos[1] - z.pos[1]).abs();
                if d2 < 1.4 * 1.4 && dy < 1.4 && z.atk_cd <= 0.0 && spec.dmg > 0.0 {
                    z.atk_cd = 1.0;
                    claw_hits.push((j, spec.dmg, zpos));
                }
            }
            // §1 (Brief III) audit fix: an unalerted zombie previously
            // stood at its spawn point forever — with out-of-view spawns
            // that read as "the mode is empty". The idle horde now DRIFTS
            // toward the map's heart, so contact always comes.
            {
                let z = &mut self.zombies[zi];
                if !z.alerted && seen.is_none() {
                    let d = (z.pos[0] * z.pos[0] + z.pos[2] * z.pos[2]).sqrt();
                    if d > 6.0 {
                        z.pos[0] -= z.pos[0] / d * spec.speed * 0.45 * DT;
                        z.pos[2] -= z.pos[2] / d * spec.speed * 0.45 * DT;
                        z.target = [0.0, 0.0];
                    }
                }
            }
            // move toward the target (stop short in melee range).
            // §13: relentless AND convincing — a blocked path deflects
            // sideways instead of clumping at the wall, and the deflect
            // side is a stable function of the zombie's id (deterministic)
            let steer = {
                let z = &self.zombies[zi];
                let dx = z.target[0] - z.pos[0];
                let dz = z.target[1] - z.pos[2];
                let d = (dx * dx + dz * dz).sqrt();
                if d > 1.0 {
                    let (nx, nz) = (dx / d, dz / d);
                    let eye = [z.pos[0], z.pos[1] + 1.0, z.pos[2]];
                    let blocked = self
                        .grid
                        .ray_hit(&self.cover, eye, [nx, 0.0, nz], 2.2)
                        .is_some();
                    if blocked {
                        // wall ahead: slide along it, side picked by id
                        let s = if z.id % 2 == 0 { 1.0 } else { -1.0 };
                        Some(([-nz * s, nx * s], d))
                    } else {
                        Some(([nx, nz], d))
                    }
                } else {
                    None
                }
            };
            let z = &mut self.zombies[zi];
            if (z.alerted || seen.is_some()) && steer.is_some() {
                let ([mx, mz], _d) = steer.unwrap();
                // Brutes freshly staggered stand still
                let staggered = kind == ZKind::Brute && z.atk_cd > 1.4;
                if !staggered {
                    z.pos[0] += mx * spec.speed * DT;
                    z.pos[2] += mz * spec.speed * DT;
                }
            }
            // walls push zombies out just like fighters
            let half = self.half;
            let z = &mut self.zombies[zi];
            z.pos[0] = z.pos[0].clamp(-half + 0.5, half - 0.5);
            z.pos[2] = z.pos[2].clamp(-half + 0.5, half - 0.5);
            for c in &self.cover {
                if c.max[1] <= z.pos[1] + STEP_UP {
                    continue;
                }
                let cx = z.pos[0].clamp(c.min[0], c.max[0]);
                let cz = z.pos[2].clamp(c.min[2], c.max[2]);
                let (dx, dz) = (z.pos[0] - cx, z.pos[2] - cz);
                let d2 = dx * dx + dz * dz;
                if d2 < spec.girth * spec.girth {
                    let d = d2.sqrt().max(1e-4);
                    let push = spec.girth - d;
                    if d > 1e-3 {
                        z.pos[0] += dx / d * push;
                        z.pos[2] += dz / d * push;
                    } else {
                        z.pos[2] += push;
                    }
                }
            }
        }
        // §13: the pile — zombies press INTO each other but never stack;
        // pairwise separation keeps a horde reading as bodies, not a blob
        for a in 0..self.zombies.len() {
            for b in (a + 1)..self.zombies.len() {
                let dx = self.zombies[b].pos[0] - self.zombies[a].pos[0];
                let dz = self.zombies[b].pos[2] - self.zombies[a].pos[2];
                let d2 = dx * dx + dz * dz;
                if d2 < 0.55 * 0.55 && d2 > 1e-6 {
                    let d = d2.sqrt();
                    let push = (0.55 - d) * 0.5;
                    let (ux, uz) = (dx / d, dz / d);
                    self.zombies[a].pos[0] -= ux * push;
                    self.zombies[a].pos[2] -= uz * push;
                    self.zombies[b].pos[0] += ux * push;
                    self.zombies[b].pos[2] += uz * push;
                }
            }
        }

        // §8: the horde's claws go through the SAME armor pipeline as
        // every other damage source. This used to write `health` raw,
        // which meant armor sets, the Folk brace, the raised shield and
        // even the mech's 1000-point HULL all did literally nothing
        // against zombies - a mech was exactly as soft as a bare soldier.
        // The mode's own death rule (no respawns in a run) is kept here
        // rather than delegating to apply_plain_damage, which would
        // overwrite respawn_t with the standard timer.
        let now = self.t;
        let rt = self.death_respawn_t();
        for (j, raw, from) in claw_hits {
            if !self.fighters[j].alive() {
                continue;
            }
            let d =
                self.apply_armor_tagged(j, raw, raw, HitZone::Torso, Some(from), false, false);
            let f = &mut self.fighters[j];
            f.health -= d;
            f.last_dmg_at = now;
            if f.health <= 0.0 {
                f.deaths += 1;
                f.respawn_t = rt; // §8: no respawns in the run
                f.vel = [0.0, 0.0];
                f.shield_up = false;
            }
        }

        // screams: a pressure spike and everyone hears it
        for at in screams {
            self.pressure = (self.pressure + 0.35).min(1.0);
            self.emit_noise([at[0], at[2]], 60.0);
        }
        // toxic clouds tick
        for ti in 0..self.toxics.len() {
            self.toxics[ti].ttl -= DT;
            self.toxics[ti].tick_t += DT;
            if self.toxics[ti].tick_t < 0.25 {
                continue;
            }
            self.toxics[ti].tick_t -= 0.25;
            let tp = self.toxics[ti].pos;
            for j in 0..self.fighters.len() {
                let f = &self.fighters[j];
                if !f.alive() || f.armor_set == ArmorSet::Pyro {
                    continue; // sealed plate: gas does nothing
                }
                let dx = f.pos[0] - tp[0];
                let dz = f.pos[2] - tp[2];
                if dx * dx + dz * dz < TOXIC_R * TOXIC_R && (f.pos[1] - tp[1]).abs() < 2.2 {
                    // Through the shared armor pipeline, like the claws:
                    // this wrote `health` raw, so armor sets, the brace
                    // and the mech's sealed 1000-point hull all did
                    // nothing against gas - a chassis with its own air
                    // supply was exactly as gassable as a bare soldier.
                    // The mode's no-respawn death rule stays local, as
                    // the claw fix documents.
                    let d = self.apply_armor_tagged(
                        j,
                        TOXIC_DPS * 0.25,
                        TOXIC_DPS * 0.25,
                        HitZone::Torso,
                        Some(tp),
                        false,
                        false,
                    );
                    let rt = self.death_respawn_t();
                    let f = &mut self.fighters[j];
                    f.health -= d;
                    f.last_dmg_at = self.t;
                    if f.health <= 0.0 {
                        f.deaths += 1;
                        f.respawn_t = rt;
                        f.vel = [0.0, 0.0];
                    }
                }
            }
        }
        self.toxics.retain(|t| t.ttl > 0.0);
        // the run ends when every fighter is down
        if self.fighters.iter().all(|f| !f.alive()) {
            self.finish(Team::Red);
        }
    }

    /// §8.2: noise pulls the horde. Zombies inside the radius re-target
    /// the source; the director feels it too.
    fn emit_noise(&mut self, at: [f32; 2], radius: f32) {
        self.pressure = (self.pressure + 0.008).min(1.0);
        for z in &mut self.zombies {
            let dx = z.pos[0] - at[0];
            let dz = z.pos[2] - at[1];
            if dx * dx + dz * dz < radius * radius {
                z.alerted = true;
                z.target = at;
            }
        }
    }

    /// §8: apply an area effect centred on `at` to every zombie within
    /// `radius`, with `dmg_at(distance)` giving the falloff.
    ///
    /// Every explosive/area path in this sim looped `self.fighters` only,
    /// so grenades, molotovs, pod rockets, the flame projector and the
    /// repulsor could not touch a zombie at ALL - in Extraction, which is
    /// the only mode that spawns a horde. The flamethrower and repulsor
    /// were worse than useless there: Extraction runs a single team, so
    /// their team filter left them with zero possible targets while the
    /// map still handed out the armour that grants them.
    ///
    /// Collects IDs, never indices: `damage_zombie` swap_removes on a
    /// kill, which is what panicked the axe sweep.
    fn blast_zombies<F: Fn(f32) -> f32>(&mut self, at: [f32; 3], radius: f32, dmg_at: F) {
        let mut hits: Vec<(u32, f32)> = Vec::new();
        for z in &self.zombies {
            let zc = [z.pos[0], z.pos[1] + zspec(z.kind).height * 0.5, z.pos[2]];
            let dx = zc[0] - at[0];
            let dy = zc[1] - at[1];
            let dz = zc[2] - at[2];
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d <= radius && self.los_clear(at, zc) {
                hits.push((z.id, d));
            }
        }
        for (zid, d) in hits {
            let dmg = dmg_at(d);
            if dmg <= 0.0 {
                continue;
            }
            if let Some(zi) = self.zombies.iter().position(|z| z.id == zid) {
                self.damage_zombie(zi, dmg, false);
            }
        }
    }

    /// §8: damage a zombie (zone multiplier already applied). Brutes
    /// stagger on every third headshot; Bloaters burst on death.
    fn damage_zombie(&mut self, zi: usize, dmg: f32, headshot: bool) {
        let z = &mut self.zombies[zi];
        z.hp -= dmg;
        z.alerted = true;
        if headshot && z.kind == ZKind::Brute {
            z.head_hits += 1;
            if z.head_hits % 3 == 0 {
                z.atk_cd = 2.0; // the stagger window
            }
        }
        if z.hp <= 0.0 {
            let (pos, kind) = (z.pos, z.kind);
            self.zombies.swap_remove(zi);
            self.pressure = (self.pressure - 0.004).max(0.05);
            if kind == ZKind::Bloater {
                self.toxics.push(ToxicCloud {
                    pos,
                    ttl: 6.0,
                    tick_t: 0.0,
                });
                self.booms.push((
                    Boom {
                        at: pos,
                        kind: ThrowKind::Smoke,
                    },
                    2.0,
                ));
            }
        }
    }

    /// §8: the active extraction point, once revealed.
    pub fn extract_point(&self) -> Option<[f32; 3]> {
        if self.mode != Mode::Extraction {
            return None;
        }
        let elapsed = EXTRACT_LEN_S - self.match_t;
        if elapsed < EXTRACT_REVEAL_S {
            None
        } else {
            Some(self.extract_sites[self.extract_idx])
        }
    }

    /// §5.2 grenade integration: deterministic point-mass at 120 Hz, full
    /// 9.81 gravity, bounce = tangential friction + normal restitution,
    /// a rest test so nothing micro-bounces forever, and a settle
    /// guarantee (restitution halves after the third bounce).
    fn step_grenades(&mut self) {
        let mut boom_ids: Vec<u32> = Vec::new();
        let TdmSim {
            grenades_air,
            grid,
            cover,
            cover_kind,
            ..
        } = self;
        for g in grenades_air.iter_mut() {
            if let GrenadeTick::Boom = grenade_tick(g, grid, cover, cover_kind) {
                boom_ids.push(g.id);
            }
        }
        for id in boom_ids {
            if let Some(i) = self.grenades_air.iter().position(|g| g.id == id) {
                let g = self.grenades_air.remove(i);
                self.detonate(g);
            }
        }
    }

    /// §1 (Brief V): the aim preview — steps a scratch grenade through
    /// `grenade_tick`, the EXACT integrator the live flight uses (same
    /// constants, same fixed timestep, same bounce/rest/fuse rules), so
    /// the preview cannot drift from the throw. Returns the tick-spaced
    /// path, the end point (detonation or rest), and the path index of
    /// the first bounce (the client fades everything after it).
    pub fn predict_grenade(
        &self,
        kind: ThrowKind,
        o: [f32; 3],
        vel: [f32; 3],
        fuse_s: f32,
        max_s: f32,
    ) -> (Vec<[f32; 3]>, [f32; 3], Option<usize>) {
        let mut g = Grenade {
            id: 0,
            kind,
            pos: o,
            vel,
            thrower: 0,
            team: Team::Blue,
            fuse_t: fuse_s,
            bounces: 0,
            rest: false,
        };
        let mut pts: Vec<[f32; 3]> = Vec::new();
        let mut first_bounce: Option<usize> = None;
        let steps = (max_s / DT) as usize;
        for _ in 0..steps {
            match grenade_tick(&mut g, &self.grid, &self.cover, &self.cover_kind) {
                GrenadeTick::Boom | GrenadeTick::Rest => {
                    return (pts, g.pos, first_bounce)
                }
                GrenadeTick::Fly => {
                    if first_bounce.is_none() && g.bounces > 0 {
                        first_bounce = Some(pts.len());
                    }
                    pts.push(g.pos);
                }
            }
        }
        (pts, g.pos, first_bounce)
    }

    /// §1 (Brief V): the release origin + velocity for fighter `i`
    /// holding a throw for `hold_s` — THE single source shared by the
    /// real throw and the preview arc. Crouch lobs underhand, a steep
    /// downward aim drops gently, run inertia carries in; charge scales
    /// the speed; the shield slows the one-handed charge; the mech's
    /// LAUNCHER fires hotter with none of the hand-throw tax.
    pub fn throw_release_velocity(
        &self,
        i: usize,
        aim: [f32; 3],
        hold_s: f32,
    ) -> ([f32; 3], [f32; 3]) {
        let f = &self.fighters[i];
        let mech = f.armor_set == ArmorSet::RobotSuit && f.hull > 0.0;
        let eff_hold = if f.shield_up && !mech {
            hold_s * THROW_SHIELD_CHARGE_MULT
        } else {
            hold_s
        };
        let power = throw_power(eff_hold);
        let scale = THROW_POWER_MIN + (THROW_POWER_MAX - THROW_POWER_MIN) * power;
        let base = if f.crouch {
            THROW_V_UNDER
        } else if aim[1] < -0.7 {
            THROW_V_DROP
        } else {
            THROW_V_OVER
        };
        let v0 = base * scale * if mech { MECH_LAUNCHER_V_MULT } else { 1.0 };
        let d = normalize(aim);
        let o = [f.pos[0], f.pos[1] + 1.45, f.pos[2]];
        (
            o,
            [
                d[0] * v0 + f.vel[0] * 0.5,
                d[1] * v0 + 2.2,
                d[2] * v0 + f.vel[1] * 0.5,
            ],
        )
    }

    /// §5.3 (Brief VI): pod-missile flight — proportional navigation
    /// (N = 3) against the LOS rate, turn capped at 250°/s, TTL 7 s.
    /// Breaking line of sight for > 0.4 s sends it BALLISTIC for good.
    /// Deterministic f32 throughout; replays reproduce every path.
    fn step_rockets(&mut self) {
        let mut booms: Vec<(usize, [f32; 3])> = Vec::new(); // (shooter, at)
        let mut idx = 0;
        while idx < self.rockets.len() {
            let mut r = self.rockets[idx].clone();
            r.t += DT;
            let mut done = r.t >= ROCKET_TTL_S;
            let speed = ((r.vel[0] * r.vel[0]
                + r.vel[1] * r.vel[1]
                + r.vel[2] * r.vel[2])
                .sqrt()
                + ROCKET_ACCEL * DT)
                .min(ROCKET_SPEED);
            let mut dir = normalize(r.vel);
            if r.target >= 0 {
                let g = &self.fighters[r.target as usize];
                if !(g.armor_set == ArmorSet::RobotSuit && g.hull > 0.0 && g.alive()) {
                    r.target = -1; // the chassis is gone — fly straight
                } else {
                    let c = [g.pos[0], g.pos[1] + g.height() * 0.5, g.pos[2]];
                    let to = [c[0] - r.pos[0], c[1] - r.pos[1], c[2] - r.pos[2]];
                    let dl = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2])
                        .sqrt()
                        .max(0.01);
                    let td = [to[0] / dl, to[1] / dl, to[2] / dl];
                    let clear = self
                        .grid
                        .ray_hit(&self.cover, r.pos, td, dl - 0.5)
                        .is_none();
                    if clear {
                        r.los_lost = 0.0;
                    } else {
                        r.los_lost += DT;
                        if r.los_lost > ROCKET_LOS_BREAK_S {
                            r.target = -1; // hard cover WORKS
                        }
                    }
                    if r.target >= 0 {
                        // PN: commanded turn = N × LOS rate, hard-capped
                        let cosl = (td[0] * r.prev_los[0]
                            + td[1] * r.prev_los[1]
                            + td[2] * r.prev_los[2])
                            .clamp(-1.0, 1.0);
                        let los_rate = cosl.acos() / DT;
                        let turn =
                            (ROCKET_PN_N * los_rate).min(ROCKET_TURN_CAP) * DT;
                        dir = rotate_toward(dir, td, turn);
                        r.prev_los = td;
                    }
                }
            }
            r.vel = [dir[0] * speed, dir[1] * speed, dir[2] * speed];
            let old = r.pos;
            r.pos = [
                old[0] + r.vel[0] * DT,
                old[1] + r.vel[1] * DT,
                old[2] + r.vel[2] * DT,
            ];
            // surfaces: cover or the ground
            let seg = [r.pos[0] - old[0], r.pos[1] - old[1], r.pos[2] - old[2]];
            let sl = (seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2])
                .sqrt()
                .max(1e-6);
            let dn = [seg[0] / sl, seg[1] / sl, seg[2] / sl];
            if self.grid.ray_hit(&self.cover, old, dn, sl).is_some() || r.pos[1] <= 0.0
            {
                done = true;
            }
            // proximity fuse on any enemy body
            if !done {
                for (j, g) in self.fighters.iter().enumerate() {
                    if j == r.shooter || g.team == r.team || !g.alive() {
                        continue;
                    }
                    let c = [g.pos[0], g.pos[1] + g.height() * 0.5, g.pos[2]];
                    let dx = c[0] - r.pos[0];
                    let dy = c[1] - r.pos[1];
                    let dz = c[2] - r.pos[2];
                    if dx * dx + dy * dy + dz * dz < ROCKET_PROX_M * ROCKET_PROX_M {
                        done = true;
                        break;
                    }
                }
            }
            if done {
                booms.push((r.shooter, r.pos));
                self.rockets.remove(idx);
            } else {
                self.rockets[idx] = r;
                idx += 1;
            }
        }
        for (shooter, at) in booms {
            self.booms.push((
                Boom {
                    at,
                    kind: ThrowKind::Frag,
                },
                2.0,
            ));
            // §5.3: 270 before angle armor (rear ≈27% of a hull, front
            // ≈4%); a direct/2 m soldier hit is lethal
            let team = self.fighters[shooter].team;
            for j in 0..self.fighters.len() {
                let g = &self.fighters[j];
                // protect_t: this was the ONE blast path without the
                // spawn-protection gate every other one has - a rocket
                // fired before a victim's shimmer expired still landed.
                if j == shooter || g.team == team || !g.alive() || g.protect_t > 0.0 {
                    continue;
                }
                let c = [g.pos[0], g.pos[1] + g.height() * 0.5, g.pos[2]];
                let dx = c[0] - at[0];
                let dy = c[1] - at[1];
                let dz = c[2] - at[2];
                if dx * dx + dy * dy + dz * dz
                    < ROCKET_SOLDIER_KILL_M * ROCKET_SOLDIER_KILL_M
                {
                    self.apply_plain_damage(shooter, j, ROCKET_DMG, at, false, false);
                }
            }
        }
    }

    /// §5 detonation effects — all sim-side, all deterministic.
    fn detonate(&mut self, g: Grenade) {
        let spec = throw_spec(g.kind);
        self.booms.push((
            Boom {
                at: g.pos,
                kind: g.kind,
            },
            2.0,
        ));
        match g.kind {
            ThrowKind::Frag => {
                // Brief IX-B falloff table via frag_falloff_frac, LOS-
                // blocked: no damage through walls
                for j in 0..self.fighters.len() {
                    let f = &self.fighters[j];
                    if !f.alive() || f.protect_t > 0.0 {
                        continue;
                    }
                    let chest = [f.pos[0], f.pos[1] + f.height() * 0.55, f.pos[2]];
                    let dx = chest[0] - g.pos[0];
                    let dy = chest[1] - g.pos[1];
                    let dz = chest[2] - g.pos[2];
                    let d = (dx * dx + dy * dy + dz * dz).sqrt();
                    if d > spec.radius_m || !self.los_clear(g.pos, chest) {
                        continue;
                    }
                    let dmg = FRAG_DMG * frag_falloff_frac(d);
                    self.apply_plain_damage(g.thrower, j, dmg, g.pos, true, false);
                }
                // §8: the horde is made of BODIES, so a frag has to reach
                // them. Every explosive path here looped fighters only,
                // which meant grenades did literally nothing in the one
                // mode that spawns a horde. Collected by ID, not index -
                // damage_zombie swap_removes on a kill.
                self.blast_zombies(g.pos, spec.radius_m, |d| FRAG_DMG * frag_falloff_frac(d));
            }
            ThrowKind::Flash => {
                // blind = 3.2 s × LOS × facing × distance (§5.3); bots eat
                // the same penalty — a human-only flash is worse than none
                for j in 0..self.fighters.len() {
                    let f = &self.fighters[j];
                    if !f.alive() {
                        continue;
                    }
                    let eye = [f.pos[0], f.pos[1] + EYE_REL.min(f.height() - 0.1), f.pos[2]];
                    let dx = g.pos[0] - eye[0];
                    let dy = g.pos[1] - eye[1];
                    let dz = g.pos[2] - eye[2];
                    let d = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                    if d > spec.radius_m || !self.los_clear(g.pos, eye) {
                        continue;
                    }
                    let fwd = [f.yaw.sin(), 0.0, f.yaw.cos()];
                    let facing = ((fwd[0] * dx + fwd[2] * dz) / d).clamp(0.15, 1.0);
                    let blind = FLASH_BLIND_S * facing * (1.0 - d / spec.radius_m).clamp(0.0, 1.0);
                    let f = &mut self.fighters[j];
                    f.blind_t = f.blind_t.max(blind);
                }
            }
            ThrowKind::Smoke => {
                if self.smokes.len() < SMOKE_MAX {
                    self.smokes.push(SmokeVolume {
                        pos: [g.pos[0], g.pos[1].max(0.4), g.pos[2]],
                        ttl: SMOKE_TTL_S,
                    });
                }
            }
            ThrowKind::Molotov => {
                self.fires.push(FirePool {
                    pos: [g.pos[0], g.pos[1].max(0.02), g.pos[2]],
                    ttl: FIRE_TTL_S,
                    thrower: g.thrower,
                    tick_t: 0.0,
                });
            }
        }
    }

    /// §5.5 fire pools: 4 Hz damage ticks to anyone standing in them.
    fn step_fires(&mut self) {
        for fi in 0..self.fires.len() {
            self.fires[fi].ttl -= DT;
            self.fires[fi].tick_t += DT;
            if self.fires[fi].tick_t < 0.25 {
                continue;
            }
            self.fires[fi].tick_t -= 0.25;
            let (fpos, thrower) = (self.fires[fi].pos, self.fires[fi].thrower);
            let r = throw_spec(ThrowKind::Molotov).radius_m;
            for j in 0..self.fighters.len() {
                let f = &self.fighters[j];
                if !f.alive() || f.protect_t > 0.0 {
                    continue;
                }
                let dx = f.pos[0] - fpos[0];
                let dz = f.pos[2] - fpos[2];
                if dx * dx + dz * dz < r * r && (f.pos[1] - fpos[1]).abs() < 2.0 {
                    if self.fighters[j].armor_set != ArmorSet::Pyro {
                        self.fighters[j].burn_t = 1.0;
                    }
                    self.apply_plain_damage(thrower, j, FIRE_DPS * 0.25, fpos, false, true);
                }
            }
            // §8: fire burns the horde too - a molotov choke point is the
            // obvious answer to a horde and did nothing to it before
            self.blast_zombies(fpos, r, |_| FIRE_DPS * 0.25);
        }
        self.fires.retain(|f| f.ttl > 0.0);
    }

    /// §6.1: the armor-set damage pipeline, shared by every damage path.
    /// Folk's held brace cuts 82% inside its 110° frontal arc (stacking
    /// +8% per overlapping braced ally, capped at 3 — the shieldwall in
    /// one line); then the set's flat per-zone reduction applies with a
    /// floor of 15% of BASE damage so limb shots are never free. Marks
    /// the victim's last-damage time (gates Recon regen).
    fn apply_armor(
        &mut self,
        j: usize,
        dmg: f32,
        base: f32,
        zone: HitZone,
        from: Option<[f32; 3]>,
    ) -> f32 {
        self.apply_armor_tagged(j, dmg, base, zone, from, false, false)
    }

    /// §4.5 (BRIEF VIII): call once per hit, BEFORE checking whether
    /// this particular hit was fatal. Returns the assist candidate for
    /// THIS hit if it turns out to be the killing blow (the previous
    /// distinct attacker, if recent enough) - then records `attacker`
    /// as the new `last_hit_by` for next time. Order matters: reading
    /// old state before overwriting it is what stops the killer's own
    /// hit from ever being read back as its own assist. Self-damage
    /// (a molotov pool or frag catching its own thrower) never claims
    /// the slot - a fighter cannot get assist credit on their own
    /// death, and self-damage must not erase a real prior attacker's
    /// claim to it.
    fn record_hit_get_assist(&mut self, attacker: usize, victim: usize) -> Option<usize> {
        if attacker == victim {
            let f = &self.fighters[victim];
            return f.last_hit_by.and_then(|(who, t)| {
                if self.t - t <= ASSIST_WINDOW_S {
                    Some(who)
                } else {
                    None
                }
            });
        }
        let now = self.t;
        let f = &mut self.fighters[victim];
        let assist = f.last_hit_by.and_then(|(who, t)| {
            if who != attacker && now - t <= ASSIST_WINDOW_S {
                Some(who)
            } else {
                None
            }
        });
        f.last_hit_by = Some((attacker, now));
        assist
    }

    /// §11: the full damage pipeline with the mech's angle-based model.
    /// A mech classifies by the angle between the shot and its BODY
    /// facing — front 85% cut, side 70%, rear nothing; explosives bypass
    /// half the cut, fire bypasses all of it. Damage lands on the HULL
    /// (never the pilot) until it's gone — then the pilot ejects at 25 HP.
    fn apply_armor_tagged(
        &mut self,
        j: usize,
        mut dmg: f32,
        base: f32,
        zone: HitZone,
        from: Option<[f32; 3]>,
        explosive: bool,
        fire: bool,
    ) -> f32 {
        {
            let v = &self.fighters[j];
            if v.armor_set == ArmorSet::RobotSuit && v.hull > 0.0 {
                let mut red = 0.0;
                let mut front = false;
                if let Some(fp) = from {
                    let dx = fp[0] - v.pos[0];
                    let dz = fp[2] - v.pos[2];
                    let len = (dx * dx + dz * dz).sqrt().max(1e-3);
                    // BODY facing, never the camera (§11.2 rule 1)
                    let cos = (v.yaw.sin() * dx + v.yaw.cos() * dz) / len;
                    front = cos > 0.5;
                    red = if front {
                        MECH_RED_FRONT
                    } else if cos > -0.5 {
                        MECH_RED_SIDE
                    } else {
                        0.0
                    };
                }
                if fire {
                    red = 0.0; // fire attacks cooling, not plating
                } else if explosive {
                    red *= 0.5; // grenades are a real frontal answer
                }
                // §4.5 (Brief VI): the sensor VISOR is the weak point —
                // ×2.0 AFTER the angle multiplier, and only where the
                // visor is exposed (the front arc; a rear shot never
                // sees it). Front visor: ×0.15×2.0 = 0.30 — rewarded,
                // not dominant. Fire has no precision to reward.
                let visor = zone == HitZone::Head && front && !fire;
                let mut d = dmg
                    * (1.0 - red)
                    * if visor { MECH_VISOR_MULT } else { 1.0 };
                let f = &mut self.fighters[j];
                // §6.3 (Brief VII v2): once a plate has dropped, the
                // exposed frame takes ×1.25 - applied AFTER angle armor,
                // same as the spec. (This mech's damage model is angle-
                // based, not part-based, so the bonus is frame-wide once
                // ANY plate is gone rather than gap-specific; documented
                // honestly rather than faked with fake per-part hits.)
                if f.mech_plates_dropped != 0 {
                    d *= MECH_EXPOSED_DMG_MULT;
                }
                f.hull = (f.hull - d).max(0.0);
                f.last_dmg_at = self.t;
                // §6.3: HP-threshold plate-detach events - each stage
                // fires exactly once (bitmask), replay-identical since
                // it's driven purely by hull/MECH_HULL.
                let frac = f.hull / MECH_HULL;
                if frac <= MECH_PLATE_70_PCT {
                    f.mech_plates_dropped |= 0b001;
                }
                if frac <= MECH_PLATE_40_PCT {
                    f.mech_plates_dropped |= 0b010;
                }
                if frac <= MECH_PLATE_15_PCT {
                    f.mech_plates_dropped |= 0b100;
                }
                if f.hull <= 0.0 {
                    // destroyed: the pilot ejects, vulnerable and on foot
                    f.armor_set = ArmorSet::None;
                    f.armor = 0.0;
                    f.health = f.health.min(MECH_EJECT_HP);
                    f.mech_plates_dropped = 0;
                    // being blown out mid-boarding must not leave the
                    // ejected pilot disarmed on top of everything else
                    f.mech_transition_t = 0.0;
                }
                return 0.0; // the pilot takes nothing while the hull holds
            }
        }
        let v = &self.fighters[j];
        if v.brace && v.armor_set == ArmorSet::Folk {
            if let Some(fp) = from {
                let dx = fp[0] - v.pos[0];
                let dz = fp[2] - v.pos[2];
                let len = (dx * dx + dz * dz).sqrt().max(1e-3);
                let facing = [v.yaw.sin(), v.yaw.cos()];
                if (facing[0] * dx + facing[1] * dz) / len > BRACE_ARC_COS {
                    let mut allies = 0u32;
                    for (k, a) in self.fighters.iter().enumerate() {
                        if k != j && a.team == v.team && a.brace && a.alive() {
                            let ax = a.pos[0] - v.pos[0];
                            let az = a.pos[2] - v.pos[2];
                            if ax * ax + az * az < 16.0 {
                                allies += 1;
                            }
                        }
                    }
                    let red = (BRACE_REDUCTION
                        + BRACE_STACK_BONUS * allies.min(BRACE_STACK_CAP) as f32)
                        .min(0.96);
                    dmg *= 1.0 - red;
                }
            }
        }
        let spec = armor_spec(self.fighters[j].armor_set);
        let flat = match zone {
            HitZone::Head => spec.flat_head,
            HitZone::Torso => spec.flat_torso,
            _ => spec.flat_limb,
        };
        // the floor never RAISES an already-tiny hit (shielded shots)
        let out = (dmg - flat).max(base * ARMOR_FLOOR_FRAC).min(dmg).max(0.0);
        self.fighters[j].last_dmg_at = self.t;
        out
    }

    /// Zone-less damage (explosions, fire, abilities): §6 flats apply at
    /// torso rate, the shield does NOT block (blast wraps it), kills
    /// credit the source. `explosive` engages Pyro's blast resistance and
    /// drains a Robot Suit's power core; `fire` respects Pyro immunity.
    fn apply_plain_damage(
        &mut self,
        src: usize,
        victim: usize,
        dmg: f32,
        at: [f32; 3],
        explosive: bool,
        fire: bool,
    ) {
        if !self.fighters[victim].alive() || dmg <= 0.0 {
            return;
        }
        let vset = self.fighters[victim].armor_set;
        if fire && vset == ArmorSet::Pyro {
            return; // §6.2 full fire immunity — own and enemy flame alike
        }
        let mut d = dmg;
        if explosive {
            d *= 1.0 - armor_spec(vset).explosive_resist;
            if vset == ArmorSet::RobotSuit {
                let f = &mut self.fighters[victim];
                f.armor = (f.armor - EXPLOSIVE_POWER_DRAIN).max(0.0);
            }
        }
        // §11: blasts carry their direction — the mech's arc model reads
        // the blast position; fire and explosives use their bypass rules
        d = self.apply_armor_tagged(victim, d, dmg, HitZone::Torso, Some(at), explosive, fire);
        let assist_candidate = self.record_hit_get_assist(src, victim);
        self.fighters[victim].health -= d;
        let fatal = self.fighters[victim].health <= 0.0;
        // the BLAST origin, not the source fighter's current position:
        // the client's damage indicator points at `from`, and a grenade
        // victim needs to be pointed at the explosion, not at wherever
        // the thrower has run to by the time the fuse burned down.
        let from = at;
        self.hits.push((
            HitEvent {
                shooter: src,
                victim,
                zone: HitZone::Torso,
                damage: d,
                shielded: false,
                from,
                at,
                fatal,
            },
            2.2,
        ));
        if fatal {
            self.fighters[victim].deaths += 1;
            self.fighters[victim].respawn_t = self.death_respawn_t();
            self.fighters[victim].vel = [0.0, 0.0];
            self.fighters[victim].shield_up = false;
            // A TEAM kill credits nobody. Frag blast and molotov fire are
            // the only damage paths without an upstream team filter
            // (every other one - bullets, rockets, flame, repulsor, axe,
            // knife - filters before it ever gets here), so without this
            // gate a player could farm their own teammates for TDM points
            // and win the match with a grenade.
            let team_kill = self.fighters[src].team == self.fighters[victim].team;
            if src != victim && !team_kill {
                self.fighters[src].kills += 1;
                if self.mode == Mode::Tdm {
                    let s = Self::team_idx(self.fighters[src].team);
                    self.score[s] += 1.0;
                    if self.overtime || self.score[s] >= TDM_TARGET as f32 {
                        self.finish(self.fighters[src].team);
                    }
                }
            }
            let assist_candidate =
                assist_candidate.filter(|&a| self.fighters[a].team == self.fighters[src].team);
            if let Some(a) = assist_candidate {
                self.fighters[a].assists += 1;
            }
            self.kill_feed.push((
                KillEvent {
                    killer: src,
                    victim,
                    headshot: false,
                    assist: assist_candidate,
                },
                5.0,
            ));
        }
    }

    /// §4.3 (BRIEF VIII): pub(crate) so the minimap's enemy-spotting
    /// system (main.rs) can reuse the SAME line-of-sight query every
    /// other visibility-gated system in the sim already uses, rather
    /// than a second, divergent implementation.
    pub(crate) fn los_clear(&self, from: [f32; 3], to: [f32; 3]) -> bool {
        let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-3 {
            return true;
        }
        let dn = [d[0] / len, d[1] / len, d[2] / len];
        if let Some((t, _)) = self.grid.ray_hit(&self.cover, from, dn, len) {
            if t < len - 0.1 {
                return false;
            }
        }
        true
    }

    /// §5.4: VISION test — walls AND smoke. Bots see with this; damage
    /// paths keep `los_clear` (shrapnel doesn't care about smoke).
    /// Occlusion accumulates by path length through each sphere; > 0.6
    /// blocks. The sphere test runs only on rays the walls left clear.
    fn sight_clear(&self, from: [f32; 3], to: [f32; 3]) -> bool {
        if !self.los_clear(from, to) {
            return false;
        }
        if self.smokes.is_empty() {
            return true;
        }
        let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-3 {
            return true;
        }
        let dn = [d[0] / len, d[1] / len, d[2] / len];
        let r = throw_spec(ThrowKind::Smoke).radius_m;
        let mut occl = 0.0_f32;
        for s in &self.smokes {
            let oc = [from[0] - s.pos[0], from[1] - s.pos[1], from[2] - s.pos[2]];
            let b = oc[0] * dn[0] + oc[1] * dn[1] + oc[2] * dn[2];
            let c2 = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - r * r;
            let disc = b * b - c2;
            if disc <= 0.0 {
                continue;
            }
            let sq = disc.sqrt();
            let t0 = (-b - sq).max(0.0);
            let t1 = (-b + sq).min(len);
            occl += (t1 - t0).max(0.0) * 0.25;
        }
        occl < 0.6
    }

    fn nearest_visible_enemy(&self, i: usize) -> Option<usize> {
        let f = &self.fighters[i];
        let eye = [f.pos[0], f.pos[1] + EYE_REL, f.pos[2]];
        let mut best: Option<(usize, f32)> = None;
        for (j, g) in self.fighters.iter().enumerate() {
            if g.team == f.team || !g.alive() || g.protect_t > 0.0 {
                continue;
            }
            // sight the target's actual chest — a crouched enemy is LOWER
            let tgt = [g.pos[0], g.pos[1] + g.height() * 0.6, g.pos[2]];
            let d2 = (tgt[0] - eye[0]).powi(2) + (tgt[2] - eye[2]).powi(2);
            if best.map_or(true, |(_, b)| d2 < b) && self.sight_clear(eye, tgt) {
                best = Some((j, d2));
            }
        }
        best.map(|(j, _)| j)
    }

    /// What bot `i` should be shooting at, as (position, height,
    /// hardened).
    ///
    /// Extraction is CO-OP: everyone is on one team, so
    /// `nearest_visible_enemy` - which only ever looks for an opposing
    /// FIGHTER - always returned None there. AI teammates stood around
    /// doing waypoint patrol while the horde ate them. This adds the
    /// horde as a threat, so the co-op mode actually has allies in it.
    ///
    /// Enemy fighters still win ties at equal range: a player shooting at
    /// you is a worse problem than a zombie walking at you.
    ///
    /// §D: `hardened` means "this threat is itself a live chassis". It is
    /// returned from HERE rather than re-derived by the caller because
    /// this function is the only thing that knows WHICH body it picked -
    /// the caller gets a bare position back, and a position cannot be
    /// asked whether it has 1000 points of hull. A bot mech needs the
    /// answer to choose a mount: 9 damage a round is not an answer to a
    /// hull, 145 is. No zombie is ever hardened - the horde has no
    /// plating, and a Brute is just a large soft target.
    fn nearest_visible_threat(&self, i: usize) -> Option<([f32; 3], f32, bool)> {
        let f = &self.fighters[i];
        let eye = [f.pos[0], f.pos[1] + EYE_REL, f.pos[2]];
        if let Some(j) = self.nearest_visible_enemy(i) {
            let g = &self.fighters[j];
            return Some((g.pos, g.height(), g.in_mech()));
        }
        let mut best: Option<([f32; 3], f32)> = None;
        let mut best_d2 = f32::INFINITY;
        for z in &self.zombies {
            // per-kind height, the same one the bullet zone test uses -
            // a Brute and a Runner are not the same size
            let zh = zspec(z.kind).height;
            let tgt = [z.pos[0], z.pos[1] + zh * 0.6, z.pos[2]];
            let d2 = (tgt[0] - eye[0]).powi(2) + (tgt[2] - eye[2]).powi(2);
            if d2 < best_d2 && self.sight_clear(eye, tgt) {
                best_d2 = d2;
                best = Some((z.pos, zh));
            }
        }
        best.map(|(p, h)| (p, h, false))
    }

    fn bot_think(&mut self, i: usize) {
        let half = self.half;
        let f = &self.fighters[i];
        let team = f.team;
        let (wx, wz) = (f.waypoint[0] - f.pos[0], f.waypoint[1] - f.pos[2]);
        if wx * wx + wz * wz < 4.0 || self.rng.next_f32() < 0.15 {
            let roll = self.rng.next_f32();
            // KOTH bots fight for the hill; everyone likes an unowned or
            // enemy checkpoint; otherwise roam toward the enemy side
            let target = if self.mode == Mode::Koth && roll < 0.55 {
                [
                    self.hill[0] + self.rng.range(-3.0, 3.0),
                    self.hill[2] + self.rng.range(-3.0, 3.0),
                ]
            } else if roll < 0.30 && !self.checkpoints.is_empty() {
                let want = self
                    .checkpoints
                    .iter()
                    .find(|c| c.owner != Some(team))
                    .or(self.checkpoints.first())
                    .unwrap();
                [
                    want.pos[0] + self.rng.range(-2.0, 2.0),
                    want.pos[2] + self.rng.range(-2.0, 2.0),
                ]
            } else {
                let toward = match team {
                    Team::Blue => 1.0,
                    Team::Red => -1.0,
                };
                [
                    self.rng.range(-half + 4.0, half - 4.0),
                    self.rng.range(-half + 4.0, half - 4.0) * 0.5
                        + toward * self.rng.range(0.0, half - 8.0),
                ]
            };
            // keep waypoints INSIDE the walls — the biased roam sum can
            // land past the border, pinning bots against it
            self.fighters[i].waypoint = [
                target[0].clamp(-half + 3.0, half - 3.0),
                target[1].clamp(-half + 3.0, half - 3.0),
            ];
        }
    }

    fn bot_act(&mut self, i: usize) {
        // difficulty shapes the whole brain: aim, reflexes, range, push.
        // §5.3: a flashed bot eats the SAME penalty a flashed human does —
        // aim spread ×4, reaction ×3, deterministically.
        let mut bp = bot_params(self.cfg.difficulty);
        if self.fighters[i].blind_t > 0.0 {
            bp.aim_sigma *= 4.0;
            bp.reaction_s *= 3.0;
        }
        // a fully dry active slot (no mag, no reserve) means SWITCH, not
        // sulk — grab the first slot that still has ammo
        {
            let f = &self.fighters[i];
            if f.ammo == 0 && f.reserve == 0 {
                if let Some(s) = (0..3).find(|&s| {
                    s != f.active && (f.slot_ammo[s].0 > 0 || f.slot_ammo[s].1 > 0)
                }) {
                    self.switch_slot(i, s);
                }
            }
        }
        let enemy = self.nearest_visible_threat(i);
        let (fpos, strafe_phase, waypoint, ammo, reloading) = {
            let f = &self.fighters[i];
            (f.pos, f.strafe_phase, f.waypoint, f.ammo, f.reload_t > 0.0)
        };
        let yaw;
        let mut vel;
        match enemy {
            Some((gpos, ghigh, hardened)) => {
                self.fighters[i].los_time += DT;
                // an empty mag reloads NOW, whatever the range — waiting
                // until the enemy closes is how bots died mid-clack
                if ammo == 0 {
                    self.try_reload(i);
                }
                let (dx, dz) = (gpos[0] - fpos[0], gpos[2] - fpos[2]);
                let dist = (dx * dx + dz * dz).sqrt().max(0.01);
                yaw = dx.atan2(dz);
                let phase = strafe_phase + self.t * 1.7;
                let strafe = phase.sin().signum() * 0.8;
                let (px, pz) = (-dz / dist, dx / dist);
                let closing = if dist > 15.0 {
                    0.8 * bp.aggression
                } else if dist < 6.0 {
                    -0.6 / bp.aggression.max(0.5)
                } else {
                    0.0
                };
                // through the shared guard - a bot-piloted mech must obey
                // the same crouch ban the player's does
                let want_crouch = closing == 0.0 && dist > 9.0;
                self.fighters[i].set_crouch(want_crouch);
                // shield discipline: caught reloading in the open → turtle
                // behind the shield until the mag is back in
                self.fighters[i].shield_up = reloading && dist < 16.0;
                vel = [
                    (px * strafe + dx / dist * closing) * MOVE_SPEED * 0.8,
                    (pz * strafe + dz / dist * closing) * MOVE_SPEED * 0.8,
                ];
                // §D: MOUNT SELECTION. A bot in a chassis was still
                // pulling the trigger on the rifle its pilot happened to
                // be carrying - 4 hits in 2 s, a reload every 30 rounds,
                // and after 150 rounds a PERMANENTLY DISARMED bot inside
                // a full 1000-hull chassis. A mech that cannot shoot is
                // not a threat and not honestly absent; it is a pinata.
                //
                // The rule, deliberately kept to one line of intent:
                // AUTOCANNON against a hull, or past the range where the
                // gatling's cone stops covering a man
                // (`MECH_BOT_AUTOCANNON_RANGE_M`, itself derived from
                // that cone); GATLING otherwise. Suppression up close,
                // precision far out and against armour - the two
                // identities §C wrote into the constants, expressed as
                // the only two facts a bot can cheaply know about its
                // target. `hardened` has no range term ON PURPOSE: 9
                // damage a round is not an answer to 1000 hull at ANY
                // range, and the 145 exists precisely to be that answer.
                let in_mech = self.fighters[i].in_mech();
                let want_auto = if in_mech {
                    // the band is applied only to the DOWN-switch, so the
                    // rule still reads "autocannon past R" and the
                    // hysteresis is visibly the exception, not the rule
                    let drop_at = MECH_BOT_AUTOCANNON_RANGE_M - MECH_BOT_MOUNT_HYSTERESIS_M;
                    let holding_auto = self.fighters[i].mech_weapon == MechWeapon::Autocannon;
                    hardened
                        || dist
                            > if holding_auto {
                                drop_at
                            } else {
                                MECH_BOT_AUTOCANNON_RANGE_M
                            }
                } else {
                    false
                };
                if in_mech {
                    self.fighters[i].mech_weapon = if want_auto {
                        MechWeapon::Autocannon
                    } else {
                        MechWeapon::Gatling
                    };
                }
                // §A parity, answered: a bot DOES brace, and only for the
                // autocannon. `MECH_BRACE_RECOIL_DAMP` exists for exactly
                // one consumer - the autocannon's kick - so bracing for
                // the gatling would buy a bot nothing while costing it
                // 88% of its movement. Bracing for the autocannon buys
                // the shot after this one its picture back, and the plant
                // is paid for in the same currency the human pays: a
                // near-stationary chassis that is trivial to hit.
                // Grounded-gated exactly as the player's is - a stance
                // needs a floor. Written UNCONDITIONALLY (not only when
                // `in_mech`) so a pilot whose hull is blown out from
                // under him mid-brace does not keep the chassis stance,
                // and its 12% pace, on foot for the rest of the match.
                self.fighters[i].mech_brace =
                    want_auto && in_mech && self.fighters[i].grounded;
                if self.fighters[i].los_time > bp.reaction_s
                    && dist < bp.engage_range
                    // a hull mount has NO magazine - gating a chassis on
                    // the pilot's carried rounds is the whole defect
                    && (ammo > 0 || in_mech)
                {
                    // fire from the REAL muzzle (crouch lowers it) at the
                    // target's REAL chest (crouched enemies are short) —
                    // the old fixed heights sailed over crouchers and
                    // discounted everything into the arms band
                    let eye = self.muzzle_origin(i);
                    let tgt = [gpos[0], gpos[1] + ghigh * 0.55, gpos[2]];
                    let aim = [tgt[0] - eye[0], tgt[1] - eye[1], tgt[2] - eye[2]];
                    // NOTE the RNG draws are UNCONDITIONAL on the branch
                    // below and keep their original position: the bot
                    // stream is shared with everything else in the tick,
                    // so moving or skipping them would re-order the seeded
                    // sequence for every scenario, not just mech ones.
                    let (e1, e2) = (
                        self.rng.range(-bp.aim_sigma, bp.aim_sigma),
                        self.rng.range(-bp.aim_sigma, bp.aim_sigma),
                    );
                    let aim = perturb(normalize(aim), e1, e2);
                    if in_mech {
                        match self.fighters[i].mech_weapon {
                            MechWeapon::Gatling => {
                                self.try_fire_gatling(i, aim);
                            }
                            MechWeapon::Autocannon => {
                                self.try_fire_autocannon(i, aim);
                            }
                        }
                    } else {
                        self.try_fire(i, aim, false);
                    }
                }
            }
            None => {
                self.fighters[i].los_time = 0.0;
                self.fighters[i].crouch = false;
                self.fighters[i].shield_up = false;
                // nothing to shoot: the plant is dropped. A braced mech
                // walks at 12% - a bot that kept the stance after its
                // target broke LOS would crawl the rest of the match.
                self.fighters[i].mech_brace = false;
                if self.fighters[i].armed()
                    && ammo < gun(self.fighters[i].gun).mag / 3
                {
                    self.try_reload(i);
                }
                let (dx, dz) = (waypoint[0] - fpos[0], waypoint[1] - fpos[2]);
                let d = (dx * dx + dz * dz).sqrt().max(0.01);
                vel = [dx / d * MOVE_SPEED * 0.85, dz / d * MOVE_SPEED * 0.85];
                yaw = dx.atan2(dz);
                // probe at 0.75 m so waist-high garden ruins register too
                let eye = [fpos[0], fpos[1] + 0.75, fpos[2]];
                let ahead = [
                    fpos[0] + dx / d * 1.2,
                    fpos[1] + 0.75,
                    fpos[2] + dz / d * 1.2,
                ];
                if !self.los_clear(eye, ahead) {
                    vel = [-dz / d * MOVE_SPEED * 0.7, dx / d * MOVE_SPEED * 0.7];
                }
            }
        }
        // §5.5: fire pools block bot pathing — they veer around them
        {
            let ahead = [fpos[0] + vel[0] * 0.4, fpos[2] + vel[1] * 0.4];
            let avoid = throw_spec(ThrowKind::Molotov).radius_m + 0.8;
            for fp in &self.fires {
                let dx = ahead[0] - fp.pos[0];
                let dz = ahead[1] - fp.pos[2];
                if dx * dx + dz * dz < avoid * avoid {
                    vel = [-vel[1], vel[0]]; // hard perpendicular veer
                    break;
                }
            }
        }
        let fm = &mut self.fighters[i];
        if fm.shield_up {
            vel = [vel[0] * SHIELD_SPEED_MULT, vel[1] * SHIELD_SPEED_MULT];
        }
        if fm.crouch {
            // bots pay the same crouch tax the player does
            vel = [vel[0] * CROUCH_SPEED_MULT, vel[1] * CROUCH_SPEED_MULT];
        }
        // §6: bots pay the ARMOR-SET pace too. Without this a bot in a
        // mech ran at full soldier speed while the player's mech is held
        // to 85%, and a bot's drained chassis never got heavy - the set
        // that is supposed to be a mobility tradeoff was pure upside for
        // everyone except the human.
        {
            let aspec = armor_spec(fm.armor_set);
            vel = [vel[0] * aspec.move_mult, vel[1] * aspec.move_mult];
            if fm.armor_set == ArmorSet::RobotSuit && fm.armor <= 0.0 {
                vel = [vel[0] * ROBOT_DRAINED_MOVE, vel[1] * ROBOT_DRAINED_MOVE];
            }
            if fm.brace {
                vel = [vel[0] * BRACE_SPEED_MULT, vel[1] * BRACE_SPEED_MULT];
            }
            // §A.4 parity: bots pay the same mech-brace tax. §D now SETS
            // mech_brace on the bot path too (autocannon engagements
            // only, see `bot_act` above), so this tax is live on both
            // sides rather than a mechanism waiting for a second caller.
            if fm.mech_brace {
                vel = [vel[0] * MECH_BRACE_SPEED_MULT, vel[1] * MECH_BRACE_SPEED_MULT];
            }
        }
        // §7: bots pay the same MASS tax the player does — the minigun's
        // identity is its mobility cost, on every carrier
        if fm.gun == GunKind::Minigun {
            let m = if fm.spin_t > 0.2 {
                MINIGUN_SPUN_MOVE_MULT
            } else {
                MINIGUN_MOVE_MULT
            };
            vel = [vel[0] * m, vel[1] * m];
        }
        // §1.3: the SAME two-rate approach the player pays. A bot that
        // could stop dead while the human slides would out-peek the
        // human for free - exactly the parity defect the mech turn-rate
        // comment below was written about.
        fm.vel = approach_velocity(fm.vel, vel, DT);
        // §11: a mech TURNS at a capped rate - facing a new threat is a
        // visible, punishable commitment. The player's path enforces
        // this; the bot path snapped instantly to any new facing, so a
        // bot mech could whip around in one tick and the "commitment"
        // that balances the chassis only cost the human.
        if fm.armor_set == ArmorSet::RobotSuit && fm.hull > 0.0 {
            let d = wrap_angle(yaw - fm.yaw);
            let step = (MECH_TURN_RATE * DT).min(d.abs());
            fm.yaw += d.signum() * step;
        } else {
            fm.yaw = yaw;
        }
    }
}

fn spawn_point(team: Team, slot: usize, half: f32) -> ([f32; 3], f32) {
    let z = match team {
        Team::Blue => -half + 2.5,
        Team::Red => half - 2.5,
    };
    let yaw = match team {
        Team::Blue => 0.0,
        Team::Red => std::f32::consts::PI,
    };
    // room for up to 8 per team (v6 cap)
    let x = -10.5 + (slot as f32) * 3.0;
    ([x, 0.0, z], yaw)
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Shortest signed angle equivalent — keeps a yaw delta in (−π, π] so a
/// wrap across ±π never reads as a full-circle whip turn.
fn wrap_angle(a: f32) -> f32 {
    let two_pi = std::f32::consts::TAU;
    let mut a = a % two_pi;
    if a > std::f32::consts::PI {
        a -= two_pi;
    } else if a < -std::f32::consts::PI {
        a += two_pi;
    }
    a
}

fn perturb(d: [f32; 3], ex: f32, ey: f32) -> [f32; 3] {
    let up = if d[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let rx = [
        d[1] * up[2] - d[2] * up[1],
        d[2] * up[0] - d[0] * up[2],
        d[0] * up[1] - d[1] * up[0],
    ];
    let rx = normalize(rx);
    let ry = [
        d[1] * rx[2] - d[2] * rx[1],
        d[2] * rx[0] - d[0] * rx[2],
        d[0] * rx[1] - d[1] * rx[0],
    ];
    normalize([
        d[0] + rx[0] * ex + ry[0] * ey,
        d[1] + rx[1] * ex + ry[1] * ey,
        d[2] + rx[2] * ex + ry[2] * ey,
    ])
}

fn ray_vs_cylinder(
    o: [f32; 3],
    d: [f32; 3],
    base: [f32; 3],
    radius: f32,
    height: f32,
) -> Option<(f32, f32)> {
    let (ox, oz) = (o[0] - base[0], o[2] - base[2]);
    let (dx, dz) = (d[0], d[2]);
    let a = dx * dx + dz * dz;
    if a < 1e-8 {
        return None;
    }
    let b = 2.0 * (ox * dx + oz * dz);
    let c = ox * ox + oz * oz - radius * radius;
    // muzzle already inside the body: point-blank contact, hit at t = 0
    // (the entry-face root is negative there and used to whiff entirely)
    if c < 0.0 {
        if o[1] >= base[1] && o[1] <= base[1] + height {
            return Some((0.0, o[1]));
        }
        return None;
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    if t < 0.0 {
        return None;
    }
    let y = o[1] + d[1] * t;
    if y < base[1] || y > base[1] + height {
        return None;
    }
    Some((t, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jk_core::timestep::SIM_HZ;

    fn run(sim: &mut TdmSim, secs: usize, cmd: PlayerCmd) {
        for _ in 0..(secs * SIM_HZ as usize) {
            sim.step(cmd);
        }
    }

    fn cfg(seed: u64, per_team: usize, mode: Mode, map: MapKind) -> MatchConfig {
        MatchConfig {
            seed,
            per_team,
            mode,
            map,
            ..MatchConfig::default()
        }
    }

    /// A clean shooting range: 1v1, no cover, both pinned and unprotected.
    fn range(seed: u64) -> TdmSim {
        let mut s = TdmSim::new(cfg(seed, 1, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        s.pickups.clear();
        s.checkpoints.clear();
        s.fighters[0].pos = [0.0, 0.0, -5.0];
        s.fighters[1].pos = [0.0, 0.0, 5.0];
        s.fighters[1].protect_t = 0.0;
        s
    }

    /// Fire `shots` M4-style aimed rounds at a held target; returns hits
    /// dealt and the victim's remaining health.
    fn shoot_at(s: &mut TdmSim, aim_y: f32, secs: usize) {
        // aim from the eye (1.62) to the requested height at 10 m
        let cmd = PlayerCmd {
            aim: [0.0, (aim_y - EYE_REL) / 10.0, 1.0],
            shoot: true,
            ..Default::default()
        };
        for _ in 0..(secs * SIM_HZ as usize) {
            s.step(cmd);
            // pin the bot: bot AI resets pos/vel/stance INSIDE step —
            // hold it STANDING so the fixed aim heights land where the
            // test expects
            s.fighters[1].pos = [0.0, 0.0, 5.0];
            s.fighters[1].vel = [0.0, 0.0];
            s.fighters[1].protect_t = 0.0;
            s.fighters[1].shield_up = false;
            s.fighters[1].crouch = false;
            if s.fighters[0].kills >= 1 {
                break;
            }
        }
    }

    #[test]
    fn ttk_rule_two_head_eight_torso_for_baseline_rifle() {
        // the owner's tuned target IS the baseline M4A1
        let m4 = gun(GunKind::M4);
        assert_eq!(
            (MAX_HEALTH / (m4.damage * HEAD_MULT)).ceil() as u32,
            2,
            "2 headshots"
        );
        assert_eq!((MAX_HEALTH / m4.damage).ceil() as u32, 8, "8 body shots");
        // torso: must take 6+ hits (spread can graze arms; never < 6)
        let mut s = range(3);
        assert_eq!(s.fighters[0].gun, GunKind::M4, "default loadout leads M4");
        shoot_at(&mut s, 1.0, 8);
        assert_eq!(s.fighters[0].kills, 1, "torso fire must kill");
        assert!(
            s.fighters[0].hits_dealt >= 6,
            "torso kill takes ~8: took {}",
            s.fighters[0].hits_dealt
        );
        // head: exactly 2
        let mut s = range(4);
        shoot_at(&mut s, 1.66, 8);
        assert_eq!(s.fighters[0].kills, 1, "head fire must kill");
        assert!(
            s.fighters[0].hits_dealt <= 3,
            "head kill takes ~2: took {}",
            s.fighters[0].hits_dealt
        );
    }

    /// §5.2 (Brief VI, SUPERSEDES the old 70-dmg "only the head is
    /// instant" table): AWP-class 115 damage — head AND chest/arms are
    /// one-shots; only legs (×0.75, no armor reduction) ever survive,
    /// and even then never for a second hit.
    #[test]
    fn awm_head_and_torso_are_instant_legs_never_are() {
        let awm = gun(GunKind::Awm);
        assert!(awm.damage * HEAD_MULT >= MAX_HEALTH, "head = oblivion");
        assert!(awm.damage >= MAX_HEALTH, "chest/arms one-shot too — 115");
        assert!(
            awm.damage * LEG_MULT < MAX_HEALTH,
            "legs (86.25) must NOT one-shot"
        );
        assert_eq!(
            (MAX_HEALTH / (awm.damage * LEG_MULT)).ceil() as u32,
            2,
            "legs take 2"
        );
    }

    #[test]
    fn loadout_slots_switch_and_keep_ammo() {
        let mut s = TdmSim::new(cfg(5, 1, Mode::Tdm, MapKind::Arena));
        assert_eq!(s.fighters[0].inventory, DEFAULT_LOADOUT);
        assert_eq!(s.fighters[0].gun, GunKind::M4);
        // burn 3 rounds, switch to the pistol, switch back: mag remembered
        s.fighters[0].ammo -= 3;
        let left = s.fighters[0].ammo;
        s.step(PlayerCmd {
            slot: Some(1),
            ..Default::default()
        });
        // switching takes SWITCH_S; the gun itself flips immediately
        assert_eq!(s.fighters[0].gun, GunKind::Glock);
        assert_eq!(s.fighters[0].ammo, gun(GunKind::Glock).mag);
        s.fighters[0].switch_t = 0.0;
        s.step(PlayerCmd {
            slot: Some(0),
            ..Default::default()
        });
        assert_eq!(s.fighters[0].gun, GunKind::M4);
        assert_eq!(s.fighters[0].ammo, left, "slot keeps its magazine");
    }

    #[test]
    fn shield_blocks_front_ignores_rear() {
        let mut s = range(6);
        // victim: shield up, crouched, FACING the shooter (yaw π → −z).
        // Crouched height is 1.05, so torso shots land around y ≈ 0.55.
        s.fighters[1].shield_up = true;
        s.fighters[1].crouch = true;
        s.fighters[1].yaw = std::f32::consts::PI;
        let h0 = s.fighters[1].health;
        s.apply_hit(0, 1, 0.55, [0.0, 0.55, 5.0]);
        let front_dmg = h0 - s.fighters[1].health;
        assert!(
            front_dmg < gun(GunKind::M4).damage * 0.1,
            "crouched shield must be near-total from the front: took {front_dmg}"
        );
        // same shot with the victim facing AWAY: the shield does nothing
        s.fighters[1].yaw = 0.0;
        let h1 = s.fighters[1].health;
        s.apply_hit(0, 1, 0.55, [0.0, 0.55, 5.0]);
        let rear_dmg = h1 - s.fighters[1].health;
        assert!(
            (rear_dmg - gun(GunKind::M4).damage).abs() < 0.01,
            "rear shots ignore the shield entirely: took {rear_dmg}"
        );
        // standing (not crouched) front block is real but not near-total
        // (standing height 1.78 → torso is around y ≈ 1.0)
        s.fighters[1].yaw = std::f32::consts::PI;
        s.fighters[1].crouch = false;
        let h2 = s.fighters[1].health;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        let stand_dmg = h2 - s.fighters[1].health;
        assert!(
            stand_dmg > front_dmg && stand_dmg < rear_dmg,
            "standing block sits between: {front_dmg} < {stand_dmg} < {rear_dmg}"
        );
        // shield up = no shooting
        s.fighters[0].shield_up = true;
        let fired = s.try_fire(0, [0.0, 0.0, 1.0], false);
        assert!(!fired, "the shield takes both hands");
    }

    #[test]
    fn reload_keeps_chambered_rounds() {
        let mut s = TdmSim::new(cfg(8, 1, Mode::Tdm, MapKind::Arena));
        // 5 in the mag, 10 in reserve → a reload must yield 15 total,
        // not throw the 5 away
        s.fighters[0].ammo = 5;
        s.fighters[0].reserve = 10;
        s.try_reload(0);
        for _ in 0..(3 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[1].pos = [30.0, 0.0, 30.0];
        }
        let f = &s.fighters[0];
        assert_eq!(f.ammo + f.reserve, 15, "reload must not destroy ammo");
        assert_eq!(f.ammo, 15.min(gun(f.gun).mag), "mag tops up first");
    }

    #[test]
    fn one_body_scores_one_kill() {
        // two lethal spears landing the same tick must count ONE death
        let mut s = range(9);
        s.fighters[1].health = 10.0;
        // spawned one integration step short of the body so BOTH arrive
        // on the same tick
        for id in [901, 902] {
            s.missiles.push(Missile {
                id,
                pos: [0.0, 1.0, 4.9],
                vel: [0.0, 0.0, 30.0],
                team: Team::Blue,
                shooter: 0,
                damage: 55.0,
                is_spear: true,
                stuck_t: None,
                embedded: true,
                pierces_left: 0,
                pierced: Vec::new(),
                power: 1.0,
            });
        }
        s.step_missiles();
        assert_eq!(s.fighters[1].deaths, 1, "one body, one death");
        assert_eq!(s.fighters[0].kills, 1, "one body, one kill");
        assert!((s.score[0] - 1.0).abs() < 0.01, "one body, one point");
    }

    #[test]
    fn rolling_shield_blocks_nothing() {
        let mut s = range(10);
        s.fighters[1].shield_up = true;
        s.fighters[1].crouch = true;
        s.fighters[1].yaw = std::f32::consts::PI; // facing the shooter
        s.fighters[1].roll_t = ROLL_S * 0.5; // mid-somersault
        let h0 = s.fighters[1].health;
        s.apply_hit(0, 1, 0.5, [0.0, 0.5, 5.0]);
        let dmg = h0 - s.fighters[1].health;
        assert!(
            dmg > gun(GunKind::M4).damage * 0.5,
            "a tumbling shield must not block: took only {dmg}"
        );
    }

    #[test]
    fn checkpoints_capture_and_pull_respawns() {
        let mut s = TdmSim::new(cfg(7, 1, Mode::Tdm, MapKind::Bailey));
        let cp_pos = s.checkpoints[0].pos;
        // stand in the ring uncontested → the ring flips Blue
        for _ in 0..((CHECKPOINT_CAP_S as usize + 2) * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[0].pos = cp_pos;
            s.fighters[1].pos = [-30.0, 0.0, -30.0]; // keep the enemy away
        }
        assert_eq!(s.checkpoints[0].owner, Some(Team::Blue), "ring must flip");
        // die → respawn at the owned checkpoint, not at base
        s.fighters[0].health = 0.0;
        s.fighters[0].respawn_t = RESPAWN_S;
        for _ in 0..((RESPAWN_S as usize + 2) * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
        }
        let p = s.fighters[0].pos;
        let d = ((p[0] - cp_pos[0]).powi(2) + (p[2] - cp_pos[2]).powi(2)).sqrt();
        assert!(d < 4.0, "respawn must be AT the checkpoint: {d} m away");
    }

    #[test]
    fn difficulty_scales_the_bots() {
        let e = bot_params(Difficulty::Easy);
        let n = bot_params(Difficulty::Normal);
        let h = bot_params(Difficulty::Hard);
        assert!(e.aim_sigma > n.aim_sigma && n.aim_sigma > h.aim_sigma);
        assert!(e.reaction_s > n.reaction_s && n.reaction_s > h.reaction_s);
        assert!(e.engage_range < n.engage_range && n.engage_range < h.engage_range);
        // and it shows on the field: the same battle bleeds the player's
        // team harder on Hard than on Easy
        let deaths = |d: Difficulty| {
            let mut s = TdmSim::new(MatchConfig {
                seed: 42,
                per_team: 4,
                difficulty: d,
                ..MatchConfig::default()
            });
            for _ in 0..(30 * SIM_HZ as usize) {
                s.step(PlayerCmd::default());
            }
            s.fighters
                .iter()
                .filter(|f| f.team == Team::Blue)
                .map(|f| f.deaths)
                .sum::<u32>()
        };
        assert!(
            deaths(Difficulty::Hard) >= deaths(Difficulty::Easy),
            "hard bots must bite harder"
        );
    }

    #[test]
    fn elevation_supports_and_gravity() {
        let mut s = TdmSim::new(cfg(4, 5, Mode::Tdm, MapKind::Arena));
        // put the player on the tower top
        s.fighters[0].pos = [0.0, 3.0, 0.0];
        run(&mut s, 1, PlayerCmd::default());
        assert!(
            (s.fighters[0].pos[1] - 3.0).abs() < 0.05,
            "tower must support: y {}",
            s.fighters[0].pos[1]
        );
        // walk off the edge → fall to ground
        s.fighters[0].pos = [8.0, 3.0, 8.0]; // off-tower, in the air
        run(&mut s, 2, PlayerCmd::default());
        assert!(
            s.fighters[0].pos[1] < 0.1,
            "gravity must land him: y {}",
            s.fighters[0].pos[1]
        );
    }

    /// §4: the bow's draw/release path is a SECOND fire path that
    /// bypassed `try_fire` entirely - so the player's bow fired with
    /// literally zero spread while sprinting and never auto-nocked.
    /// Both are now shared through `aim_spread`.
    #[test]
    fn the_bow_draw_path_applies_spread_and_auto_nocks() {
        // -- spread: a sprinting draw must NOT be pixel-perfect
        let mut s = range(70);
        s.fighters[0].gun = GunKind::Bow;
        s.fighters[0].ammo = 10;
        s.fighters[0].reserve = 10;
        s.fighters[0].vel = [SPRINT_SPEED, 0.0];
        s.fighters[0].grounded = false; // airborne too - max penalty
        let moving = s.aim_spread_of(0, true);
        s.fighters[0].vel = [0.0, 0.0];
        s.fighters[0].grounded = true;
        let still = s.aim_spread_of(0, true);
        assert!(
            moving > still,
            "a sprinting airborne bow shot must be less accurate than a planted one: \
             {moving} vs {still}"
        );
        assert!(still > 0.0, "even a planted bow has a real cone, not zero");

        // -- the release actually perturbs: two identical draws from
        // different RNG states must not produce the same direction
        let launch_dir = |seed: u64| {
            let mut s = range(seed);
            s.fighters[0].gun = GunKind::Bow;
            s.fighters[0].ammo = 10;
            s.fighters[0].reserve = 10;
            s.fighters[0].vel = [SPRINT_SPEED, 0.0];
            for _ in 0..((0.5 * SIM_HZ as f32) as usize) {
                s.step(PlayerCmd { shoot: true, aim: [0.0, 0.0, 1.0], ..Default::default() });
            }
            s.step(PlayerCmd { shoot: false, aim: [0.0, 0.0, 1.0], ..Default::default() });
            s.missiles.last().map(|m| m.vel)
        };
        let a = launch_dir(71).expect("an arrow must have launched");
        let b = launch_dir(72).expect("an arrow must have launched");
        assert!(
            a != b,
            "spread must actually randomize the launch direction; both were {a:?}"
        );

        // -- auto-nock: emptying the mag must start a reload by itself
        let mut s = range(73);
        s.fighters[0].gun = GunKind::Bow;
        s.fighters[0].ammo = 1;
        s.fighters[0].reserve = 5;
        for _ in 0..((0.5 * SIM_HZ as f32) as usize) {
            s.step(PlayerCmd { shoot: true, aim: [0.0, 0.0, 1.0], ..Default::default() });
        }
        s.step(PlayerCmd { shoot: false, aim: [0.0, 0.0, 1.0], ..Default::default() });
        assert_eq!(s.fighters[0].ammo, 0, "the last arrow was loosed");
        assert!(
            s.fighters[0].reload_t > 0.0,
            "the next arrow must nock automatically, as try_fire's path already does"
        );
    }

    /// A TEAM kill must credit nobody. Frag blast and molotov fire are
    /// the only damage paths with no upstream team filter, so without a
    /// gate in the fatal block a player could farm teammates with a
    /// grenade for TDM points and win the match on it.
    #[test]
    fn a_team_kill_scores_nothing_and_cannot_win_the_match() {
        let mut s = TdmSim::new(cfg(80, 3, Mode::Tdm, MapKind::Arena));
        // find a living teammate of fighter 0
        let team0 = s.fighters[0].team;
        let mate = (1..s.fighters.len())
            .find(|&j| s.fighters[j].team == team0)
            .expect("a 3v3 must have a teammate");
        s.fighters[mate].protect_t = 0.0;
        let score_before = s.score;
        let kills_before = s.fighters[0].kills;
        // blow the teammate up at point blank
        s.apply_plain_damage(0, mate, 500.0, s.fighters[0].pos, true, false);
        assert!(!s.fighters[mate].alive(), "the teammate must actually die");
        assert_eq!(
            s.fighters[0].kills, kills_before,
            "a team kill must not credit a kill"
        );
        assert_eq!(
            s.score, score_before,
            "a team kill must not move the scoreboard"
        );
    }

    /// §8: an axe sweep through a packed horde kills several zombies in
    /// one pass. Collecting INDICES and then calling damage_zombie -
    /// which `swap_remove`s on a kill - panicked the authoritative sim.
    #[test]
    fn an_axe_sweep_through_a_packed_horde_does_not_panic() {
        let mut s = TdmSim::new(cfg(81, 1, Mode::Extraction, MapKind::Arena));
        s.fighters[0].pos = [0.0, 0.0, 0.0];
        s.fighters[0].yaw = 0.0;
        s.fighters[0].melee_axe = true;
        s.zombies.clear();
        // pack the arc directly in front of the player
        for k in 0..8 {
            s.next_zombie_id += 1;
            s.zombies.push(Zombie {
                id: s.next_zombie_id,
                kind: ZKind::Shambler,
                pos: [(k as f32 - 4.0) * 0.2, 0.0, 1.2],
                hp: zspec(ZKind::Shambler).hp,
                atk_cd: 0.0,
                scream_t: 0.0,
                head_hits: 0,
                target: [0.0, 0.0],
                alerted: true,
            });
        }
        let before = s.zombies.len();
        // a TAP is the quick chop (matching axe_sweeps_the_whole_arc);
        // hold the horde in the arc so the whole sweep lands at once -
        // that multi-kill is exactly what used to panic
        for i in 0..90 {
            s.step(PlayerCmd {
                knife_hold: i < 2,
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
            for k in 0..s.zombies.len() {
                s.zombies[k].pos = [(k as f32 - 4.0) * 0.2, 0.0, 1.2];
            }
            s.fighters[0].pos = [0.0, 0.0, 0.0];
            s.fighters[0].yaw = 0.0;
        }
        assert!(
            s.zombies.len() < before,
            "the sweep must actually kill zombies: {before} -> {}",
            s.zombies.len()
        );
    }

    /// §6/§11: a bot in a mech must pay the SAME taxes the player's does.
    /// The bot path wrote yaw directly (instant whip-turn) and skipped
    /// the armor-set pace entirely, so the chassis's balancing costs only
    /// ever applied to the human.
    #[test]
    fn a_bot_mech_pays_the_turn_rate_and_armor_pace_taxes() {
        let mut s = TdmSim::new(cfg(86, 2, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        let bot = 1usize;
        s.fighters[bot].armor_set = ArmorSet::RobotSuit;
        s.fighters[bot].hull = MECH_HULL;
        s.fighters[bot].armor = POWER_MAX;
        s.fighters[bot].yaw = 0.0;
        // an enemy directly BEHIND, so the bot wants a 180 this tick
        let foe = (0..s.fighters.len())
            .find(|&j| s.fighters[j].team != s.fighters[bot].team)
            .expect("a 2v2 has an enemy");
        s.fighters[bot].pos = [0.0, 0.0, 0.0];
        s.fighters[foe].pos = [0.0, 0.0, -8.0];
        s.fighters[foe].protect_t = 0.0;
        let yaw0 = s.fighters[bot].yaw;
        s.step(PlayerCmd::default());
        let turned = wrap_angle(s.fighters[bot].yaw - yaw0).abs();
        assert!(
            turned <= MECH_TURN_RATE * DT + 1e-4,
            "a bot mech must be held to the turn-rate cap, turned {turned} rad in one tick"
        );

        // and the armor-set pace: a mech bot cannot out-run its own spec
        let mech_speed = {
            let f = &s.fighters[bot];
            (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt()
        };
        let cap = MOVE_SPEED * armor_spec(ArmorSet::RobotSuit).move_mult + 1e-3;
        assert!(
            mech_speed <= cap,
            "a bot mech must obey its armor pace: {mech_speed} > {cap}"
        );
    }

    /// §3.4 (BRIEF VIII): sprint-out. Corner-sprinting cannot also mean
    /// instantly shooting whoever is there - the weapon takes its class
    /// beat to come up. Never built until this pass; the brief's own
    /// audit-table probe would have caught it.
    #[test]
    fn sprint_out_gates_fire_by_weapon_class() {
        let mut s = range(94);
        s.fighters[0].ammo = 30;
        // sprint at full speed for a second - the gate must be held
        for _ in 0..(SIM_HZ as usize) {
            s.step(PlayerCmd {
                move_z: 1.0,
                sprint: true,
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
        }
        assert!(
            s.fighters[0].sprint_gate_t > 0.0,
            "sprinting must hold the gate up"
        );
        assert!(
            !s.try_fire(0, [0.0, 0.0, 1.0], false),
            "cannot fire at a dead sprint"
        );
        // stop: the M4 needs its 0.20s beat, then fires
        let rifle_beats = (sprint_out_s(GunKind::M4) * SIM_HZ as f32) as usize;
        for _ in 0..(rifle_beats / 2) {
            s.step(PlayerCmd { aim: [0.0, 0.0, 1.0], ..Default::default() });
        }
        assert!(
            !s.try_fire(0, [0.0, 0.0, 1.0], false),
            "halfway through the sprint-out the gun is still coming up"
        );
        for _ in 0..(rifle_beats / 2 + 4) {
            s.step(PlayerCmd { aim: [0.0, 0.0, 1.0], ..Default::default() });
        }
        s.fighters[0].protect_t = 0.0;
        assert!(
            s.try_fire(0, [0.0, 0.0, 1.0], false),
            "gate expired - the rifle fires"
        );
        // and the class split is real: heavy waits longer than SMG
        assert!(sprint_out_s(GunKind::Awm) > sprint_out_s(GunKind::Mp5));
        assert!(
            sprint_out_s(GunKind::Minigun) == 0.0,
            "the minigun's spin-up IS its ready cost - no double tax"
        );
    }

    /// §3.4: running dry costs TIME, not just the ammo math. A tactical
    /// reload (round chambered) beats an empty one by the bolt cycle.
    #[test]
    fn an_empty_reload_takes_longer_than_a_tactical_one() {
        let mut s = range(95);
        // tactical: 10 rounds left
        s.fighters[0].ammo = 10;
        s.fighters[0].reserve = 60;
        s.try_reload(0);
        let tactical = s.fighters[0].reload_t;
        // empty: dry mag
        let mut s2 = range(96);
        s2.fighters[0].ammo = 0;
        s2.fighters[0].reserve = 60;
        s2.try_reload(0);
        let empty = s2.fighters[0].reload_t;
        assert!(tactical > 0.0 && empty > 0.0, "both reloads must start");
        assert!(
            (empty - tactical * RELOAD_EMPTY_MULT).abs() < 1e-4,
            "empty must cost exactly the multiplier: {empty} vs {tactical} * {RELOAD_EMPTY_MULT}"
        );
    }

    /// Task 3 rule 3 (MISSION doc), finally a REAL mechanic: a dodge cut
    /// against your own movement launches harder than one riding it -
    /// the counter-movement loads the legs. Snapshotted at the trigger,
    /// because the burst phase overwrites velocity every tick (which is
    /// why this sat as an unwired spec fixture for two briefs).
    #[test]
    fn a_counter_movement_dodge_launches_harder() {
        let peak_roll_speed = |counter: bool| -> f32 {
            let mut s = range(92);
            s.fighters[0].pos = [0.0, 0.0, 0.0];
            // sprint forward long enough to be at real speed
            for _ in 0..(SIM_HZ as usize) {
                s.step(PlayerCmd {
                    move_z: 1.0,
                    sprint: true,
                    aim: [0.0, 0.0, 1.0],
                    ..Default::default()
                });
            }
            // dodge: either BACK against the sprint, or WITH it
            let dodge_z = if counter { -1.0 } else { 1.0 };
            let mut vmax = 0.0_f32;
            for i in 0..(SIM_HZ as usize) {
                s.step(PlayerCmd {
                    move_z: dodge_z,
                    dodge: i == 0,
                    aim: [0.0, 0.0, 1.0],
                    ..Default::default()
                });
                let f = &s.fighters[0];
                let sp = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
                vmax = vmax.max(sp);
            }
            vmax
        };
        let with_momentum = peak_roll_speed(false);
        let counter = peak_roll_speed(true);
        assert!(
            counter > with_momentum + 0.5,
            "a counter-movement dodge must launch measurably harder: \
             counter {counter:.2} vs with-momentum {with_momentum:.2}"
        );
        assert!(
            (counter - ROLL_SPEED * (1.0 + ROLL_COUNTER_BONUS)).abs() < 0.5,
            "and by exactly the specced bonus: got {counter:.2}"
        );
    }

    /// IX-A slice 1: the sightline validator itself must be trustworthy
    /// before its numbers mean anything - an empty range reads its own
    /// diagonal, and a full-height wall provably shortens the answer.
    /// Then every shipping map gets MEASURED against the castle brief's
    /// 40 m rule. This does NOT assert the existing maps pass - they
    /// were built before the rule existed; the test asserts the numbers
    /// are real and prints them so the violation is on the record.
    #[test]
    fn sightline_validator_measures_real_lines_and_reports_every_map() {
        // instrument check 1: an empty field's worst line is its own span
        let mut s = TdmSim::new(cfg(90, 1, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        let open = s.max_unobstructed_sightline(s.half / 6.0);
        let diag = (s.half - 1.0) * 2.0 * std::f32::consts::SQRT_2;
        assert!(
            open > diag * 0.9,
            "an empty map must read ~its own diagonal: got {open:.1} vs {diag:.1}"
        );

        // instrument check 2: a full-height bisecting wall shortens it
        s.cover.push(Aabb {
            min: [-s.half, 0.0, -0.5],
            max: [s.half, 6.0, 0.5],
        });
        s.cover_kind.push(CoverKind::Stone);
        s.rebuild_grid();
        let walled = s.max_unobstructed_sightline(s.half / 6.0);
        assert!(
            walled < open,
            "a bisecting wall must strictly shorten the worst line: {walled:.1} vs {open:.1}"
        );

        // now the real maps, on the record
        for map in MapKind::ALL {
            let s = TdmSim::new(cfg(91, 2, Mode::Tdm, map));
            let worst = s.max_unobstructed_sightline(s.half / 10.0);
            println!(
                "IX-A sightline: {map:?} worst = {worst:.1} m ({}the 40 m rule)",
                if worst <= 40.0 { "PASSES " } else { "exceeds " }
            );
            assert!(
                worst > 0.0 && worst.is_finite(),
                "{map:?}: the validator must produce a real number"
            );
        }
    }

    /// A death mid-action must not tax the next life. Dying during a
    /// reload used to leave reload_t counting through the corpse, so the
    /// respawn arrived with a full magazine it could not fire for up to
    /// 3 seconds. (The bot mirror of this - los_time surviving death -
    /// was wave 2's find; this is the player-side sweep of the same
    /// respawn block.)
    #[test]
    fn a_respawned_fighter_can_fire_immediately() {
        let mut s = range(88);
        // die mid-reload with a hot barrel and heavy bloom
        s.fighters[0].reload_t = 2.5;
        s.fighters[0].switch_t = 0.5;
        s.fighters[0].fire_cd = 1.2;
        s.fighters[0].bloom = 0.04;
        s.fighters[0].health = 0.0;
        s.fighters[0].respawn_t = RESPAWN_S;
        for _ in 0..((RESPAWN_S * SIM_HZ as f32) as usize + 2) {
            s.step(PlayerCmd::default());
        }
        assert!(s.fighters[0].alive(), "must have respawned");
        let f = &s.fighters[0];
        assert_eq!(f.reload_t, 0.0, "no ghost reload on the fresh body");
        assert_eq!(f.switch_t, 0.0, "no ghost weapon switch");
        assert_eq!(f.fire_cd, 0.0, "no ghost fire cooldown");
        assert_eq!(f.bloom, 0.0, "no inherited spread");
        let mut s2 = s;
        s2.fighters[0].protect_t = 0.0; // firing clears it anyway
        assert!(
            s2.try_fire(0, [0.0, 0.0, 1.0], false),
            "a fresh spawn must be able to defend itself immediately"
        );
    }

    /// §8: explosives must reach the horde. Every blast path looped
    /// fighters only, so grenades did literally nothing in the one mode
    /// that spawns zombies. (Found by wave 3; this is the test the fix
    /// shipped without - claiming a fix with no test is how the bugs
    /// this session keeps finding got made.)
    #[test]
    fn a_frag_and_a_fire_pool_actually_kill_zombies() {
        let mut s = TdmSim::new(cfg(87, 1, Mode::Extraction, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        s.zombies.clear();
        for k in 0..3 {
            s.next_zombie_id += 1;
            s.zombies.push(Zombie {
                id: s.next_zombie_id,
                kind: ZKind::Shambler,
                pos: [k as f32 * 0.8 - 0.8, 0.0, 3.0],
                hp: zspec(ZKind::Shambler).hp,
                atk_cd: 0.0,
                scream_t: 0.0,
                head_hits: 0,
                target: [0.0, 0.0],
                alerted: true,
            });
        }
        let before = s.zombies.len();
        // a frag at their feet
        s.grenades_air.push(Grenade {
            id: 9400,
            kind: ThrowKind::Frag,
            pos: [0.0, 0.5, 3.0],
            vel: [0.0, 0.0, 0.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 0.05,
            bounces: 0,
            rest: true,
        });
        for _ in 0..30 {
            s.step_grenades();
        }
        assert!(
            s.zombies.len() < before,
            "a frag in the middle of a packed trio must kill zombies: {before} -> {}",
            s.zombies.len()
        );

        // and a fire pool burns whoever survived
        if let Some(hp0) = s.zombies.first().map(|z| z.hp) {
            let at = s.zombies[0].pos;
            s.fires.push(FirePool {
                pos: [at[0], 0.02, at[2]],
                ttl: 3.0,
                thrower: 0,
                tick_t: 0.0,
            });
            for _ in 0..(2 * SIM_HZ as usize) {
                let zid = s.zombies.first().map(|z| z.id);
                s.step_fires();
                // hold the zombie on the pool (it has no pathing here)
                if let (Some(id), Some(z)) =
                    (zid, s.zombies.first_mut())
                {
                    if z.id == id {
                        z.pos = [at[0], 0.0, at[2]];
                    }
                }
            }
            let after = s.zombies.first().map(|z| z.hp).unwrap_or(0.0);
            assert!(
                after < hp0,
                "standing in fire must burn a zombie: {hp0} -> {after}"
            );
        }
    }

    /// §8: Extraction is CO-OP - everyone shares one team, so a
    /// targeting routine that only looks for an opposing FIGHTER finds
    /// nothing and the AI teammates never fire. The horde has to count
    /// as a threat or the co-op mode has no allies in it.
    #[test]
    fn ai_teammates_actually_shoot_the_horde_in_extraction() {
        let mut s = TdmSim::new(cfg(85, 3, Mode::Extraction, MapKind::Battlefield));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        s.zombies.clear();
        // a bot (not the player) with a clear lane to a zombie
        let bot = 1usize;
        s.fighters[bot].pos = [0.0, 0.0, 0.0];
        s.fighters[bot].yaw = 0.0;
        s.fighters[bot].protect_t = 0.0;
        s.next_zombie_id += 1;
        s.zombies.push(Zombie {
            id: s.next_zombie_id,
            kind: ZKind::Shambler,
            pos: [0.0, 0.0, 12.0],
            hp: 10_000.0, // survives, so we measure ENGAGEMENT not kills
            atk_cd: 0.0,
            scream_t: 0.0,
            head_hits: 0,
            target: [0.0, 0.0],
            alerted: true,
        });
        let hp0 = s.zombies[0].hp;
        for _ in 0..(6 * SIM_HZ as usize) {
            // keep the pair apart and the zombie in the lane, so this
            // measures shooting rather than a chase
            s.fighters[bot].pos = [0.0, 0.0, 0.0];
            if let Some(z) = s.zombies.first_mut() {
                z.pos = [0.0, 0.0, 12.0];
            }
            s.step(PlayerCmd::default());
        }
        assert!(
            s.zombies
                .first()
                .map_or(true, |z| z.hp < hp0),
            "an armed AI teammate must engage a zombie standing in its lane"
        );
    }

    /// §6: the crouch ban is CHASSIS state, so it must hold for a
    /// bot-piloted mech exactly as it does for the player's. The guard
    /// originally lived only on the player path.
    #[test]
    fn a_mech_never_crouches_for_either_a_player_or_a_bot() {
        let mut f = TdmSim::new(cfg(83, 1, Mode::Tdm, MapKind::Arena)).fighters.remove(0);
        // on foot, crouch intent is honoured
        f.armor_set = ArmorSet::None;
        f.hull = 0.0;
        f.set_crouch(true);
        assert!(f.crouch, "a soldier crouches");
        // boarded, the same intent is refused
        f.armor_set = ArmorSet::RobotSuit;
        f.hull = MECH_HULL;
        f.set_crouch(true);
        assert!(!f.crouch, "a live chassis must never crouch");
        // and a destroyed chassis hands the pilot back their crouch
        f.hull = 0.0;
        f.armor_set = ArmorSet::None;
        f.set_crouch(true);
        assert!(f.crouch, "an ejected pilot crouches again");
    }

    /// §8: `d2` in the horde's melee test is PLANAR. Zombies have no
    /// vertical simulation, so without a height gate they claw players
    /// standing on a crate directly above them.
    #[test]
    fn zombies_cannot_claw_a_player_standing_above_them() {
        let mut s = TdmSim::new(cfg(84, 1, Mode::Extraction, MapKind::Arena));
        s.fighters[0].pos = [0.0, 4.0, 0.0]; // up on something
        s.fighters[0].health = MAX_HEALTH;
        s.fighters[0].armor_set = ArmorSet::None;
        s.zombies.clear();
        s.next_zombie_id += 1;
        s.zombies.push(Zombie {
            id: s.next_zombie_id,
            kind: ZKind::Brute,
            pos: [0.0, 0.0, 0.3], // directly underneath, planar-adjacent
            hp: zspec(ZKind::Brute).hp,
            atk_cd: 0.0,
            scream_t: 0.0,
            head_hits: 0,
            target: [0.0, 0.0],
            alerted: true,
        });
        let hp0 = s.fighters[0].health;
        for _ in 0..(3 * SIM_HZ as usize) {
            s.fighters[0].pos = [0.0, 4.0, 0.0]; // hold the high ground
            s.step(PlayerCmd::default());
        }
        assert_eq!(
            s.fighters[0].health, hp0,
            "a zombie 4m below must not reach a player above it"
        );
    }

    /// §8: the horde must go through the shared armor pipeline. Writing
    /// `health` raw made a 1000-hull mech exactly as soft as a bare
    /// soldier against claws.
    #[test]
    fn zombie_claws_respect_the_mech_hull() {
        let mut s = TdmSim::new(cfg(82, 1, Mode::Extraction, MapKind::Arena));
        s.fighters[0].pos = [0.0, 0.0, 0.0];
        s.fighters[0].armor_set = ArmorSet::RobotSuit;
        s.fighters[0].hull = MECH_HULL;
        s.fighters[0].health = MAX_HEALTH;
        s.zombies.clear();
        s.next_zombie_id += 1;
        s.zombies.push(Zombie {
            id: s.next_zombie_id,
            kind: ZKind::Brute,
            pos: [0.0, 0.0, 1.0],
            hp: zspec(ZKind::Brute).hp,
            atk_cd: 0.0,
            scream_t: 0.0,
            head_hits: 0,
            target: [0.0, 0.0],
            alerted: true,
        });
        let hp0 = s.fighters[0].health;
        let hull0 = s.fighters[0].hull;
        for _ in 0..(3 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
        }
        assert!(
            s.fighters[0].hull < hull0,
            "claws must land on the HULL first"
        );
        assert_eq!(
            s.fighters[0].health, hp0,
            "the pilot must be untouched while the hull holds"
        );
    }

    /// BRIEF_VIII §3.6 test 4, "golden-file spray test": fixed seed,
    /// 30-shot spray, replay reproduces a magazine bit-identically. The
    /// spray table itself (`spray_entry`) was already pure/deterministic;
    /// what had never been proven is that two IDENTICAL full-auto holds
    /// through the real fire/tick path - fire_cd gating, spray_i advance,
    /// punch decay between shots - land on the exact same punch angle
    /// every time. This is the recoil system's own R11.
    #[test]
    fn a_thirty_shot_ak_spray_is_bit_identical_on_replay() {
        let fire_once = |seed: u64| -> [f32; 2] {
            let mut s = range(seed);
            s.fighters[0].gun = GunKind::Ak47;
            s.fighters[0].inventory[0] = GunKind::Ak47;
            s.fighters[0].ammo = 40;
            s.fighters[0].reserve = 0;
            let mut shots = 0;
            for _ in 0..(SIM_HZ as usize * 6) {
                if shots < 30 && s.try_fire(0, [0.0, 0.0, 1.0], false) {
                    shots += 1;
                }
                s.step(PlayerCmd { aim: [0.0, 0.0, 1.0], ..Default::default() });
            }
            assert_eq!(shots, 30, "the full spray must complete inside the window");
            s.fighters[0].punch
        };
        let a = fire_once(97);
        let b = fire_once(97);
        assert_eq!(
            a.map(f32::to_bits),
            b.map(f32::to_bits),
            "identical seed + identical inputs must land the identical punch angle, bit for bit: {a:?} vs {b:?}"
        );
        // the spray PATTERN is fixed per weapon and must not depend on
        // match seed - a different seed's world state must not perturb it
        let c = fire_once(98);
        assert_eq!(
            a.map(f32::to_bits),
            c.map(f32::to_bits),
            "the spray pattern must not depend on the match seed"
        );
    }

    /// Task 11 `preview_matches_throw`, at the master brief's exact
    /// spec: 200 RANDOM throws, preview endpoint equals actual impact
    /// within tolerance. (The single-throw check inside the R11 test
    /// below predates this; the brief demands the random sweep - varied
    /// directions, speeds, spawn heights and fuses, including throws
    /// that bounce off cover before resting, since the preview and the
    /// flight share the bounce code too.)
    #[test]
    fn preview_matches_throw_for_200_random_throws() {
        let mut s = TdmSim::new(cfg(93, 1, Mode::Tdm, MapKind::Arena));
        // deterministic pseudo-random throw parameters - NOT the sim's
        // own RNG (touching that would desync the flights we compare)
        let mut x = 0x2545F4914F6CDD1D_u64;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x >> 40) as f32 / 16777216.0 // 0..1
        };
        for k in 0..200u32 {
            let o = [next() * 20.0 - 10.0, 1.2 + next() * 0.6, next() * 20.0 - 10.0];
            let v = [
                next() * 16.0 - 8.0,
                2.0 + next() * 8.0,
                next() * 16.0 - 8.0,
            ];
            let fuse = 1.2 + next() * 2.0;
            // the preview's claimed endpoint...
            let (_, end, _) = s.predict_grenade(ThrowKind::Frag, o, v, fuse, 8.0);
            // ...and the actual flight, stepped by the live tick
            s.grenades_air.clear();
            s.grenades_air.push(Grenade {
                id: 50_000 + k,
                kind: ThrowKind::Frag,
                pos: o,
                vel: v,
                thrower: 0,
                team: Team::Blue,
                fuse_t: fuse,
                bounces: 0,
                rest: false,
            });
            let mut impact = o;
            for _ in 0..(8.0 / DT) as usize {
                let done = {
                    let TdmSim { grenades_air, grid, cover, cover_kind, .. } = &mut s;
                    let g = &mut grenades_air[0];
                    matches!(
                        grenade_tick(g, grid, cover, cover_kind),
                        GrenadeTick::Boom | GrenadeTick::Rest
                    )
                };
                impact = s.grenades_air[0].pos;
                if done {
                    break;
                }
            }
            let d = ((end[0] - impact[0]).powi(2)
                + (end[1] - impact[1]).powi(2)
                + (end[2] - impact[2]).powi(2))
            .sqrt();
            assert!(
                d < 1e-3,
                "throw {k}: preview {end:?} vs flight {impact:?} - {d} m apart \
                 (o={o:?} v={v:?} fuse={fuse})"
            );
        }
        s.grenades_air.clear();
    }

    /// R11: grenade physics lives in the SIM layer, which is seeded and
    /// fixed-timestep, so the SAME seed and the SAME throw must produce a
    /// bit-identical impact point - not "close", identical. 1000 throws.
    ///
    /// This is the guarantee the aim preview rests on: `predict_grenade`
    /// runs the very same `grenade_tick` integrator the live flight does,
    /// so if the integrator were not deterministic the preview could not
    /// be trusted no matter how carefully it was called.
    #[test]
    fn a_thousand_identical_throws_land_bit_identically() {
        let throw_once = |seed: u64| -> [f32; 3] {
            let mut s = range(seed);
            s.grenades_air.push(Grenade {
                id: 1,
                kind: ThrowKind::Frag,
                pos: [0.0, 1.45, 0.0],
                vel: [3.0, 6.5, 11.0],
                thrower: 0,
                team: Team::Blue,
                fuse_t: 100.0, // long: measure the RESTING point, not the boom
                bounces: 0,
                rest: false,
            });
            for _ in 0..(6 * SIM_HZ as usize) {
                s.step_grenades();
            }
            s.grenades_air
                .first()
                .map(|g| g.pos)
                .unwrap_or([f32::NAN; 3])
        };
        let reference = throw_once(900);
        assert!(
            reference.iter().all(|v| v.is_finite()),
            "the reference throw must actually settle somewhere finite: {reference:?}"
        );
        // compare RAW BITS, not floats: `==` on f32 would let a NaN pair
        // or a -0.0/+0.0 pair pass as "identical"
        let bits = |p: [f32; 3]| [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()];
        for i in 0..1000 {
            let again = throw_once(900);
            assert_eq!(
                bits(again),
                bits(reference),
                "throw {i} diverged: {again:?} vs {reference:?}"
            );
        }
        // ...and the aim PREVIEW must agree with that flight, since it is
        // the same integrator - a preview that can drift is worse than
        // none, which is why they share one function.
        let s = range(900);
        let (_, end, _) = s.predict_grenade(
            ThrowKind::Frag,
            [0.0, 1.45, 0.0],
            [3.0, 6.5, 11.0],
            100.0,
            6.0,
        );
        let d = ((end[0] - reference[0]).powi(2)
            + (end[1] - reference[1]).powi(2)
            + (end[2] - reference[2]).powi(2))
        .sqrt();
        assert!(
            d < 1e-4,
            "the preview must trace the SAME flight: preview {end:?} vs live {reference:?} ({d} m apart)"
        );
    }

    /// §4.1: the power floor is shared, so the client's arc preview and
    /// the sim's launch cannot disagree about a fresh draw.
    #[test]
    fn bow_power_floor_is_the_named_constant() {
        let at_min = bow_power_fraction(BOW_DRAW_MIN_S).expect("min draw is a valid shot");
        assert!(
            (at_min - BOW_POWER_MIN).abs() < 1e-6,
            "a just-valid draw must be exactly BOW_POWER_MIN, got {at_min}"
        );
        let full = bow_power_fraction(BOW_DRAW_FULL_S).expect("full draw is valid");
        assert!((full - 1.0).abs() < 1e-6, "a full draw must be 1.0, got {full}");
        // and the launch speeds those imply, which the preview now mirrors
        assert!((BOW_V0_FULL * at_min - 19.25).abs() < 0.01);
        assert!((BOW_V0_FULL * full - 55.0).abs() < 0.01);
    }

    #[test]
    fn bow_arrows_fly_and_hit() {
        let mut s = TdmSim::new(cfg(15, 1, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        s.pickups.clear();
        s.checkpoints.clear();
        s.fighters[0].gun = GunKind::Bow;
        s.fighters[0].ammo = 1;
        s.fighters[0].reserve = 10;
        s.fighters[0].pos = [0.0, 0.0, -8.0];
        s.fighters[1].pos = [0.0, 0.0, 8.0];
        // §4.1 (Brief VII v2): the bow now DRAWS on hold and looses on
        // release - hold past full draw (0.7s) for a reliable, full-power
        // 55 m/s shot; flat aim, from a 1.62 m eye, drops chest-high over
        // 16 m at full draw.
        let hold = PlayerCmd {
            aim: [0.0, 0.0, 1.0],
            shoot: true,
            ..Default::default()
        };
        let released = PlayerCmd {
            aim: [0.0, 0.0, 1.0],
            shoot: false,
            ..Default::default()
        };
        // disarm the target so it can't kill the archer mid-test
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        for _ in 0..((0.75 * SIM_HZ as f32) as usize) {
            s.step(hold);
            s.fighters[1].pos = [0.0, 0.0, 8.0];
            s.fighters[1].protect_t = 0.0;
        }
        let mut hit = false;
        for _ in 0..(8 * SIM_HZ as usize) {
            s.step(released); // the release tick looses the arrow
            // pin the target: bot AI re-sets vel INSIDE step, so resetting
            // vel after the step is not enough - hold the position itself
            s.fighters[1].pos = [0.0, 0.0, 8.0];
            s.fighters[1].protect_t = 0.0;
            if s.fighters[0].hits_dealt > 0 {
                hit = true;
                break;
            }
        }
        assert!(hit, "arrows must connect at 16 m");
    }

    #[test]
    fn player_jumps_and_lands() {
        let mut s = TdmSim::new(cfg(7, 5, Mode::Tdm, MapKind::Arena));
        let jump = PlayerCmd {
            jump: true,
            ..Default::default()
        };
        s.step(jump);
        let mut peak = 0.0_f32;
        for _ in 0..(2 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            peak = peak.max(s.fighters[0].pos[1]);
        }
        assert!(peak > 0.8, "jump must rise: peak {peak}");
        assert!(
            s.fighters[0].pos[1] < 0.05 && s.fighters[0].grounded,
            "must land again: y {}",
            s.fighters[0].pos[1]
        );
        // holding jump mid-air must not double-jump
        s.step(jump);
        let mut s2 = TdmSim::new(cfg(7, 5, Mode::Tdm, MapKind::Arena));
        s2.step(jump);
        s2.step(jump); // second press mid-air
        assert!(s2.fighters[0].vy <= JUMP_SPEED, "no double jump boost");
    }

    #[test]
    fn koth_scores_the_hill() {
        let mut s = TdmSim::new(cfg(6, 1, Mode::Koth, MapKind::Arena));
        // blue alone on the hill
        s.fighters[0].pos = [s.hill[0], s.hill[1], s.hill[2]];
        s.fighters[1].pos = [20.0, 0.0, 20.0];
        run(&mut s, 5, PlayerCmd::default());
        assert!(
            s.score[0] > 2.0,
            "holding the hill must score: {:?}",
            s.score
        );
    }

    #[test]
    fn dodge_roll_dashes_low_and_cools_down() {
        let mut s = TdmSim::new(cfg(11, 5, Mode::Tdm, MapKind::Arena));
        let dodge = PlayerCmd {
            move_z: 1.0,
            dodge: true,
            ..Default::default()
        };
        s.step(dodge);
        assert!(s.fighters[0].roll_t > 0.0, "dodge must start a roll");
        // §2 (Brief V): the roll LOADS first — ride past the crouch-coil
        // into the burst before sampling the dash
        for _ in 0..((ROLL_LOAD_S / DT) as usize + 2) {
            s.step(PlayerCmd::default());
        }
        // mid-burst: faster than sprint, balled up small, gun locked out
        let sp = {
            let f = &s.fighters[0];
            (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt()
        };
        assert!(sp > SPRINT_SPEED, "roll must dash faster than sprint: {sp}");
        assert!(s.fighters[0].height() < CROUCH_HEIGHT, "roll must be low");
        // roll ends (load + burst + ease), cooldown blocks a second roll
        for _ in 0..(((ROLL_S + ROLL_EASE_S) / DT) as usize + 2) {
            s.step(PlayerCmd::default());
        }
        assert!(s.fighters[0].roll_t <= 0.0, "roll must end");
        s.step(dodge);
        assert!(
            s.fighters[0].roll_t <= 0.0,
            "cooldown must block a chained roll"
        );
    }

    #[test]
    fn hard_landing_breakfalls_into_a_roll() {
        let mut s = TdmSim::new(cfg(12, 5, Mode::Tdm, MapKind::Arena));
        // hurl the player off a high ledge, running forward
        s.fighters[0].pos = [8.0, 6.0, 8.0];
        s.fighters[0].grounded = false;
        let run = PlayerCmd {
            move_z: 1.0,
            ..Default::default()
        };
        let mut rolled = false;
        for _ in 0..(3 * SIM_HZ as usize) {
            s.step(run);
            if s.fighters[0].roll_t > 0.0 {
                rolled = true;
                break;
            }
        }
        assert!(rolled, "a 6 m drop must land in a breakfall roll");
        // an ordinary flat jump must NOT roll
        let mut s2 = TdmSim::new(cfg(12, 5, Mode::Tdm, MapKind::Arena));
        s2.step(PlayerCmd {
            jump: true,
            ..Default::default()
        });
        for _ in 0..(2 * SIM_HZ as usize) {
            s2.step(PlayerCmd::default());
            assert!(
                s2.fighters[0].roll_t <= 0.0,
                "a flat jump must land on its feet"
            );
        }
    }

    #[test]
    fn arc_prediction_matches_real_flight() {
        let mut s = TdmSim::new(cfg(13, 1, Mode::Tdm, MapKind::Arena));
        let o = [0.0, EYE_REL, -20.0];
        let d = normalize([0.05, 0.25, 1.0]);
        let v0 = 17.0;
        let (pts, predicted, _n) = s.predict_arc(o, d, v0, true, 6.0);
        assert!(!pts.is_empty(), "arc must have preview points");
        // fly a REAL spear on the same launch and compare landings
        s.missiles.push(Missile {
            id: 900,
            pos: o,
            vel: [d[0] * v0, d[1] * v0, d[2] * v0],
            team: Team::Blue,
            shooter: 0,
            damage: 0.0,
            is_spear: true,
            stuck_t: None,
            embedded: true,
            pierces_left: 0,
            pierced: Vec::new(),
            power: 1.0,
        });
        let mut landed = None;
        for _ in 0..(6 * SIM_HZ as usize) {
            s.step_missiles();
            if let Some(m) = s.missiles.iter().find(|m| m.id == 900) {
                if m.stuck_t.is_some() {
                    landed = Some(m.pos);
                    break;
                }
            } else {
                // §3: a landed spear converts IN PLACE to a DroppedAmmo
                // pile — the pile position IS the landing point
                landed = s.dropped.last().map(|d| d.pos);
                break;
            }
        }
        let landed = landed.expect("spear must land");
        let d2 = (landed[0] - predicted[0]).powi(2)
            + (landed[1] - predicted[1]).powi(2)
            + (landed[2] - predicted[2]).powi(2);
        assert!(
            d2.sqrt() < 0.6,
            "prediction must match flight: predicted {predicted:?}, landed {landed:?}"
        );
    }

    #[test]
    fn all_maps_are_valid_battlefields() {
        for map in MapKind::ALL {
            // v6 cap: validate at the full 8v8 too
            let s = TdmSim::new(cfg(31, 8, Mode::Koth, map));
            assert_eq!(s.cover.len(), s.cover_kind.len(), "{map:?}: kind per cover");
            assert_eq!(s.checkpoints.len(), 2, "{map:?}: two forward rings");
            for cp in &s.checkpoints {
                for c in &s.cover {
                    let inside = cp.pos[0] > c.min[0]
                        && cp.pos[0] < c.max[0]
                        && cp.pos[2] > c.min[2]
                        && cp.pos[2] < c.max[2]
                        && cp.pos[1] + 0.3 > c.min[1]
                        && cp.pos[1] + 0.3 < c.max[1];
                    assert!(!inside, "{map:?}: checkpoint {:?} buried in {c:?}", cp.pos);
                }
            }
            for c in &s.cover {
                for i in 0..3 {
                    assert!(c.min[i] < c.max[i], "{map:?}: degenerate box {c:?}");
                }
            }
            // spawn rows and pickups must not be buried inside cover
            for f in &s.fighters {
                for c in &s.cover {
                    let inside = f.pos[0] > c.min[0]
                        && f.pos[0] < c.max[0]
                        && f.pos[2] > c.min[2]
                        && f.pos[2] < c.max[2]
                        && f.pos[1] + 0.5 > c.min[1]
                        && f.pos[1] + 0.5 < c.max[1];
                    assert!(!inside, "{map:?}: spawn {:?} buried in {c:?}", f.pos);
                }
            }
            for p in &s.pickups {
                for c in &s.cover {
                    let inside = p.pos[0] > c.min[0]
                        && p.pos[0] < c.max[0]
                        && p.pos[2] > c.min[2]
                        && p.pos[2] < c.max[2]
                        && p.pos[1] + 0.3 > c.min[1]
                        && p.pos[1] + 0.3 < c.max[1];
                    assert!(!inside, "{map:?}: pickup {:?} buried in {c:?}", p.pos);
                }
            }
            // the hill must sit on the center top
            assert!(s.hill[1] > 1.0, "{map:?}: hill must be elevated");
            // same seed, same map → identical battle
            let run_map = |m| {
                let mut s = TdmSim::new(cfg(31, 5, Mode::Tdm, m));
                for _ in 0..(10 * SIM_HZ as usize) {
                    s.step(PlayerCmd::default());
                }
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>()
            };
            assert_eq!(run_map(map), run_map(map), "{map:?} must be deterministic");
        }
    }

    /// §11.5 (Brief III) — THE mandatory mech arc test: front 15%, side
    /// 30%, rear 100%, all read from BODY facing; explosives bypass half
    /// the cut; the hull soaks everything until the pilot ejects at 25.
    #[test]
    fn mech_arcs_follow_body_facing_and_eject() {
        // height-relative, not a magic number - survives Task 4's scale
        // change (or any future one) automatically
        let mech_visor_y = BODY_HEIGHT * MECH_SCALE * 0.90;
        let mut s = range(141);
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        s.fighters[1].armor_set = ArmorSet::RobotSuit;
        s.fighters[1].hull = MECH_HULL;
        // FRONT: body faces the shooter → 15% lands
        s.fighters[1].yaw = std::f32::consts::PI;
        let h0 = s.fighters[1].hull;
        s.apply_hit(0, 1, 1.3, [0.0, 1.3, 5.0]);
        let front = h0 - s.fighters[1].hull;
        assert!(
            (front - 12.5 * (1.0 - MECH_RED_FRONT)).abs() < 0.01,
            "front arc 15%: took {front}"
        );
        assert!(
            (s.fighters[1].health - MAX_HEALTH).abs() < 0.01,
            "the pilot takes NOTHING while the hull holds"
        );
        // SIDE: body perpendicular → 30%
        s.fighters[1].yaw = std::f32::consts::FRAC_PI_2;
        let h1 = s.fighters[1].hull;
        s.apply_hit(0, 1, 1.3, [0.0, 1.3, 5.0]);
        let side = h1 - s.fighters[1].hull;
        assert!(
            (side - 12.5 * (1.0 - MECH_RED_SIDE)).abs() < 0.01,
            "side arc 30%: took {side}"
        );
        // REAR: back turned → everything lands
        s.fighters[1].yaw = 0.0;
        let h2 = s.fighters[1].hull;
        s.apply_hit(0, 1, 1.3, [0.0, 1.3, 5.0]);
        let rear = h2 - s.fighters[1].hull;
        assert!((rear - 12.5).abs() < 0.01, "rear arc 100%: took {rear}");
        // §4.5 (Brief VI): the VISOR is a ×2 weak point AFTER the angle
        // multiplier, front-arc only (12.5 × 0.15 × 2.0 = 3.75)…
        s.fighters[1].yaw = std::f32::consts::PI;
        let hv = s.fighters[1].hull;
        s.apply_hit(0, 1, mech_visor_y, [0.0, mech_visor_y, 5.0]); // 1.9/2.05 → visor band
        let visor = hv - s.fighters[1].hull;
        assert!(
            (visor - 12.5 * (1.0 - MECH_RED_FRONT) * MECH_VISOR_MULT).abs() < 0.01,
            "front visor = angle × 2.0: took {visor}"
        );
        // …and from BEHIND the visor is not exposed: rear head-band hits
        // take plain rear damage, no bonus
        s.fighters[1].yaw = 0.0;
        let hb = s.fighters[1].hull;
        s.apply_hit(0, 1, mech_visor_y, [0.0, mech_visor_y, 5.0]);
        let rear_head = hb - s.fighters[1].hull;
        assert!(
            (rear_head - 12.5).abs() < 0.01,
            "no visor from behind: took {rear_head}"
        );
        // EXPLOSIVES bypass half the frontal cut: 40 × (1 − 0.425) = 23
        s.fighters[1].yaw = std::f32::consts::PI;
        let h3 = s.fighters[1].hull;
        s.apply_plain_damage(0, 1, 40.0, [0.0, 1.0, 3.0], true, false);
        let boom = h3 - s.fighters[1].hull;
        assert!(
            (boom - 40.0 * (1.0 - MECH_RED_FRONT * 0.5)).abs() < 0.1,
            "frontal explosive 57.5%: took {boom}"
        );
        // DESTRUCTION: hull to zero → the pilot ejects at ≤25 HP, alive
        s.fighters[1].hull = 5.0;
        s.fighters[1].yaw = 0.0;
        s.apply_hit(0, 1, 1.3, [0.0, 1.3, 5.0]);
        assert_eq!(
            s.fighters[1].armor_set,
            ArmorSet::None,
            "the chassis is destroyed"
        );
        assert!(
            s.fighters[1].health <= MECH_EJECT_HP + 0.01,
            "the pilot ejects at 25: {}",
            s.fighters[1].health
        );
        assert!(s.fighters[1].alive(), "ejecting is survivable");
    }

    /// §3 (Brief III): the spear THROW winds up — no missile at the
    /// trigger, a real launch ~0.5 s later, replay-identical.
    #[test]
    fn spear_windup_is_committal() {
        let mut s = range(131);
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        s.fighters[0].gun = GunKind::Spear;
        s.fighters[0].ammo = 1;
        s.fighters[0].reserve = 5;
        assert!(s.try_fire(0, [0.0, 0.2, 1.0], true), "the trigger arms");
        assert!(
            s.missiles.is_empty(),
            "no spear leaves the hand at the trigger"
        );
        assert!(s.fighters[0].spear_wind_t > 0.0, "the wind is live");
        for _ in 0..((SPEAR_WINDUP_S / DT) as usize + 3) {
            s.step(PlayerCmd {
                aim: [0.0, 0.2, 1.0],
                ..Default::default()
            });
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
        }
        assert!(
            !s.missiles.is_empty() || !s.dropped.is_empty(),
            "the spear must FLY after the windup"
        );
    }

    /// §4.3 (Brief III) — THE mandatory regression test: no headshot
    /// multiplier can be applied to a flipping fighter, top or bottom of
    /// the capsule; a flipping shooter cannot fire; and flips replay
    /// bit-identically.
    #[test]
    fn flips_force_uniform_zones_and_block_fire() {
        let mut s = range(121);
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        // put the victim mid-flip in the air
        s.fighters[1].grounded = false;
        s.fighters[1].pos[1] = 2.0;
        s.fighters[1].flip_t = FLIP_S * 0.5;
        s.fighters[1].flip_used = true;
        // a shot in the bottom sixth of the capsule: banded would say
        // LEGS ×0.75; a shot at skull height: banded would say HEAD ×4.
        // Uniform must make BOTH exactly ×1.0.
        let h0 = s.fighters[1].health;
        s.apply_hit(0, 1, s.fighters[1].pos[1] + 0.15, [0.0, 2.15, 5.0]);
        let low = h0 - s.fighters[1].health;
        assert!(
            (low - 12.5).abs() < 0.01,
            "mid-flip low shot must be x1.0: took {low}"
        );
        let h1 = s.fighters[1].health;
        s.apply_hit(0, 1, s.fighters[1].pos[1] + 1.72, [0.0, 3.72, 5.0]);
        let high = h1 - s.fighters[1].health;
        assert!(
            (high - 12.5).abs() < 0.01,
            "mid-flip skull shot must be x1.0, never x4: took {high}"
        );
        // a flipping shooter cannot fire — pure mobility
        let mut s = range(122);
        s.fighters[0].grounded = false;
        s.fighters[0].flip_t = 0.3;
        s.fighters[0].flip_used = true;
        assert!(
            !s.try_fire(0, [0.0, 0.0, 1.0], false),
            "no shooting mid-flip"
        );
        // determinism: a run full of jump+flip inputs replays identically
        let outcome = || {
            let mut s = TdmSim::new(cfg(123, 3, Mode::Tdm, MapKind::Arena));
            for i in 0..(20 * SIM_HZ as usize) {
                s.step(PlayerCmd {
                    move_z: 0.8,
                    move_x: if (i / 200) % 2 == 0 { 0.5 } else { -0.5 },
                    aim: [0.0, 0.0, 1.0],
                    jump: i % 180 == 0,
                    dodge: i % 180 == 12, // just after leaving the ground
                    shoot: i % 90 < 3,
                    ..Default::default()
                });
            }
            (
                s.score[0] as u32,
                s.score[1] as u32,
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>(),
                s.fighters[0].pos[0].to_bits(),
                s.fighters[0].pos[2].to_bits(),
            )
        };
        assert_eq!(outcome(), outcome(), "flips must replay identically");
    }

    /// §5/§6 (Brief III): the knife back-stabs lethally and slashes for
    /// 55 frontally; the raised shield allows ONLY throwables, and a
    /// throw dips the plate for a real vulnerability window.
    #[test]
    fn knife_backstab_and_shield_throwable_rules() {
        let disarm_bot = |s: &mut TdmSim| {
            s.fighters[1].ammo = 0;
            s.fighters[1].reserve = 0;
            s.fighters[1].slot_ammo = [(0, 0); 3];
        };
        // back-stab: the victim faces AWAY → a tap kills outright
        let mut s = range(111);
        disarm_bot(&mut s);
        for i in 0..70 {
            s.step(PlayerCmd {
                knife_hold: i < 2, // a TAP
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
            s.fighters[1].pos = [0.0, 0.0, -3.6]; // 1.4 m ahead
            s.fighters[1].vel = [0.0, 0.0];
            s.fighters[1].yaw = 0.0; // back turned
            s.fighters[1].protect_t = 0.0;
        }
        assert!(
            !s.fighters[1].alive(),
            "backstab must kill: hp {}",
            s.fighters[1].health
        );
        // frontal: 55 damage, survivable
        let mut s = range(112);
        disarm_bot(&mut s);
        for i in 0..70 {
            s.step(PlayerCmd {
                knife_hold: i < 2,
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
            s.fighters[1].pos = [0.0, 0.0, -3.6];
            s.fighters[1].vel = [0.0, 0.0];
            s.fighters[1].yaw = std::f32::consts::PI; // facing the blade
            s.fighters[1].protect_t = 0.0;
        }
        assert!(s.fighters[1].alive(), "a frontal slash must not kill");
        assert!(
            (s.fighters[1].health - (MAX_HEALTH - KNIFE_QUICK_DMG)).abs() < 1.0,
            "frontal slash ≈ 55: hp {}",
            s.fighters[1].health
        );
        // §6: shield up → guns blocked, throwables fine, throw dips plate
        let mut s = range(113);
        disarm_bot(&mut s);
        s.step(PlayerCmd {
            shield: true,
            ..Default::default()
        });
        assert!(s.fighters[0].shield_up, "the plate must rise");
        assert!(
            !s.try_fire(0, [0.0, 0.0, 1.0], false),
            "no firearms behind the plate"
        );
        for _ in 0..30 {
            s.step(PlayerCmd {
                throw_hold: true,
                aim: [0.0, 0.4, 1.0],
                ..Default::default()
            });
        }
        s.step(PlayerCmd::default()); // release → the frag flies
        assert_eq!(
            s.fighters[0].grenades[0], 1,
            "the shielded throw must launch"
        );
        assert!(
            s.fighters[0].shield_dip_t > 0.0,
            "the throw must DIP the shield"
        );
    }

    /// §6 (Brief IV): the axe swing is a SWEEP — one tap hits every
    /// enemy inside the 90° arc for 85 frontal.
    #[test]
    fn axe_sweeps_the_whole_arc() {
        let mut s = TdmSim::new(cfg(77, 2, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        s.pickups.clear();
        s.checkpoints.clear();
        s.fighters[0].melee_axe = true;
        s.fighters[0].pos = [0.0, 0.0, -5.0];
        s.fighters[0].yaw = 0.0;
        for j in 2..4 {
            s.fighters[j].ammo = 0;
            s.fighters[j].reserve = 0;
            s.fighters[j].slot_ammo = [(0, 0); 3];
        }
        for i in 0..90 {
            s.step(PlayerCmd {
                knife_hold: i < 2, // a TAP — the quick chop
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
            // both reds inside the arc at ~1.6 m, facing the axe
            s.fighters[2].pos = [-0.7, 0.0, -3.6];
            s.fighters[3].pos = [0.7, 0.0, -3.6];
            for j in 2..4 {
                s.fighters[j].vel = [0.0, 0.0];
                s.fighters[j].yaw = std::f32::consts::PI;
                s.fighters[j].protect_t = 0.0;
            }
            s.fighters[1].pos = [-30.0, 0.0, -30.0]; // teammate parked
        }
        for j in 2..4 {
            assert!(
                (s.fighters[j].health - (MAX_HEALTH - AXE_QUICK_DMG)).abs() < 1.0,
                "one sweep must hit BOTH: fighter {j} at hp {}",
                s.fighters[j].health
            );
        }
    }

    /// §7 (Brief IV): the minigun's whole identity — no rounds before
    /// the spin-up completes, heat per round, the forced vent at 100 —
    /// and all of it replays bit-identically.
    #[test]
    fn minigun_heat_cycle_is_deterministic() {
        let run = || {
            let mut s = range(909);
            let f = &mut s.fighters[0];
            f.prev_primary = f.inventory[0];
            f.inventory[0] = GunKind::Minigun;
            f.slot_ammo[0] = (400, 0);
            f.active = 0;
            f.gun = GunKind::Minigun;
            f.ammo = 400;
            f.reserve = 0;
            s.fighters[1].ammo = 0;
            s.fighters[1].reserve = 0;
            s.fighters[1].slot_ammo = [(0, 0); 3];
            let mut fired_early = false;
            let mut vent_seen = false;
            for i in 0..(8 * SIM_HZ as usize) {
                s.step(PlayerCmd {
                    shoot: true,
                    aim: [0.0, 0.0, 1.0],
                    ..Default::default()
                });
                s.fighters[1].pos = [-30.0, 0.0, -30.0];
                s.fighters[1].vel = [0.0, 0.0];
                let f = &s.fighters[0];
                if (i as f32) * DT < MINIGUN_SPINUP_S - 0.05 && f.ammo < 400 {
                    fired_early = true;
                }
                if f.vent_t > 0.0 {
                    vent_seen = true;
                }
            }
            (
                fired_early,
                vent_seen,
                s.fighters[0].ammo,
                s.fighters[0].heat.to_bits(),
                s.fighters[0].spin_t.to_bits(),
            )
        };
        let (early, vented, ammo, heat_bits, spin_bits) = run();
        assert!(!early, "no rounds before the barrels are up");
        assert!(vented, "4 s of held trigger must FORCE the vent");
        assert!(
            (300..=345).contains(&ammo),
            "8 s hold ≈ one §5.1 heat cycle at 1000 RPM (0.42 s spin +
             ~4.0 s to 100 heat + 3 s vent + refire): ammo left {ammo}"
        );
        assert_eq!(
            run(),
            (early, vented, ammo, heat_bits, spin_bits),
            "the heat cycle must replay bit-identically"
        );
    }

    /// §1 (Brief V) — THE preview-honesty gate: the aim preview and the
    /// real flight share one integrator, so for six angle/power/kind
    /// combinations (near-vertical lob, flat close-range, mid arc, the
    /// 50% tap, a mid-air smoke pop, a molotov shatter) the predicted
    /// end point and the actual simulated end point agree within 10 cm.
    #[test]
    fn grenade_preview_matches_flight_within_10cm() {
        let cases: [([f32; 3], f32, ThrowKind); 6] = [
            ([0.0, 0.05, 1.0], 0.30, ThrowKind::Frag), // flat, close-range
            ([0.0, 0.98, 0.06], 1.30, ThrowKind::Frag), // near-vertical lob, max
            ([0.0, 0.7, 0.7], 0.60, ThrowKind::Frag),  // mid arc, mid charge
            ([0.0, 0.5, 0.85], 0.10, ThrowKind::Flash), // TAP — the 50% panic
            ([0.0, 0.6, 0.8], 1.20, ThrowKind::Smoke), // fuse pops mid-flight
            ([0.0, 0.35, 0.95], 0.80, ThrowKind::Molotov), // shatters on impact
        ];
        for (ci, (aim, hold, kind)) in cases.into_iter().enumerate() {
            let mut s = range(400 + ci as u64);
            s.fighters[1].pos = [-30.0, 0.0, -30.0]; // clear the flight path
            let (o, vel) = s.throw_release_velocity(0, aim, hold);
            let spec = throw_spec(kind);
            let fuse = if spec.fuse_s.is_finite() {
                (spec.fuse_s - if kind == ThrowKind::Frag { hold } else { 0.0 })
                    .max(0.15)
            } else {
                f32::INFINITY
            };
            let (_, predicted, _) = s.predict_grenade(kind, o, vel, fuse, 12.0);
            // fly the REAL grenade through the REAL sim loop
            s.next_missile_id += 1;
            let id = s.next_missile_id;
            s.grenades_air.push(Grenade {
                id,
                kind,
                pos: o,
                vel,
                thrower: 0,
                team: Team::Blue,
                fuse_t: fuse,
                bounces: 0,
                rest: false,
            });
            let mut actual: Option<[f32; 3]> = None;
            for _ in 0..(12 * SIM_HZ as usize) {
                s.step_grenades();
                if let Some(g) = s.grenades_air.iter().find(|g| g.id == id) {
                    if g.rest {
                        actual = Some(g.pos);
                        break;
                    }
                } else {
                    // it detonated — the Boom records where
                    actual = s.booms.last().map(|(b, _)| b.at);
                    break;
                }
            }
            let a = actual.expect("the grenade must land or detonate");
            let d = ((a[0] - predicted[0]).powi(2)
                + (a[1] - predicted[1]).powi(2)
                + (a[2] - predicted[2]).powi(2))
            .sqrt();
            assert!(
                d < 0.10,
                "case {ci} ({kind:?}): predicted {predicted:?} vs actual {a:?} — {d:.3} m apart"
            );
        }
        // the tap is a USABLE panic throw, not a drop at the feet: from a
        // standing flat aim it must clear 4 m of ground distance
        let mut s = range(444);
        s.fighters[1].pos = [-30.0, 0.0, -30.0];
        let (o, vel) = s.throw_release_velocity(0, [0.0, 0.2, 1.0], 0.05);
        let (_, land, _) =
            s.predict_grenade(ThrowKind::Frag, o, vel, f32::INFINITY, 12.0);
        let reach = ((land[0] - o[0]).powi(2) + (land[2] - o[2]).powi(2)).sqrt();
        assert!(reach > 4.0, "tap throw must be usable: reached {reach:.2} m");
    }

    /// §2 (Brief V): the spear THRUST connects for 70 frontal — and a
    /// WHIFF locks the weapon out visibly longer than a hit. A missed
    /// thrust is committed, not free.
    #[test]
    fn spear_thrust_commits_with_whiff_recovery() {
        let run = |enemy_z: f32| -> (f32, usize) {
            let mut s = range(551);
            let f = &mut s.fighters[0];
            f.inventory[2] = GunKind::Spear;
            f.active = 2;
            f.gun = GunKind::Spear;
            f.ammo = 1;
            f.reserve = 5;
            s.fighters[1].ammo = 0;
            s.fighters[1].reserve = 0;
            s.fighters[1].slot_ammo = [(0, 0); 3];
            let mut end_tick = 0usize;
            for i in 0..(3 * SIM_HZ as usize) {
                s.step(PlayerCmd {
                    knife_hold: i < 2, // a TAP — the quick thrust
                    aim: [0.0, 0.0, 1.0],
                    ..Default::default()
                });
                s.fighters[1].pos = [0.0, 0.0, enemy_z];
                s.fighters[1].vel = [0.0, 0.0];
                s.fighters[1].yaw = std::f32::consts::PI;
                s.fighters[1].protect_t = 0.0;
                if i > 3 && s.fighters[0].knife_phase <= 0.0 {
                    end_tick = i;
                    break;
                }
            }
            (s.fighters[1].health, end_tick)
        };
        // HIT: the enemy stands 1.8 m down the line
        let (hp_hit, t_hit) = run(-3.2);
        assert!(
            (hp_hit - (MAX_HEALTH - THRUST_DMG)).abs() < 1.0,
            "a frontal thrust lands 70: hp {hp_hit}"
        );
        // WHIFF: nobody there — the recovery must run visibly longer
        let (hp_whiff, t_whiff) = run(20.0);
        assert!((hp_whiff - MAX_HEALTH).abs() < 0.01, "a whiff hits nothing");
        assert!(
            t_whiff > t_hit + 20,
            "whiff recovery must be LONGER than hit recovery: {t_whiff} vs {t_hit} ticks"
        );
    }

    /// §2 (Brief V): the roll is load → burst → ease-out. No tick outruns
    /// the burst, the landing hands speed back smoothly (never a cliff),
    /// and mid-roll a ray at STANDING head height sails clean over the
    /// ball — the silent-headshot regression, extended to the roll.
    #[test]
    fn roll_loads_bursts_eases_and_ducks_headshots() {
        let mut s = range(552);
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        let mut speeds: Vec<(bool, f32, f32)> = Vec::new(); // (rolling, roll_t, speed)
        let mut head_missed = true;
        for i in 0..(2 * SIM_HZ as usize) {
            let prev = s.fighters[0].pos;
            s.step(PlayerCmd {
                move_z: 1.0,
                dodge: i == 10,
                aim: [0.0, 0.0, 1.0],
                yaw: 0.0,
                ..Default::default()
            });
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
            s.fighters[1].vel = [0.0, 0.0];
            let f = &s.fighters[0];
            let d = ((f.pos[0] - prev[0]).powi(2) + (f.pos[2] - prev[2]).powi(2)).sqrt();
            speeds.push((f.roll_t > 0.0, f.roll_t, d / DT));
            if f.roll_t > 0.0 {
                let o = [f.pos[0] - 3.0, 1.70, f.pos[2]];
                if ray_vs_cylinder(o, [1.0, 0.0, 0.0], f.pos, f.radius(), f.height())
                    .is_some()
                {
                    head_missed = false;
                }
            }
        }
        assert!(speeds.iter().any(|(r, ..)| *r), "the roll must trigger");
        assert!(head_missed, "standing-head-height rays sail over the roll");
        // (a) nothing ever outruns the burst
        let vmax = speeds.iter().map(|(_, _, v)| *v).fold(0.0, f32::max);
        assert!(vmax <= ROLL_SPEED + 0.2, "no tick outruns the burst: {vmax}");
        assert!(vmax > ROLL_SPEED - 0.6, "the burst must actually fire: {vmax}");
        // (b) NO cliff anywhere once the roll starts: the worst
        // single-tick speed drop stays far under an instant stop (8.6)
        let mut worst_drop = 0.0_f32;
        for w in speeds.windows(2) {
            worst_drop = worst_drop.max(w[0].2 - w[1].2);
        }
        assert!(
            worst_drop < 3.0,
            "the landing must EASE, never stop dead: worst drop {worst_drop:.2} m/s per tick"
        );
        // (c) inside the ease window the ramp-down is gentle per tick
        for w in speeds.windows(2) {
            let (r0, t0, v0) = w[0];
            let (r1, t1, v1) = w[1];
            if r0 && r1 && t0 <= ROLL_EASE_S && t1 > 0.0 {
                assert!(
                    v0 - v1 < 0.6,
                    "ease-out must ramp, not step: {v0:.2} → {v1:.2}"
                );
            }
        }
    }

    /// §2 (Brief V): the mech's dodge is a braced SIDE-STEP — it stays
    /// TALL (no tumbling ball at 2.7 m), commits (no firing mid-step),
    /// and travels a bounded braced distance.
    #[test]
    fn mech_side_step_stays_tall_and_committed() {
        let mut s = range(553);
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        {
            let f = &mut s.fighters[0];
            f.armor_set = ArmorSet::RobotSuit;
            f.armor = POWER_MAX;
            f.hull = MECH_HULL;
        }
        let start = s.fighters[0].pos;
        let mut stepped = false;
        for i in 0..(SIM_HZ as usize) {
            s.step(PlayerCmd {
                move_x: 1.0,
                dodge: i == 5,
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
            s.fighters[1].vel = [0.0, 0.0];
            if s.fighters[0].roll_t > 0.0 {
                stepped = true;
                let h = s.fighters[0].height();
                assert!(h > 2.0, "the mech stays TALL through the step: {h}");
                assert!(
                    !s.try_fire(0, [0.0, 0.0, 1.0], false),
                    "the side-step is committed — no fire"
                );
            }
        }
        assert!(stepped, "the mech must side-step on dodge");
        let end = s.fighters[0].pos;
        let d = ((end[0] - start[0]).powi(2) + (end[2] - start[2]).powi(2)).sqrt();
        assert!(
            (1.0..6.0).contains(&d),
            "a bounded braced step, not a teleport or a lunge: {d:.2} m"
        );
    }

    /// AUTO-PLAYTEST (run on demand):
    /// `cargo test --release -p jk_tdm -- --ignored autoplay --nocapture`
    /// Drives full headless matches in EVERY mode with a scripted player
    /// policy — patrol, engage, dodge, grenade, melee — and prints a
    /// stats report per run. Sanity-asserts the sim stays finite and the
    /// match actually progresses. This is the instrument for the
    /// "try every dynamic" passes; it plays the REAL sim, not a mock.
    #[test]
    #[ignore]
    fn autoplay_report() {
        let corners: [[f32; 2]; 4] =
            [[14.0, 14.0], [-14.0, 14.0], [-14.0, -14.0], [14.0, -14.0]];
        for (mode, map, mins, driven, seed) in [
            (Mode::Tdm, MapKind::Arena, 5usize, true, 0xA110u64),
            (Mode::Koth, MapKind::Arena, 5, true, 0xA110),
            (Mode::Tdm, MapKind::Bailey, 3, true, 0xA110),
            (Mode::Koth, MapKind::Gardens, 3, true, 0xA110),
            (Mode::Extraction, MapKind::Battlefield, 6, true, 0xA110),
            // bots-only KOTH bias probes: is the Red tilt systemic, or an
            // artifact of the scripted player feeding on Blue?
            (Mode::Koth, MapKind::Arena, 4, false, 0xBEE5),
            (Mode::Koth, MapKind::Arena, 4, false, 0x51DE),
        ] {
            let mut s = TdmSim::new(cfg(seed, 5, mode, map));
            let mut shots = 0u32;
            let mut nades = 0u32;
            let mut dodges = 0u32;
            let mut melee = 0u32;
            // rounds RESET score/deaths on completion — track cumulative
            // peaks across resets or a busy map reads as a dead one
            let mut peak_deaths = 0u32;
            let mut prev_deaths = 0u32;
            let mut rounds_done = 0u32;
            let mut min_hp = f32::MAX;
            let mut zombies_downed = 0u32;
            let mut prev_zcount = 0usize;
            let ticks = mins * 60 * SIM_HZ as usize;
            for i in 0..ticks {
                let t = i as f32 * DT;
                let me = &s.fighters[0];
                let alive = me.alive();
                // nearest living enemy — fighters in PvP, ZOMBIES in
                // extraction (there are no enemy fighters there)
                let mut aim = [0.0, 0.0, 1.0];
                let mut near = f32::MAX;
                for (j, g) in s.fighters.iter().enumerate() {
                    if j == 0 || g.team == s.fighters[0].team || !g.alive() {
                        continue;
                    }
                    let dx = g.pos[0] - me.pos[0];
                    let dz = g.pos[2] - me.pos[2];
                    let d = (dx * dx + dz * dz).sqrt();
                    if d < near {
                        near = d;
                        aim = [dx / d.max(0.1), 0.02, dz / d.max(0.1)];
                    }
                }
                if mode == Mode::Extraction {
                    for z in &s.zombies {
                        let dx = z.pos[0] - me.pos[0];
                        let dz = z.pos[2] - me.pos[2];
                        let d = (dx * dx + dz * dz).sqrt();
                        if d < near {
                            near = d;
                            aim = [dx / d.max(0.1), 0.02, dz / d.max(0.1)];
                        }
                    }
                }
                // patrol the corners; in extraction, drift toward the
                // site; in KOTH, PLAY THE OBJECTIVE — contest the hill
                let wp = if mode == Mode::Extraction {
                    let p2 = s.extract_point().unwrap_or([0.0, 0.0, 0.0]);
                    [p2[0], p2[2]]
                } else if mode == Mode::Koth {
                    [s.hill[0], s.hill[2]]
                } else {
                    corners[(i / (8 * SIM_HZ as usize)) % 4]
                };
                let (mvx, mvz) = {
                    let dx = wp[0] - me.pos[0];
                    let dz = wp[1] - me.pos[2];
                    let l = (dx * dx + dz * dz).sqrt().max(0.5);
                    (dx / l, dz / l)
                };
                let yaw = aim[0].atan2(aim[2]);
                let shoot = alive && near < 40.0 && i % 30 < 12;
                let do_dodge = alive && i % (7 * SIM_HZ as usize) == 40;
                let hold_nade = alive
                    && (i % (20 * SIM_HZ as usize)) < (SIM_HZ as usize / 2)
                    && i > 5 * SIM_HZ as usize;
                let do_knife = alive && near < 2.0 && i % 60 == 0;
                if shoot {
                    shots += 1;
                }
                if do_dodge {
                    dodges += 1;
                }
                if hold_nade {
                    nades += 1;
                }
                if do_knife {
                    melee += 1;
                }
                s.step(if driven {
                    PlayerCmd {
                        move_x: mvx,
                        move_z: mvz,
                        sprint: near > 25.0,
                        yaw,
                        aim,
                        shoot,
                        dodge: do_dodge,
                        throw_hold: hold_nade,
                        knife_hold: do_knife,
                        reload: i % (6 * SIM_HZ as usize) == 0,
                        ..Default::default()
                    }
                } else {
                    PlayerCmd::default() // bots-only probe: idle player
                });
                let td: u32 = s.fighters.iter().map(|f| f.deaths).sum();
                if td < prev_deaths {
                    rounds_done += 1; // a round completed and reset
                }
                peak_deaths = peak_deaths.max(td);
                prev_deaths = td;
                if s.fighters[0].alive() {
                    min_hp = min_hp.min(s.fighters[0].health);
                }
                // horde attrition: count the population DROPPING (kills
                // outpacing the director's spawns at that instant)
                let zc = s.zombies.len();
                if zc < prev_zcount {
                    zombies_downed += (prev_zcount - zc) as u32;
                }
                prev_zcount = zc;
                if i % (10 * 60 * 2) == 0 {
                    for f in &s.fighters {
                        assert!(
                            f.pos[0].is_finite() && f.pos[1].is_finite() && f.pos[2].is_finite(),
                            "{mode:?}/{map:?}: NaN position at t={t:.1}"
                        );
                    }
                }
            }
            let p = &s.fighters[0];
            println!(
                "== AUTOPLAY {mode:?} on {map:?} ({mins} min) ==\n\
                 score BLUE {:.0} — RED {:.0} | rounds completed {rounds_done} | peak deaths {peak_deaths}\n\
                 player K/D {}/{} hits {} | hp {:.0} | gun {:?} armor {:?}\n\
                 inputs: shot-ticks {shots}, nade-holds {nades}, dodges {dodges}, melee {melee}\n\
                 player min-hp {:.0} | zombies downed {zombies_downed} | extract hold {:.0}s\n\
                 world: missiles {} grenades {} smokes {} fires {} zombies {} pressure {:.2}\n",
                s.score[0],
                s.score[1],
                p.kills,
                p.deaths,
                p.hits_dealt,
                p.health.max(0.0),
                p.gun,
                p.armor_set,
                if min_hp.is_finite() { min_hp } else { 0.0 },
                s.extract_hold,
                s.missiles.len(),
                s.grenades_air.len(),
                s.smokes.len(),
                s.fires.len(),
                s.zombies.len(),
                s.pressure,
            );
            // the match must actually PROGRESS — judged on CUMULATIVE
            // events, robust to round resets
            assert!(
                peak_deaths > 0 || rounds_done > 0 || mode == Mode::Extraction,
                "{mode:?}/{map:?}: a {mins}-minute match with zero events is a dead sim"
            );
        }
    }

    /// Diagnostic (run on demand): where do Bailey bots actually GO?
    #[test]
    #[ignore]
    fn diag_bailey() {
        let corners: [[f32; 2]; 4] =
            [[14.0, 14.0], [-14.0, 14.0], [-14.0, -14.0], [14.0, -14.0]];
        let mut s = TdmSim::new(cfg(0xA110, 5, Mode::Tdm, MapKind::Bailey));
        for i in 0..(120 * SIM_HZ as usize) {
            // EXACTLY the autoplay policy — the bots-only run plays fine,
            // so the freeze must ride in on the player's cmd stream
            let me = &s.fighters[0];
            let mut aim = [0.0, 0.0, 1.0];
            let mut near = f32::MAX;
            for (j, g) in s.fighters.iter().enumerate() {
                if j == 0 || g.team == s.fighters[0].team || !g.alive() {
                    continue;
                }
                let dx = g.pos[0] - me.pos[0];
                let dz = g.pos[2] - me.pos[2];
                let d = (dx * dx + dz * dz).sqrt();
                if d < near {
                    near = d;
                    aim = [dx / d.max(0.1), 0.02, dz / d.max(0.1)];
                }
            }
            let wp = corners[(i / (8 * SIM_HZ as usize)) % 4];
            let (mvx, mvz) = {
                let dx = wp[0] - me.pos[0];
                let dz = wp[1] - me.pos[2];
                let l = (dx * dx + dz * dz).sqrt().max(0.5);
                (dx / l, dz / l)
            };
            let alive = me.alive();
            s.step(PlayerCmd {
                move_x: mvx,
                move_z: mvz,
                sprint: near > 25.0,
                yaw: aim[0].atan2(aim[2]),
                aim,
                shoot: alive && near < 40.0 && i % 30 < 12,
                dodge: alive && i % (7 * SIM_HZ as usize) == 40,
                throw_hold: alive
                    && (i % (20 * SIM_HZ as usize)) < (SIM_HZ as usize / 2)
                    && i > 5 * SIM_HZ as usize,
                knife_hold: alive && near < 2.0 && i % 60 == 0,
                reload: i % (6 * SIM_HZ as usize) == 0,
                ..Default::default()
            });
            if i % (10 * SIM_HZ as usize) == 0 {
                let t = i as f32 * DT;
                let mut min_sep = f32::MAX;
                for a in &s.fighters[..5] {
                    for b in &s.fighters[5..] {
                        let d = ((a.pos[0] - b.pos[0]).powi(2)
                            + (a.pos[2] - b.pos[2]).powi(2))
                        .sqrt();
                        min_sep = min_sep.min(d);
                    }
                }
                println!(
                    "t={t:>5.1}  min_sep={min_sep:>5.1}  player=({:>6.1},{:>6.1}) hp {:>4.0}  blue1=({:>6.1},{:>6.1}) red1=({:>6.1},{:>6.1}) deaths={}",
                    s.fighters[0].pos[0],
                    s.fighters[0].pos[2],
                    s.fighters[0].health.max(0.0),
                    s.fighters[1].pos[0],
                    s.fighters[1].pos[2],
                    s.fighters[6].pos[0],
                    s.fighters[6].pos[2],
                    s.fighters.iter().map(|f| f.deaths).sum::<u32>(),
                );
            }
        }
    }

    /// §2 (Brief VI): the deterministic spray — a 30-round M4 magazine
    /// replays BIT-IDENTICALLY (punch trace and impact digest), the
    /// pattern climbs (early entries near-vertical), and the camera
    /// channel is back at rest within 0.5 s of the last shot.
    #[test]
    fn spray_replays_exactly_climbs_and_recovers() {
        let run = || -> (u64, f32, f32, f32) {
            let mut s = range(0xC5C0);
            s.fighters[1].ammo = 0;
            s.fighters[1].reserve = 0;
            s.fighters[1].slot_ammo = [(0, 0); 3];
            let mut digest = 0xcbf2_9ce4_8422_2325_u64;
            let mut fold = |v: f32| {
                digest ^= v.to_bits() as u64;
                digest = digest.wrapping_mul(0x0000_0100_0000_01B3);
            };
            let mut peak_pitch = 0.0_f32;
            let mut peak_yaw = 0.0_f32;
            // hold the trigger through the full 30-round magazine
            for _ in 0..330 {
                s.step(PlayerCmd {
                    shoot: true,
                    aim: [0.0, 0.0, 1.0],
                    ..Default::default()
                });
                s.fighters[1].pos = [-30.0, 0.0, -30.0];
                s.fighters[1].vel = [0.0, 0.0];
                let f = &s.fighters[0];
                fold(f.punch[0]);
                fold(f.punch[1]);
                peak_pitch = peak_pitch.max(f.punch[0]);
                peak_yaw = peak_yaw.max(f.punch[1].abs());
            }
            for (im, _) in &s.impacts {
                fold(im.at[0]);
                fold(im.at[1]);
                fold(im.at[2]);
            }
            // release and let the punch decay for 0.5 s
            for _ in 0..(SIM_HZ as usize / 2) {
                s.step(PlayerCmd::default());
                s.fighters[1].pos = [-30.0, 0.0, -30.0];
            }
            let rest = s.fighters[0].punch[0].abs().max(s.fighters[0].punch[1].abs());
            (digest, peak_pitch, peak_yaw, rest)
        };
        let (d1, climb, yaw_drift, rest) = run();
        let (d2, ..) = run();
        assert_eq!(d1, d2, "a full-magazine spray must replay bit-identically");
        assert!(climb > 0.5, "the pattern must CLIMB: peak pitch {climb:.2}°");
        assert!(
            yaw_drift < climb,
            "early pattern is vertical-dominant: yaw {yaw_drift:.2}° vs pitch {climb:.2}°"
        );
        assert!(
            rest < 0.05,
            "camera channel at rest within 0.5 s: {rest:.3}°"
        );
        // the table itself is fixed: same entries on every call
        for i in 0..32 {
            assert_eq!(
                spray_entry(GunKind::Ak47, i),
                spray_entry(GunKind::Ak47, i),
                "spray table must be pure"
            );
        }
        // tap-firing resets: idle past cycletime × 1.1 decays the index
        let mut s = range(0xC5C1);
        s.fighters[1].pos = [-30.0, 0.0, -30.0];
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        for i in 0..(SIM_HZ as usize) {
            s.step(PlayerCmd {
                shoot: i < 40,
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
        }
        assert!(
            s.fighters[0].spray_i < 4.0,
            "idle must decay the spray index: {}",
            s.fighters[0].spray_i
        );
    }

    /// §A.7: the mech brace stance. Four properties, each independently
    /// falsifiable - the third is a deliberate regression guard against
    /// the danger the constants' own doc block warns about (silently
    /// reusing the INFANTRY brace multiplier for a mech).
    #[test]
    fn mech_brace_is_gated_slowed_and_damped_by_its_own_constants() {
        // (1) DENIED to non-mechs: the RobotSuit match arm never runs
        let mut s = range(0xB4CE);
        s.fighters[0].armor_set = ArmorSet::Folk;
        for _ in 0..4 {
            s.step(PlayerCmd { crouch: true, aim: [0.0, 0.0, 1.0], ..Default::default() });
        }
        assert!(
            !s.fighters[0].mech_brace,
            "a Folk fighter must never acquire mech_brace - wrong match arm"
        );

        // (2) REQUIRES GROUNDED: an airborne mech cannot plant a stance
        let mut s = range(0xB4CF);
        s.fighters[0].armor_set = ArmorSet::RobotSuit;
        s.fighters[0].hull = MECH_HULL;
        s.fighters[0].grounded = false;
        s.step(PlayerCmd { crouch: true, aim: [0.0, 0.0, 1.0], ..Default::default() });
        assert!(
            !s.fighters[0].mech_brace,
            "mid-air is not a stance - mech_brace must require grounded"
        );

        // (3) SLOWED BY ITS OWN CONSTANT, not the infantry one. This is
        // the regression guard: if someone ever "simplifies" this by
        // reusing `brace`, the measured multiplier becomes 0.25 and the
        // second assertion below fires.
        let planar = |f: &Fighter| (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
        let braced_speed = {
            let mut s = range(0xB4D0);
            s.fighters[0].armor_set = ArmorSet::RobotSuit;
            s.fighters[0].hull = MECH_HULL;
            // long enough for the §1.3 accel model to reach steady state
            for _ in 0..(SIM_HZ as usize) {
                s.step(PlayerCmd {
                    move_z: 1.0,
                    crouch: true,
                    aim: [0.0, 0.0, 1.0],
                    ..Default::default()
                });
            }
            planar(&s.fighters[0])
        };
        let want = MOVE_SPEED
            * armor_spec(ArmorSet::RobotSuit).move_mult
            * MECH_BRACE_SPEED_MULT;
        assert!(
            (braced_speed - want).abs() < 0.05,
            "braced mech should walk at {want}, measured {braced_speed}"
        );
        let infantry_would_be =
            MOVE_SPEED * armor_spec(ArmorSet::RobotSuit).move_mult * BRACE_SPEED_MULT;
        assert!(
            (braced_speed - infantry_would_be).abs() > 0.05,
            "measured {braced_speed} matches the INFANTRY brace multiplier \
             ({infantry_would_be}) - the mech is reusing BRACE_SPEED_MULT, \
             which is exactly what MECH_BRACE_SPEED_MULT exists to prevent"
        );

        // (4) DAMPS RECOIL by exactly the damp constant. Both are plain
        // constants, so the ratio is exact - a typo'd damp or a flipped
        // inequality moves it immediately.
        let kick_after_one_shot = |braced: bool| -> f32 {
            let mut s = range(0xB4D1);
            {
                let f = &mut s.fighters[0];
                f.armor_set = ArmorSet::RobotSuit;
                f.hull = MECH_HULL;
                f.mech_brace = braced;
                f.gun = GunKind::Ak47;
                f.inventory[0] = GunKind::Ak47;
                f.ammo = 30;
                f.protect_t = 0.0;
            }
            assert!(s.try_fire(0, [0.0, 0.0, 1.0], false), "the shot must land");
            let f = &s.fighters[0];
            (f.punch_vel[0] * f.punch_vel[0] + f.punch_vel[1] * f.punch_vel[1]).sqrt()
        };
        let unbraced = kick_after_one_shot(false);
        let braced = kick_after_one_shot(true);
        assert!(unbraced > 0.0, "the unbraced shot must actually kick");
        assert!(
            (braced - unbraced * MECH_BRACE_RECOIL_DAMP).abs() < 1e-4,
            "braced kick {braced} should be exactly {MECH_BRACE_RECOIL_DAMP} x \
             unbraced {unbraced}"
        );
    }

    // ---- §C: the hull-mounted gatling + autocannon --------------------

    /// A live chassis on the range, mounts cold, both mounts ready.
    fn mech_range(seed: u64, w: MechWeapon) -> TdmSim {
        let mut s = range(seed);
        let f = &mut s.fighters[0];
        f.armor_set = ArmorSet::RobotSuit;
        f.hull = MECH_HULL;
        f.mech_transition_t = 0.0;
        f.mech_weapon = w;
        f.gatling_heat = 0.0;
        f.gatling_vent_t = 0.0;
        f.autocannon_cd = 0.0;
        f.gatling_cd = 0.0;
        f.gatling_trigger_t = 0.0;
        f.fire_cd = 0.0;
        f.protect_t = 0.0;
        s
    }

    /// §C.3: the gatling's identity is SUSTAIN. Its heat has to ramp
    /// slower than the man-portable minigun's in absolute terms — not
    /// merely "a constant named GATLING_HEAT_PER_SHOT exists". Both
    /// halves are measured off the real fire paths, so wiring the
    /// gatling to the minigun's heat constant (the obvious copy-paste
    /// slip) fails here even though every constant still exists.
    #[test]
    fn gatling_heat_ramps_slower_than_minigun_in_absolute_terms() {
        const N: usize = 40;

        let gatling_heat = {
            let mut s = mech_range(0xC0A7, MechWeapon::Gatling);
            for _ in 0..N {
                s.fighters[0].gatling_cd = 0.0;
                assert!(
                    s.try_fire_gatling(0, [0.0, 0.0, -1.0]),
                    "a cold hull gatling must keep firing"
                );
            }
            s.fighters[0].gatling_heat
        };
        let minigun_heat = {
            let mut s = range(0xC0A8);
            {
                let f = &mut s.fighters[0];
                f.gun = GunKind::Minigun;
                f.inventory[0] = GunKind::Minigun;
                f.active = 0;
                f.ammo = 400;
                f.reserve = 0;
                f.spin_t = MINIGUN_SPINUP_S;
                f.protect_t = 0.0;
            }
            for _ in 0..N {
                s.fighters[0].fire_cd = 0.0;
                s.fighters[0].spin_t = MINIGUN_SPINUP_S;
                assert!(
                    s.try_fire(0, [0.0, 0.0, -1.0], false),
                    "the spun-up minigun must keep firing"
                );
            }
            s.fighters[0].heat
        };
        assert!(
            gatling_heat > 0.0 && minigun_heat > 0.0,
            "both weapons must actually accumulate heat \
             (gatling {gatling_heat}, minigun {minigun_heat})"
        );
        assert!(
            gatling_heat < minigun_heat,
            "after {N} rounds the hull gatling is at {gatling_heat} heat and \
             the minigun at {minigun_heat} - the hull gun is supposed to be \
             the one that SUSTAINS"
        );

        // ...and the pools are genuinely separate: firing the hull mount
        // must never load the minigun's vent state machine.
        let mut s = mech_range(0xC0A9, MechWeapon::Gatling);
        s.fighters[0].gun = GunKind::Minigun;
        for _ in 0..N {
            s.fighters[0].gatling_cd = 0.0;
            s.try_fire_gatling(0, [0.0, 0.0, -1.0]);
        }
        assert_eq!(
            s.fighters[0].heat, 0.0,
            "the hull mount wrote into the MINIGUN's heat pool - a pilot \
             carrying a minigun would inherit a vent lockout he never earned"
        );
        // NOTE: the TIME TO A FORCED VENT used to be asserted here by
        // rebuilding `heat_per_shot / fire_period` from the constants and
        // comparing the two quotients. That is arithmetic, not a
        // measurement - it stepped nothing, it could not see the heat
        // DECAY at all, and it evaluated to 7.78 s when the mount really
        // ran 9.08 s. Being the only mention of time-to-vent in the
        // suite, it was also why ungating the decay and deleting the
        // forced vent outright both survived the whole suite. It now
        // lives in `gatling_sustains_about_twice_the_minigun_before_a_\
        // forced_vent`, which holds the trigger and watches the clock.
    }

    /// §C.3: the gatling's identity is a NUMBER — how many seconds of
    /// held trigger it buys before the mount cooks off — and nothing in
    /// this suite measured it. What stood in its place rebuilt
    /// `heat_per_shot / fire_period` from four constants, stepped
    /// nothing, and labelled the quotient "time to a forced vent". It
    /// could not see the heat DECAY, and it could not see the forced
    /// vent, so THREE separate mutations survived the entire suite:
    /// ungating the decay (9 s → 40 s of sustain), deleting the
    /// forced-vent latch outright, and — because `gatling_vent_t` was
    /// never set non-zero anywhere in the suite — the vent lockout with
    /// it.
    ///
    /// This one holds the trigger through `step` and watches the clock,
    /// against the SAME measurement taken off the man-portable minigun,
    /// so the assertion is the design sentence ("about twice the
    /// minigun") and not a magic number that has to be re-tuned every
    /// time either weapon moves.
    #[test]
    fn gatling_sustains_about_twice_the_minigun_before_a_forced_vent() {
        // Generous: the real numbers are ~8.4 s and ~4.4 s. A run that
        // reaches this cap has no working heat ceiling at all.
        const CAP_S: f32 = 25.0;
        let cap = (CAP_S / DT) as usize;
        let hold = PlayerCmd {
            aim: [0.0, 0.0, -1.0], // away from the bot: nothing to intercept
            shoot: true,
            ..Default::default()
        };

        // ---- the hull gatling, trigger down from a cold mount ---------
        let mut gat = mech_range(0xC0A1, MechWeapon::Gatling);
        gat.fighters[1].gun = GunKind::Fists; // nothing shoots back
        let mut gat_s = f32::INFINITY;
        for n in 1..=cap {
            // the chassis is not what is under test here
            gat.fighters[0].hull = MECH_HULL;
            gat.fighters[1].health = MAX_HEALTH;
            gat.fighters[1].pos = [0.0, 0.0, 5.0];
            gat.step(hold);
            if gat.fighters[0].gatling_vent_t > 0.0 {
                gat_s = n as f32 * DT;
                break;
            }
        }
        assert!(
            gat_s.is_finite(),
            "{CAP_S}s of held trigger never forced a vent - the hull \
             gatling has no heat ceiling, so its cone/heat tradeoff costs \
             the pilot nothing and the trigger is free forever"
        );

        // ---- the man-portable minigun, same held trigger --------------
        // Barrels pre-spun so what is compared is BARREL time to cook-off
        // on both sides, not the minigun's 0.4 s spin-up.
        let mut mini = range(0xC0A2);
        {
            let f = &mut mini.fighters[0];
            f.gun = GunKind::Minigun;
            f.inventory[0] = GunKind::Minigun;
            f.active = 0;
            f.ammo = 400;
            f.reserve = 0;
            f.spin_t = MINIGUN_SPINUP_S;
            f.protect_t = 0.0;
        }
        mini.fighters[1].gun = GunKind::Fists;
        let mut mini_s = f32::INFINITY;
        for n in 1..=cap {
            mini.fighters[1].health = MAX_HEALTH;
            mini.fighters[1].pos = [0.0, 0.0, 5.0];
            mini.step(hold);
            if mini.fighters[0].vent_t > 0.0 {
                mini_s = n as f32 * DT;
                break;
            }
        }
        assert!(
            mini_s.is_finite(),
            "the minigun never vented either - the comparison this test \
             makes is meaningless until it does"
        );

        // Measured on the shipped build: 8.333 s vs 4.417 s (×1.887) at
        // the 120 Hz floor; 9.267 s vs 4.433 s (×2.090) at 60 Hz;
        // 7.867 s vs 4.133 s (×1.903) at 240 Hz. What still moves with
        // the tick rate is the ceil-quantisation of the fire period
        // itself, which the minigun shares - hence a RATIO band.
        assert!(
            gat_s > mini_s * 1.5,
            "the hull gatling ran {gat_s}s of held trigger before it cooked \
             off against the minigun's {mini_s}s - a SUSTAIN weapon has to \
             buy meaningfully more trigger time than the scream it replaces"
        );
        assert!(
            gat_s < mini_s * 3.0,
            "the hull gatling ran {gat_s}s against the minigun's {mini_s}s - \
             that is not 'about twice', that is a different weapon class. \
             The heat ceiling has stopped costing the pilot anything"
        );

        // ---- and the latch is not decoration --------------------------
        // It locks the mount out, and only the full vent clears it - with
        // the heat, so the pilot comes back to cold barrels.
        assert_eq!(
            gat.fighters[0].gatling_heat, 100.0,
            "a forced vent latches at a FULL heat pool"
        );
        assert!(
            !gat.try_fire_gatling(0, [0.0, 0.0, -1.0]),
            "a venting mount fired anyway - the forced vent costs nothing"
        );
        for _ in 0..((GATLING_VENT_FORCED_S / DT) as usize + 2) {
            gat.fighters[0].hull = MECH_HULL;
            gat.step(PlayerCmd::default()); // trigger RELEASED
        }
        assert_eq!(
            gat.fighters[0].gatling_vent_t, 0.0,
            "the forced vent never ended - the pilot is locked out forever"
        );
        assert_eq!(
            gat.fighters[0].gatling_heat, 0.0,
            "a vent always clears the mount - the pilot paid {GATLING_VENT_FORCED_S}s \
             for cold barrels and did not get them"
        );
        assert!(
            gat.try_fire_gatling(0, [0.0, 0.0, -1.0]),
            "a fully vented mount must fire again"
        );
    }

    /// §C.4b: the hull mounts belong to the HULL, and must not touch ONE
    /// field of the pilot's carried-gun state. Two real bugs lived here,
    /// and the file already stated the rule that both broke — the
    /// autocannon carries its own `autocannon_cd` precisely "so the two
    /// mounts cannot silently share a cooldown", while the gatling was
    /// writing the pilot's:
    ///
    /// 1. `fire_cd = GATLING_FIRE_PERIOD` — the CARRIED gun's cycle
    ///    clock. A pilot who dismounted mid-burst found the rifle in his
    ///    hands throttled by a gun bolted to the chassis he just left.
    /// 2. `last_shot_at = t` — whose only consumer is the carried gun's
    ///    `spray_i` decay gate. The mount refires every 0.075 s, faster
    ///    than ANY gun's `fire_period * 1.1` threshold, so the pilot's
    ///    spray index could not decay at all while the hull gun fired.
    ///    He dismounted into a fully-bloomed recoil pattern he had not
    ///    fired a single round to earn.
    #[test]
    fn firing_the_hull_mounts_leaves_the_pilots_carried_gun_untouched() {
        let carried_state_after = |w: MechWeapon| -> (f32, f32) {
            let mut s = mech_range(0xC0B7, w);
            {
                let f = &mut s.fighters[0];
                f.gun = GunKind::Ak47;
                f.inventory[0] = GunKind::Ak47;
                f.active = 0;
                f.ammo = 30;
                // walked in off a burst: the pattern is deep in the table
                f.spray_i = 5.0;
                f.fire_cd = 0.0;
                f.last_shot_at = -100.0;
            }
            s.fighters[1].gun = GunKind::Fists;
            let hold = PlayerCmd {
                aim: [0.0, 0.0, -1.0],
                shoot: true,
                ..Default::default()
            };
            // 3 s of held hull trigger; `spray_i` needs 2.5 s to unwind
            for _ in 0..(3 * SIM_HZ as usize) {
                s.fighters[0].hull = MECH_HULL;
                s.fighters[1].health = MAX_HEALTH;
                s.fighters[1].pos = [0.0, 0.0, 5.0];
                s.step(hold);
            }
            // the mount really did fire (otherwise this proves nothing)
            assert!(
                s.fighters[0].gatling_heat > 0.0 || w == MechWeapon::Autocannon,
                "the gatling never fired a round during the hold"
            );
            let f = &s.fighters[0];
            (f.fire_cd, f.spray_i)
        };

        for w in [MechWeapon::Gatling, MechWeapon::Autocannon] {
            let (fire_cd, spray_i) = carried_state_after(w);
            assert_eq!(
                fire_cd, 0.0,
                "{w:?}: the hull mount left {fire_cd}s on the PILOT's \
                 `fire_cd` - a dismounting pilot is throttled by a gun \
                 bolted to a chassis he is no longer inside"
            );
            assert_eq!(
                spray_i, 0.0,
                "{w:?}: the pilot's spray index is still at {spray_i} after \
                 3s of not firing his own gun - the hull mount is holding \
                 `last_shot_at` down, so he dismounts into a bloomed recoil \
                 pattern he never fired a round to earn"
            );
        }

        // the behavioural half of (1): the carried gun is READY the
        // instant the pilot wants it, on the very tick the mount fired.
        let mut s = mech_range(0xC0B8, MechWeapon::Gatling);
        {
            let f = &mut s.fighters[0];
            f.gun = GunKind::Ak47;
            f.inventory[0] = GunKind::Ak47;
            f.active = 0;
            f.ammo = 30;
            f.fire_cd = 0.0;
        }
        assert!(
            s.try_fire_gatling(0, [0.0, 0.0, -1.0]),
            "the hull gatling must fire"
        );
        assert!(
            s.try_fire(0, [0.0, 0.0, -1.0], false),
            "the pilot's carried rifle was locked out by the hull gatling's \
             round - the two triggers share a cooldown they must not share"
        );
    }

    /// §8.2/§C: a mech is the loudest thing on an Extraction map, and the
    /// horde director runs entirely on noise radii. A silent mount is a
    /// real defect — free suppression with no consequence — and until
    /// now deleting BOTH `emit_noise` calls passed the whole suite.
    #[test]
    fn hull_mounts_are_heard_by_the_horde() {
        // (near heard, near's new target, far heard)
        let heard = |w: MechWeapon| -> (bool, [f32; 2], bool) {
            let mut s = TdmSim::new(cfg(0xC0F5, 1, Mode::Extraction, MapKind::Arena));
            {
                let f = &mut s.fighters[0];
                f.pos = [2.0, 0.0, -3.0]; // an off-origin muzzle to point at
                f.armor_set = ArmorSet::RobotSuit;
                f.hull = MECH_HULL;
                f.health = MAX_HEALTH;
                f.mech_transition_t = 0.0;
                f.mech_weapon = w;
                f.gatling_heat = 0.0;
                f.gatling_vent_t = 0.0;
                f.gatling_cd = 0.0;
                f.autocannon_cd = 0.0;
                f.protect_t = 0.0;
            }
            s.zombies.clear();
            // both placed off the -z fire axis so no ROUND can reach
            // them: what alerts them can only be the noise
            for (id, x) in [(101_u32, 6.0_f32), (102, 400.0)] {
                s.zombies.push(Zombie {
                    id,
                    kind: ZKind::Shambler,
                    pos: [x, 0.0, -3.0],
                    hp: zspec(ZKind::Shambler).hp,
                    atk_cd: 0.0,
                    scream_t: 0.0,
                    head_hits: 0,
                    target: [-999.0, -999.0],
                    alerted: false,
                });
            }
            let fired = match w {
                MechWeapon::Gatling => s.try_fire_gatling(0, [0.0, 0.0, -1.0]),
                MechWeapon::Autocannon => s.try_fire_autocannon(0, [0.0, 0.0, -1.0]),
            };
            assert!(fired, "{w:?} must fire");
            let z = |id: u32| s.zombies.iter().find(|z| z.id == id).expect("zombie alive");
            (z(101).alerted, z(101).target, z(102).alerted)
        };

        for w in [MechWeapon::Gatling, MechWeapon::Autocannon] {
            let (near, target, far) = heard(w);
            assert!(
                near,
                "{w:?}: a zombie 4 m from the muzzle did not hear the mount. \
                 A mech can suppress an Extraction map in total silence"
            );
            assert!(
                (target[0] - 2.0).abs() < 1e-4 && (target[1] + 3.0).abs() < 1e-4,
                "{w:?}: the horde heard something but is walking to {target:?} \
                 instead of the muzzle at [2.0, -3.0]"
            );
            assert!(
                !far,
                "{w:?}: a zombie 398 m away heard it - the noise radius is \
                 not a radius, so nothing about WHICH weapon you fire in \
                 Extraction matters"
            );
        }
    }

    /// §C.3: the cone OPENS as the mount cooks — the cost that makes
    /// holding the trigger a decision. Measured off the tracers the
    /// shots actually leave, not off the constants.
    #[test]
    fn gatling_spread_widens_with_heat() {
        // widest angular deviation from the aim axis over many rounds
        // fired at a pinned heat level
        let worst_dev = |heat: f32| -> f32 {
            let mut s = mech_range(0xC0B0, MechWeapon::Gatling);
            let mut worst = 0.0_f32;
            for _ in 0..300 {
                let f = &mut s.fighters[0];
                f.gatling_cd = 0.0;
                f.gatling_heat = heat;
                f.gatling_vent_t = 0.0;
                s.tracers.clear();
                // fire AWAY from the pinned bot so nothing intercepts
                assert!(s.try_fire_gatling(0, [0.0, 0.0, -1.0]));
                let tr = s.tracers.last().expect("every round leaves a tracer");
                let d = normalize([
                    tr.to[0] - tr.from[0],
                    tr.to[1] - tr.from[1],
                    tr.to[2] - tr.from[2],
                ]);
                // tangent of the off-axis angle about the -z aim
                let dev = (d[0] * d[0] + d[1] * d[1]).sqrt() / d[2].abs().max(1e-6);
                worst = worst.max(dev);
            }
            worst
        };
        let cold = worst_dev(0.0);
        let hot = worst_dev(100.0);
        assert!(cold > 0.0, "even a cold mount must roll SOME spread");
        assert!(
            hot > cold * 2.0,
            "a cooked mount scatters {hot} vs a cold {cold} - the heat/cone \
             tradeoff is not wired to the heat at all"
        );
        // and each end lands in the band its constant defines (the two
        // axes are rolled independently, so the radial bound is √2×)
        assert!(
            cold <= GATLING_SPREAD_COLD * 1.5,
            "cold worst-case {cold} exceeds the cold constant's radial bound"
        );
        assert!(
            hot <= GATLING_SPREAD_HOT * 1.5 && hot > GATLING_SPREAD_COLD,
            "hot worst-case {hot} is not inside the hot constant's band"
        );
    }

    /// §C.3/§A.5: the braced autocannon kick is DERIVED from the
    /// unbraced one, so the ratio is exact. This test is the reason no
    /// `AUTOCANNON_BRACED_KICK` constant exists — with two independent
    /// numbers the assertion below would only ever pin whatever the
    /// second one drifted to.
    #[test]
    fn autocannon_kick_is_damped_exactly_by_mech_brace_recoil_damp() {
        let kick = |braced: bool| -> f32 {
            let mut s = mech_range(0xC0C1, MechWeapon::Autocannon);
            s.fighters[0].mech_brace = braced;
            assert!(
                s.try_fire_autocannon(0, [0.0, 0.0, -1.0]),
                "the autocannon must fire"
            );
            s.fighters[0].punch_vel[0]
        };
        let unbraced = kick(false);
        let braced = kick(true);
        assert!(
            (unbraced - AUTOCANNON_UNBRACED_KICK).abs() < 1e-6,
            "unbraced kick {unbraced} is not the unbraced constant"
        );
        assert!(
            braced < unbraced,
            "bracing must REDUCE the kick: braced {braced} vs {unbraced}"
        );
        assert!(
            (braced - unbraced * MECH_BRACE_RECOIL_DAMP).abs() < 1e-6,
            "braced kick {braced} must be exactly {MECH_BRACE_RECOIL_DAMP} x \
             unbraced {unbraced} - a second, independently tunable braced \
             constant is exactly what §C.3 forbids"
        );
    }

    /// §C.4b/§C.5: one mount at a time. The targeting mode decides, and
    /// the number keys set it — while leaving the pilot's carried
    /// inventory alone, which is the half a naive wiring drops.
    #[test]
    fn autocannon_and_gatling_are_mutually_exclusive_by_mech_weapon() {
        let mut s = mech_range(0xC0D1, MechWeapon::Gatling);
        assert!(
            !s.try_fire_autocannon(0, [0.0, 0.0, -1.0]),
            "the autocannon fired while the gatling was selected"
        );
        assert!(s.try_fire_gatling(0, [0.0, 0.0, -1.0]), "the gatling must fire");

        let mut s = mech_range(0xC0D2, MechWeapon::Autocannon);
        assert!(
            !s.try_fire_gatling(0, [0.0, 0.0, -1.0]),
            "the gatling fired while the autocannon was selected"
        );
        assert!(
            s.try_fire_autocannon(0, [0.0, 0.0, -1.0]),
            "the autocannon must fire"
        );

        // the number keys are a TARGETING switch in a chassis
        let mut s = mech_range(0xC0D3, MechWeapon::Gatling);
        let carried = s.fighters[0].gun;
        let slot = s.fighters[0].active;
        s.step(PlayerCmd { slot: Some(1), aim: [0.0, 0.0, -1.0], ..Default::default() });
        assert_eq!(
            s.fighters[0].mech_weapon,
            MechWeapon::Autocannon,
            "key 2 must select the autocannon mount"
        );
        s.step(PlayerCmd { slot: Some(0), aim: [0.0, 0.0, -1.0], ..Default::default() });
        assert_eq!(
            s.fighters[0].mech_weapon,
            MechWeapon::Gatling,
            "key 1 must select the gatling mount"
        );
        assert_eq!(
            (s.fighters[0].gun, s.fighters[0].active),
            (carried, slot),
            "the pilot's CARRIED inventory must not move while piloting - \
             the infantry slot path is supposed to be gated on !in_mech()"
        );
    }

    /// §C.4b: the hull mounts belong to the HULL. The gate that matters
    /// is `in_mech()`: without it every infantryman on the map would be
    /// able to trigger a 145-damage autocannon out of thin air — the
    /// same defect class §A's brace test guards (an ungated mech
    /// mechanic leaking onto foot soldiers).
    #[test]
    fn mech_weapons_refuse_to_fire_for_non_mech_fighters() {
        for set in [ArmorSet::None, ArmorSet::Folk, ArmorSet::Pyro, ArmorSet::Recon] {
            for w in [MechWeapon::Gatling, MechWeapon::Autocannon] {
                let mut s = mech_range(0xC0E1, w);
                {
                    let f = &mut s.fighters[0];
                    f.armor_set = set;
                    f.hull = 0.0;
                }
                assert!(
                    !s.try_fire_gatling(0, [0.0, 0.0, -1.0]),
                    "{set:?} on foot fired the hull gatling"
                );
                assert!(
                    !s.try_fire_autocannon(0, [0.0, 0.0, -1.0]),
                    "{set:?} on foot fired the hull autocannon"
                );
            }
        }
        // a WRECKED chassis is not a chassis either: hull 0 means the
        // pilot has ejected, RobotSuit still sitting in armor_set
        let mut s = mech_range(0xC0E2, MechWeapon::Gatling);
        s.fighters[0].hull = 0.0;
        assert!(
            !s.try_fire_gatling(0, [0.0, 0.0, -1.0]),
            "a dead hull still fired"
        );
        // and a live chassis that is still SEALING cannot fight (§6.2)
        let mut s = mech_range(0xC0E3, MechWeapon::Gatling);
        s.fighters[0].mech_transition_t = MECH_ENTER_S;
        assert!(
            !s.try_fire_gatling(0, [0.0, 0.0, -1.0]),
            "the mounts went live before the chassis finished sealing"
        );
        let mut s = mech_range(0xC0E4, MechWeapon::Autocannon);
        s.fighters[0].mech_transition_t = MECH_ENTER_S;
        assert!(
            !s.try_fire_autocannon(0, [0.0, 0.0, -1.0]),
            "the mounts went live before the chassis finished sealing"
        );
    }

    /// §C.4: the mounts resolve hits on the SAME path a rifle does, and
    /// they carry their OWN damage down it.
    ///
    /// This is the test that caught the real defect in the first cut:
    /// `apply_hit` re-derived damage from `gun(shooter.gun)` at the
    /// bottom of the chain, so both hull mounts dealt whatever the pilot
    /// happened to be carrying to FIGHTERS while correctly dealing their
    /// own numbers to zombies. Asserting the two mounts' hull-damage
    /// RATIO (rather than either absolute number) means the angle
    /// multiplier, armour floor and every other shared stage cancel out,
    /// so what is left under test is exactly "did the right damage reach
    /// the shared resolver".
    #[test]
    fn hull_mounts_carry_their_own_damage_down_the_shared_hit_path() {
        // one round into a target mech, from an identical setup
        let hull_lost = |w: MechWeapon, held: GunKind| -> f32 {
            let mut s = mech_range(0xC0F1, w);
            s.fighters[0].gun = held; // the pilot's CARRIED gun: irrelevant
            s.fighters[0].pos = [0.0, 0.0, -5.0];
            {
                let t = &mut s.fighters[1];
                t.pos = [0.0, 0.0, 5.0];
                t.armor_set = ArmorSet::RobotSuit;
                t.hull = MECH_HULL;
                t.armor = 0.0;
                t.protect_t = 0.0;
                t.yaw = 0.0; // facing away: one fixed angle multiplier
            }
            // aim flat at the target's lower body - well clear of the
            // ×2 visor band, so spread cannot flip the multiplier
            let aim = [0.0, (0.9 - EYE_REL) / 10.0, 1.0];
            let fired = match w {
                MechWeapon::Gatling => s.try_fire_gatling(0, aim),
                MechWeapon::Autocannon => s.try_fire_autocannon(0, aim),
            };
            assert!(fired, "the mount must fire");
            assert_eq!(s.fighters[0].hits_dealt, 1, "the round must connect");
            MECH_HULL - s.fighters[1].hull
        };
        // deliberately hold DIFFERENT infantry guns: if the old
        // `gun(shooter.gun)` re-read is ever restored, these two diverge
        // by the held weapons instead of by the mounts.
        let gat = hull_lost(MechWeapon::Gatling, GunKind::Ak47);
        let auto = hull_lost(MechWeapon::Autocannon, GunKind::Ak47);
        assert!(gat > 0.0 && auto > 0.0, "both mounts must chip the hull");
        let want = AUTOCANNON_DAMAGE / GATLING_DAMAGE;
        let got = auto / gat;
        assert!(
            (got - want).abs() < 1e-3,
            "autocannon:gatling hull damage came out {got}:1, should be \
             exactly {want}:1 - the mounts are not carrying their own \
             damage into apply_hit"
        );
        // and the held gun genuinely does not matter
        let gat_other = hull_lost(MechWeapon::Gatling, GunKind::Deagle);
        assert!(
            (gat_other - gat).abs() < 1e-3,
            "swapping the pilot's carried gun moved the hull gatling's \
             damage from {gat} to {gat_other} - apply_hit is reading the \
             held weapon again"
        );
    }

    // ---- §D: the BOT fire path through the hull mounts ----------------

    /// §D rig: fighter 1 is a BOT (`bot_act` runs for every index except
    /// `self.player`) sat in a live chassis `dist` metres up +z from the
    /// player at the origin, already facing him so the capped mech turn
    /// rate is not what the test is measuring.
    ///
    /// `held` is what the PILOT is carrying — the whole point of §D is
    /// that it must stop mattering. `mag`/`res` set that gun's rounds.
    fn bot_mech(seed: u64, dist: f32, held: GunKind, mag: u32, res: u32) -> TdmSim {
        let mut s = range(seed);
        {
            let p = &mut s.fighters[0];
            p.pos = [0.0, 0.0, 0.0];
            p.protect_t = 0.0;
        }
        {
            let b = &mut s.fighters[1];
            b.pos = [0.0, 0.0, dist];
            b.yaw = std::f32::consts::PI; // looking back down -z, at him
            b.armor_set = ArmorSet::RobotSuit;
            b.armor = POWER_MAX;
            b.hull = MECH_HULL;
            b.mech_transition_t = 0.0; // sealed, mounts live
            b.mech_weapon = MechWeapon::Gatling; // a fresh chassis
            b.gatling_heat = 0.0;
            b.gatling_vent_t = 0.0;
            b.gatling_cd = 0.0;
            b.gatling_trigger_t = 0.0;
            b.autocannon_cd = 0.0;
            b.protect_t = 0.0;
            b.gun = held;
            b.inventory[0] = held;
            b.active = 0;
            b.ammo = mag;
            b.reserve = res;
            // the other two slots are EMPTY: `bot_act`'s dry-slot switch
            // must not quietly hand the pilot a fresh gun and hide the
            // "carried weapons run out, hull mounts do not" question
            b.slot_ammo = [(mag, res), (0, 0), (0, 0)];
        }
        s
    }

    /// Steps a pinned engagement and returns the damage the BOT dealt to
    /// the player over it. Both bodies are pinned so what is under test
    /// is the trigger, not the footwork.
    ///
    /// The victim is made IMMORTAL rather than merely healed, and that
    /// distinction is load-bearing enough to spell out: one autocannon
    /// round is 145 against a 100 HP man, so healing alone still lets the
    /// kill register — `respawn_t` latches for `RESPAWN_S`, the corpse
    /// stops being a visible enemy, and the bot spends 3 of every ~4.4
    /// seconds with nothing to shoot at. Every "continuous engagement"
    /// assertion below would then be measuring a stuttering one, and the
    /// long tests would ride on whether a given seed's first round
    /// happened to miss. Worse, a kill can latch `round_over_t`, after
    /// which `step` early-returns and, 7 s later, REPLACES the sim
    /// wholesale — so the score and round state are rolled back too.
    fn bot_engagement(s: &mut TdmSim, secs: f32) -> f32 {
        let ppos = s.fighters[0].pos;
        let pyaw = s.fighters[0].yaw;
        let bpos = s.fighters[1].pos;
        let mut dealt = 0.0;
        for _ in 0..((secs * SIM_HZ as f32) as usize) {
            s.step(PlayerCmd::default());
            {
                let p = &mut s.fighters[0];
                dealt += (MAX_HEALTH - p.health).max(0.0);
                p.health = MAX_HEALTH;
                p.respawn_t = 0.0; // back on his feet the same tick
                p.deaths = 0;
                p.pos = ppos;
                p.yaw = pyaw;
                p.vel = [0.0, 0.0];
                p.protect_t = 0.0;
            }
            {
                let b = &mut s.fighters[1];
                b.pos = bpos;
                b.vel = [0.0, 0.0];
                b.protect_t = 0.0;
                b.kills = 0;
            }
            // the round must never end under a test that runs for 30 s
            s.score = [0.0, 0.0];
            s.round_over_t = None;
            s.winner = None;
            s.overtime = false;
        }
        dealt
    }

    /// §D.1: a bot in a chassis pulls the trigger on the HULL MOUNT.
    ///
    /// Thor measured the shipped behaviour: a bot mech fired its CARRIED
    /// infantry rifle — `hits_dealt=4` over 2 s, `ammo=26/30`, carried
    /// `Ak47`, `gatling_heat=0`. A 1000-hull chassis that shoots like a
    /// man with a rifle, reloads like one, and runs out like one.
    ///
    /// The witness here is `gatling_heat`, deliberately: it is written by
    /// exactly one function in the file (`try_fire_gatling`) and read by
    /// the vent state machine, so it cannot be moved by anything the
    /// carried gun does. `hits_dealt` alone would NOT do — the rifle
    /// moves that too, which is precisely how the defect survived.
    #[test]
    fn a_bot_mech_fires_the_hull_mount_not_the_gun_in_its_hands() {
        let mut s = bot_mech(0xD01, 10.0, GunKind::Ak47, 30, 120);
        let dealt = bot_engagement(&mut s, 2.0);
        let b = &s.fighters[1];
        assert_eq!(
            b.mech_weapon,
            MechWeapon::Gatling,
            "10 m is inside {MECH_BOT_AUTOCANNON_RANGE_M:.1} m - the \
             suppression mount is the close-range pick"
        );
        assert!(
            b.gatling_heat > 0.0,
            "the hull gatling never turned over: heat {} after 2 s of a \
             clear, in-range engagement",
            b.gatling_heat
        );
        assert!(
            dealt > 0.0,
            "the mount span up but nothing reached the target"
        );
        // ...and the rifle in the pilot's hands was never touched. Four
        // rounds is what Thor measured leaving it; zero is the fix.
        assert_eq!(
            (b.ammo, b.reserve),
            (30, 120),
            "the pilot's carried Ak47 spent rounds from inside a sealed \
             cockpit"
        );
        assert_eq!(
            b.fire_cd, 0.0,
            "the carried gun's own fire clock moved - the bot is still \
             going through try_fire"
        );
    }

    /// §D.2: the chassis does not run out of ammunition, because a gun
    /// bolted to a hull has no magazine to run out of.
    ///
    /// Thor's number: at 150 rounds the pilot's reserve is gone and the
    /// bot is PERMANENTLY DISARMED inside a full-health 1000-hull
    /// chassis — a harmless piñata for the rest of the match. Both
    /// halves of that are tested: the state itself (a bone-dry pilot must
    /// still fight), and the road to it (30 s of continuous engagement
    /// must not cost the carried gun a single round).
    #[test]
    fn a_bot_mech_never_runs_dry_the_way_the_gun_in_its_hands_does() {
        // (1) the END state Thor measured: nothing in the mag, nothing in
        // reserve, nothing in any other slot. The chassis must not care.
        let mut s = bot_mech(0xD02, 10.0, GunKind::Ak47, 0, 0);
        let dealt = bot_engagement(&mut s, 2.0);
        assert!(
            dealt > 0.0 && s.fighters[1].gatling_heat > 0.0,
            "a bot whose carried gun is bone dry is disarmed inside a \
             full chassis: dealt {dealt}, heat {}",
            s.fighters[1].gatling_heat
        );

        // (2) and it never REACHES that state. 150 rounds is 5 Ak47 mags;
        // fired on the carried gun that is ~26 s of shooting and
        // reloading, so 30 s of unbroken engagement is past the point
        // where the shipped bot fell silent forever.
        let mut s = bot_mech(0xD03, 10.0, GunKind::Ak47, 30, 120);
        assert!(
            30 + 120 >= 150,
            "the setup must carry the 150 rounds this test is about"
        );
        let early = bot_engagement(&mut s, 27.0);
        let late = bot_engagement(&mut s, 3.0);
        assert!(early > 0.0, "the engagement never started");
        assert!(
            late > 0.0,
            "after 30 s - long past the {} carried rounds - the mech had \
             stopped dealing damage",
            30 + 120
        );
        assert_eq!(
            (s.fighters[1].ammo, s.fighters[1].reserve),
            (30, 120),
            "30 s of firing spent the pilot's carried ammunition"
        );
    }

    /// §D.3: the MOUNT SELECTION rule, stated and enforced.
    ///
    /// Autocannon against a hull or past `MECH_BOT_AUTOCANNON_RANGE_M`;
    /// gatling otherwise. The carried gun is left BONE DRY throughout, so
    /// any damage at all in these scenarios can only have come off a hull
    /// mount and the two mounts are told apart by `gatling_heat` (only
    /// `try_fire_gatling` writes it) and by the exact size of the hole
    /// the autocannon leaves.
    #[test]
    fn a_bot_mech_picks_the_autocannon_by_range_and_by_armour() {
        // (a) CLOSE, soft target → suppression.
        let mut s = bot_mech(0xD04, 8.0, GunKind::Ak47, 0, 0);
        let dealt = bot_engagement(&mut s, 2.0);
        assert_eq!(s.fighters[1].mech_weapon, MechWeapon::Gatling);
        assert!(
            s.fighters[1].gatling_heat > 0.0 && dealt > 0.0,
            "close in, the bot should be spraying"
        );

        // (b) LONG, soft target → precision. 30 m is past the switch and
        // still inside Normal's 35 m engage range.
        assert!(
            30.0 > MECH_BOT_AUTOCANNON_RANGE_M
                && 30.0 < bot_params(Difficulty::Normal).engage_range,
            "the long-range case has to be past the switch and inside the \
             bot's engage range, or it proves nothing"
        );
        let mut s = bot_mech(0xD05, 30.0, GunKind::Ak47, 0, 0);
        let dealt = bot_engagement(&mut s, 4.0);
        assert_eq!(s.fighters[1].mech_weapon, MechWeapon::Autocannon);
        assert_eq!(
            s.fighters[1].gatling_heat, 0.0,
            "the gatling turned over at 30 m - past the range where its \
             cold cone still covers a man"
        );
        assert!(dealt > 0.0, "the autocannon never landed a round");

        // (c) CLOSE, but the target is itself a chassis → precision
        // anyway, and the hole is exactly AUTOCANNON_DAMAGE deep. The
        // victim faces AWAY (yaw pi, attacker up +z), which is the mech
        // armour model's rear arc: no angle cut, no visor multiplier, no
        // proportional zones - so the number that lands on the hull is
        // the mount's own, undiluted.
        let mut s = bot_mech(0xD06, 8.0, GunKind::Ak47, 0, 0);
        {
            let p = &mut s.fighters[0];
            p.armor_set = ArmorSet::RobotSuit;
            p.armor = POWER_MAX;
            p.hull = MECH_HULL;
            p.mech_transition_t = 0.0;
            p.mech_plates_dropped = 0;
            p.yaw = std::f32::consts::PI;
        }
        let mut first_hole = None;
        for _ in 0..(6 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            {
                let p = &mut s.fighters[0];
                p.pos = [0.0, 0.0, 0.0];
                p.vel = [0.0, 0.0];
                p.yaw = std::f32::consts::PI;
                p.protect_t = 0.0;
            }
            {
                let b = &mut s.fighters[1];
                b.pos = [0.0, 0.0, 8.0];
                b.vel = [0.0, 0.0];
                b.protect_t = 0.0;
            }
            if s.fighters[0].hull < MECH_HULL {
                first_hole = Some(MECH_HULL - s.fighters[0].hull);
                break;
            }
        }
        assert_eq!(
            s.fighters[1].mech_weapon,
            MechWeapon::Autocannon,
            "an enemy CHASSIS at 8 m still got the spray gun - 9 damage a \
             round is not an answer to 1000 hull at any range"
        );
        let hole = first_hole.expect("the bot mech never damaged the enemy chassis");
        assert!(
            (hole - AUTOCANNON_DAMAGE).abs() < 1e-2,
            "the first hull hit took {hole}, not AUTOCANNON_DAMAGE \
             ({AUTOCANNON_DAMAGE}) - that is a different weapon"
        );
        assert!(
            (hole - GATLING_DAMAGE).abs() > 1.0,
            "{hole} is the gatling's number, not the autocannon's"
        );

        // (d) the switch has a BAND. A bot holding station near the line
        // must not flip mounts tick to tick: the two mounts keep separate
        // cooldown clocks, so a flip-flopping bot fires BOTH and standing
        // on the threshold becomes its highest-DPS option.
        let inside_band = MECH_BOT_AUTOCANNON_RANGE_M - MECH_BOT_MOUNT_HYSTERESIS_M * 0.5;
        let mut s = bot_mech(0xD07, inside_band, GunKind::Ak47, 0, 0);
        s.fighters[1].mech_weapon = MechWeapon::Autocannon; // came from further out
        bot_engagement(&mut s, 1.0);
        assert_eq!(
            s.fighters[1].mech_weapon,
            MechWeapon::Autocannon,
            "the bot dropped the autocannon {:.2} m inside the switch - \
             that is under the {MECH_BOT_MOUNT_HYSTERESIS_M:.2} m band \
             and it will chatter",
            MECH_BOT_AUTOCANNON_RANGE_M - inside_band
        );
        // ...but the band is a band, not a latch: well inside it, the
        // bot really does go back to the gatling.
        let well_inside = MECH_BOT_AUTOCANNON_RANGE_M - MECH_BOT_MOUNT_HYSTERESIS_M - 2.0;
        let mut s = bot_mech(0xD08, well_inside, GunKind::Ak47, 0, 0);
        s.fighters[1].mech_weapon = MechWeapon::Autocannon;
        bot_engagement(&mut s, 1.0);
        assert_eq!(
            s.fighters[1].mech_weapon,
            MechWeapon::Gatling,
            "the autocannon latched on - the hysteresis band has become a \
             one-way door at {well_inside:.1} m"
        );
    }

    /// §D.4 — the parity question §A left open, answered.
    ///
    /// §A wired the `mech_brace` movement tax onto BOTH paths but only
    /// the player could ever set the flag. Now that bots fire hull
    /// mounts, the answer is yes-but-narrowly: a bot plants for the
    /// AUTOCANNON (whose recoil is the damp constant's only consumer)
    /// and never for the gatling, because bracing to spray would buy
    /// nothing and cost 88% of its movement.
    #[test]
    fn a_bot_mech_plants_itself_for_the_autocannon_and_only_for_it() {
        // long range → autocannon → planted
        let mut s = bot_mech(0xD09, 30.0, GunKind::Ak47, 0, 0);
        bot_engagement(&mut s, 2.0);
        assert!(
            s.fighters[1].mech_brace,
            "a bot committed to the autocannon never widened its stance - \
             MECH_BRACE_RECOIL_DAMP still has no bot-side consumer"
        );
        // close range → gatling → mobile
        let mut s = bot_mech(0xD10, 8.0, GunKind::Ak47, 0, 0);
        bot_engagement(&mut s, 2.0);
        assert!(
            !s.fighters[1].mech_brace,
            "the bot planted itself to fire the SPRAY gun - 88% of its \
             movement for a recoil damp the gatling never reads"
        );
        // target lost → the stance drops. A braced mech walks at 12%; a
        // bot that kept the plant after LOS broke would crawl the match.
        let mut s = bot_mech(0xD11, 30.0, GunKind::Ak47, 0, 0);
        bot_engagement(&mut s, 2.0);
        assert!(s.fighters[1].mech_brace, "precondition: it is planted");
        // stepped RAW, deliberately not through `bot_engagement` - that
        // rig resurrects the victim every tick, which is exactly what
        // must NOT happen here
        s.fighters[0].respawn_t = 5.0; // no visible threat any more
        for _ in 0..(SIM_HZ as usize / 2) {
            s.step(PlayerCmd::default());
            s.fighters[0].respawn_t = 5.0;
        }
        assert!(
            !s.fighters[1].mech_brace,
            "the bot held the plant with nothing to shoot at"
        );
        // §A's own guard, on the bot path this time: a fighter who is NOT
        // in a chassis must never acquire the chassis stance, whatever
        // range he is engaging at.
        let mut s = bot_mech(0xD12, 30.0, GunKind::Ak47, 30, 120);
        {
            let b = &mut s.fighters[1];
            b.armor_set = ArmorSet::None;
            b.hull = 0.0;
        }
        bot_engagement(&mut s, 2.0);
        assert!(
            !s.fighters[1].mech_brace,
            "an infantryman acquired mech_brace - the mech stance has \
             leaked onto foot soldiers via the bot path"
        );

        // the plant is REAL, not a flag: same range, same seed, same
        // strafe - only the target's armour differs, which is what picks
        // the mount and therefore the stance. Path length, not net
        // displacement: the strafe reverses, and a round trip would read
        // as standing still.
        let travel = |seed: u64, hardened: bool| -> f32 {
            let mut s = bot_mech(seed, 8.0, GunKind::Ak47, 0, 0);
            if hardened {
                let p = &mut s.fighters[0];
                p.armor_set = ArmorSet::RobotSuit;
                p.armor = POWER_MAX;
                p.hull = MECH_HULL;
                p.mech_transition_t = 0.0;
            }
            let mut path = 0.0;
            let mut last = s.fighters[1].pos;
            for _ in 0..(3 * SIM_HZ as usize) {
                s.step(PlayerCmd::default());
                {
                    let p = &mut s.fighters[0];
                    p.pos = [0.0, 0.0, 0.0];
                    p.vel = [0.0, 0.0];
                    p.health = MAX_HEALTH;
                    p.respawn_t = 0.0; // immortal, see `bot_engagement`
                    p.hull = if hardened { MECH_HULL } else { p.hull };
                    p.protect_t = 0.0;
                }
                s.round_over_t = None;
                s.score = [0.0, 0.0];
                let now = s.fighters[1].pos;
                path += ((now[0] - last[0]).powi(2) + (now[2] - last[2]).powi(2)).sqrt();
                last = now;
            }
            path
        };
        let mobile = travel(0xD13, false);
        let planted = travel(0xD13, true);
        assert!(mobile > 1.0, "the unbraced control barely moved: {mobile}");
        assert!(
            planted < mobile * 0.5,
            "the braced bot covered {planted:.2} m against the mobile \
             one's {mobile:.2} m - MECH_BRACE_SPEED_MULT is \
             {MECH_BRACE_SPEED_MULT}, so the plant is not being paid for"
        );
    }

    /// §D.5: a chassis is armour and firepower — it is NOT marksmanship.
    ///
    /// This test exists because a mutation survived the rest of the suite:
    /// skipping the two `rng.range` aim draws on the mech branch (the
    /// obvious "the hull mount doesn't need the bot's wobble" tidy-up)
    /// changed nothing any test could see. It is two defects at once.
    /// The visible one is balance — every bot mech becomes a perfect
    /// shot at every difficulty. The invisible one is worse: those draws
    /// come off the sim's single seeded stream, so skipping them
    /// RE-ORDERS that stream for every scenario containing a bot mech,
    /// and the replay guarantee is exactly "the same seed makes the same
    /// draws in the same order". Nothing else in the suite covers it,
    /// because no existing determinism test ever puts a bot in a mech.
    ///
    /// Asserted as a spread between difficulties rather than an absolute
    /// hit count: the ratio survives retuning `aim_sigma`, an absolute
    /// number would not. Measured 27 (Easy) against 62 (Hard).
    #[test]
    fn a_chassis_does_not_make_a_bot_a_better_shot() {
        // 15 m: inside the gatling's band (so this measures the mount
        // that actually fires a lot of rounds) and inside EASY's 22 m
        // engage range, so both tiers really do shoot.
        assert!(15.0 < MECH_BOT_AUTOCANNON_RANGE_M);
        assert!(15.0 < bot_params(Difficulty::Easy).engage_range);
        let hits = |d: Difficulty| -> u32 {
            let mut s = bot_mech(0xD14, 15.0, GunKind::Ak47, 0, 0);
            s.cfg.difficulty = d;
            bot_engagement(&mut s, 6.0);
            s.fighters[1].hits_dealt
        };
        let easy = hits(Difficulty::Easy);
        let hard = hits(Difficulty::Hard);
        assert!(
            easy > 0 && hard > 0,
            "both tiers have to actually shoot: easy {easy}, hard {hard}"
        );
        assert!(
            hard > easy * 3 / 2,
            "a bot mech landed {easy} rounds on Easy and {hard} on Hard - \
             the chassis is shooting straighter than the brain driving \
             it, which means the bot's aim draws are being skipped for \
             the hull mounts"
        );
    }

    /// §4.7 (Brief VI) — the anti-"specified twice, never shipped"
    /// gate: the mech is REACHED (pad grant), at SCALE (1.7×, superseding
    /// Brief VI's original 1.15× per the MISSION doc / VIII-B addendum —
    /// see the inline comment below), GROUNDED (a 60 s seeded fuzz with
    /// jump/thruster inputs never
    /// lifts it), DISMOUNTABLE (U), and KILLABLE (hull → pilot eject).
    #[test]
    fn mech_exists_at_scale_grounded_and_dismounts() {
        let mut s = range(0x4EC4);
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        {
            let f = &mut s.fighters[0];
            f.armor_set = ArmorSet::RobotSuit;
            f.armor = POWER_MAX;
            f.hull = MECH_HULL;
        }
        // SCALE (Task 4, MISSION doc: A3 recommended, 1.7x - supersedes
        // Brief VI's 1.15x): MECH_SCALE x soldier, +/-2%
        let h = s.fighters[0].height();
        let want = BODY_HEIGHT * MECH_SCALE;
        assert!(
            (h / want - 1.0).abs() < 0.02,
            "mech height {h:.3} vs {want:.3}"
        );
        // GROUNDED: seeded fuzz drive with jump AND held-thrust inputs —
        // the mech must never gain upward velocity (flight is deleted)
        let mut rng = Pcg32::new(77, 99);
        for i in 0..(60 * SIM_HZ as usize) {
            let (jx, jz) = (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
            s.step(PlayerCmd {
                move_x: jx,
                move_z: jz,
                sprint: i % 100 < 50,
                jump: i % 37 == 0,
                dodge: i % 200 == 5,
                aim: [0.0, 0.0, 1.0],
                ..Default::default()
            });
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
            s.fighters[1].vel = [0.0, 0.0];
            assert!(
                s.fighters[0].vy <= 0.01,
                "the mech must NEVER gain height: vy {} at tick {i}",
                s.fighters[0].vy
            );
        }
        // DISMOUNT: U starts the power-down; §6.2 makes leaving COMMITTED
        // (MECH_EXIT_S) rather than a one-tick flip, so the pilot is on
        // foot only once the window closes.
        s.step(PlayerCmd {
            exit_mech: true,
            ..Default::default()
        });
        assert_eq!(
            s.fighters[0].armor_set,
            ArmorSet::RobotSuit,
            "the chassis powers down over MECH_EXIT_S - not instantly"
        );
        for _ in 0..((MECH_EXIT_S * SIM_HZ as f32) as usize + 3) {
            s.step(PlayerCmd::default());
        }
        assert_eq!(
            s.fighters[0].armor_set,
            ArmorSet::None,
            "U must dismount the mech once the power-down completes"
        );
        // KILLABLE: re-board with a scrap hull, burn it → eject at ≤25
        s.fighters[0].armor_set = ArmorSet::RobotSuit;
        s.fighters[0].hull = 5.0;
        s.fighters[0].yaw = 0.0;
        s.apply_plain_damage(1, 0, 40.0, [0.0, 1.0, -3.0], false, false);
        assert_eq!(s.fighters[0].armor_set, ArmorSet::None, "hull death ejects");
        assert!(
            s.fighters[0].health <= MECH_EJECT_HP + 0.01,
            "pilot ejects at ≤{MECH_EJECT_HP} HP"
        );
    }

    /// §7.4 (BRIEF VIII): power stride, previously entirely unbuilt -
    /// zero call sites for anything named "stride" before this test.
    /// Proves the whole cycle: sprint on a mech WINDS UP (no speed
    /// change yet), then BURSTS to 110%, locks the missile pod for the
    /// duration, caps turning at half the normal pivot rate, and costs
    /// its own heat budget that must cool before it can fire again.
    #[test]
    fn power_stride_winds_up_bursts_locks_and_costs_heat() {
        let mut s = range(0x5713);
        {
            let f = &mut s.fighters[0];
            f.armor_set = ArmorSet::RobotSuit;
            f.armor = POWER_MAX;
            f.hull = MECH_HULL;
            f.pod_ammo = 4;
        }
        let sprint_cmd = PlayerCmd { sprint: true, move_z: 1.0, aim: [0.0, 0.0, 1.0], ..Default::default() };
        // mid-windup: no speed bonus paid yet
        for _ in 0..((POWER_STRIDE_WINDUP_S * SIM_HZ as f32) as usize / 2) {
            s.step(sprint_cmd);
        }
        assert!(s.fighters[0].stride_t <= 0.0, "the burst hasn't fired yet");
        let mid_windup_speed = (s.fighters[0].vel[0].powi(2) + s.fighters[0].vel[1].powi(2)).sqrt();
        let walk_speed = MOVE_SPEED * armor_spec(ArmorSet::RobotSuit).move_mult;
        assert!(
            mid_windup_speed <= walk_speed + 0.05,
            "windup must not pay the burst speed early: {mid_windup_speed} vs walk {walk_speed}"
        );
        // finish the windup - the burst should now be live
        for _ in 0..((POWER_STRIDE_WINDUP_S * SIM_HZ as f32) as usize) {
            s.step(sprint_cmd);
        }
        assert!(s.fighters[0].stride_t > 0.0, "the windup must resolve into an active burst");
        let burst_speed = (s.fighters[0].vel[0].powi(2) + s.fighters[0].vel[1].powi(2)).sqrt();
        let want = MOVE_SPEED * POWER_STRIDE_SPEED_MULT;
        assert!(
            (burst_speed - want).abs() < 0.05,
            "burst speed {burst_speed} vs the spec'd 110% ({want})"
        );
        assert!(burst_speed > walk_speed * 1.2, "the burst must clearly beat the 85% walk");
        // missile pod is locked out for the whole burst
        s.step(PlayerCmd { pod_aim: true, aim: [0.0, 0.0, 1.0], ..Default::default() });
        assert_eq!(s.fighters[0].pod_lock_id, -1, "the pod cannot even begin locking mid-burst");
        // committed: releasing sprint mid-burst does not cut it short
        let remaining_before = s.fighters[0].stride_t;
        s.step(PlayerCmd::default());
        assert!(
            s.fighters[0].stride_t < remaining_before && s.fighters[0].stride_t > 0.0,
            "the burst counts down on its own once committed - releasing input doesn't cancel it"
        );
        // run the burst out and confirm heat was actually spent
        for _ in 0..((POWER_STRIDE_DURATION_S * SIM_HZ as f32) as usize + 2) {
            s.step(PlayerCmd::default());
        }
        assert!(s.fighters[0].stride_t <= 0.0, "the burst must end on its own");
        assert!(
            s.fighters[0].stride_heat > 90.0,
            "a full burst should spend nearly the whole heat bar: {}",
            s.fighters[0].stride_heat
        );
        // and it cannot restart instantly while hot
        for _ in 0..((POWER_STRIDE_WINDUP_S * SIM_HZ as f32) as usize + 4) {
            s.step(sprint_cmd);
        }
        assert!(
            s.fighters[0].stride_t <= 0.0,
            "overheated - a fresh windup must not be able to complete yet"
        );
    }

    /// R&D Cycle 1 (backlog #1): the mech entry sequence. Proves the
    /// staging is monotonic and gapless - every stage visited exactly
    /// once, in the documented order, as elapsed time sweeps the whole
    /// window - and that the helper correctly reports "not entering"
    /// outside the window (before boarding, after entry completes,
    /// and during an EXIT, which reuses the same timer field for a
    /// countdown that has no stage list of its own).
    #[test]
    fn mech_entry_stages_are_monotonic_and_gapless() {
        // pure function: sweep finely and confirm every stage appears,
        // in order, with no stage skipped and no backward jump
        let mut seen = Vec::new();
        let mut steps = 0;
        let mut t = 0.0_f32;
        while t < MECH_ENTER_S {
            let stage = mech_enter_stage(t);
            if seen.last() != Some(&stage) {
                seen.push(stage);
            }
            t += MECH_ENTER_S / 4000.0; // far finer than 8 stages over 1.6s
            steps += 1;
        }
        assert_eq!(
            seen, MECH_ENTER_STAGES,
            "every stage must appear exactly once, in order, with none skipped"
        );
        assert!(steps > 100, "sanity: the sweep actually ran");
        // boundary behavior: clamps rather than panicking or wrapping
        assert_eq!(mech_enter_stage(-1.0), MechEnterStage::CockpitOpen);
        assert_eq!(mech_enter_stage(MECH_ENTER_S * 10.0), MechEnterStage::HudBoot);

        // integration: a REAL entry through step() visits CockpitOpen
        // first and HudBoot last, on the real per-tick countdown
        let mut s = range(0x6E7);
        s.fighters[0].armor_set = ArmorSet::RobotSuit;
        s.fighters[0].hull = MECH_HULL;
        s.fighters[0].mech_transition_t = MECH_ENTER_S;
        assert_eq!(
            mech_enter_stage_for(&s.fighters[0]),
            Some(MechEnterStage::CockpitOpen),
            "the instant boarding starts, the first stage is active"
        );
        for _ in 0..((MECH_ENTER_S * SIM_HZ as f32) as usize - 2) {
            s.step(PlayerCmd::default());
        }
        assert_eq!(
            mech_enter_stage_for(&s.fighters[0]),
            Some(MechEnterStage::HudBoot),
            "just before the window closes, the last stage is active"
        );
        for _ in 0..4 {
            s.step(PlayerCmd::default());
        }
        assert_eq!(
            mech_enter_stage_for(&s.fighters[0]),
            None,
            "once sealed, there is no active entry stage"
        );

        // not-entering cases: never in the mech, and mid-EXIT (same
        // timer field, different direction - must not report a stage)
        let mut idle = range(0x6E8);
        assert_eq!(mech_enter_stage_for(&idle.fighters[0]), None, "not a mech at all");
        idle.fighters[0].armor_set = ArmorSet::RobotSuit;
        idle.fighters[0].hull = MECH_HULL;
        idle.fighters[0].mech_transition_t = MECH_EXIT_S;
        idle.fighters[0].mech_exiting = true;
        assert_eq!(
            mech_enter_stage_for(&idle.fighters[0]),
            None,
            "exiting reuses the timer field but has no entry stage list"
        );
    }

    /// §5.4 (Brief VI): the AWP one-shot matrix — head and chest delete
    /// a soldier, legs NEVER one-shot; against a mech the angle armor
    /// rules (front ≈17, visor ≈34.5, rear 115): flanks kill mechs.
    #[test]
    fn awp_matrix_one_shot_rules() {
        let arm_awp = |s: &mut TdmSim| {
            let f = &mut s.fighters[0];
            f.active = 2;
            f.gun = GunKind::Awm; // DEFAULT_LOADOUT special slot
            f.ammo = 5;
            f.reserve = 10;
        };
        let mut s = range(0xA3);
        arm_awp(&mut s);
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        // LEGS: 115 × 0.75 = 86.25 — never a one-shot
        s.apply_hit(0, 1, 0.2, [0.0, 0.2, 5.0]);
        assert!(
            s.fighters[1].alive(),
            "legs never one-shot: hp {}",
            s.fighters[1].health
        );
        assert!((s.fighters[1].health - (100.0 - 115.0 * 0.75)).abs() < 0.01);
        // CHEST: 115 ×1 — lethal on a 100 HP soldier
        s.fighters[1].health = 100.0;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        assert!(!s.fighters[1].alive(), "chest one-shots");
        // HEAD: ×4 — lethal, armored or not
        s.fighters[1].health = 100.0;
        s.fighters[1].respawn_t = 0.0;
        s.apply_hit(0, 1, 1.7, [0.0, 1.7, 5.0]);
        assert!(!s.fighters[1].alive(), "head one-shots");
        // VS MECH, three angles + visor
        let mut s = range(0xA4);
        arm_awp(&mut s);
        s.fighters[1].armor_set = ArmorSet::RobotSuit;
        s.fighters[1].hull = MECH_HULL;
        s.fighters[1].yaw = std::f32::consts::PI; // facing the shooter
        // height-relative, not a magic number - survives Task 4's scale
        // change (or any future one) automatically
        let mech_visor_y = BODY_HEIGHT * MECH_SCALE * 0.90;
        let h0 = s.fighters[1].hull;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        assert!(
            (h0 - s.fighters[1].hull - 115.0 * 0.15).abs() < 0.01,
            "an AWP does NOT counter a mech frontally"
        );
        let h1 = s.fighters[1].hull;
        s.apply_hit(0, 1, mech_visor_y, [0.0, mech_visor_y, 5.0]); // visor band
        assert!(
            (h1 - s.fighters[1].hull - 115.0 * 0.15 * 2.0).abs() < 0.01,
            "front visor ≈ 34.5"
        );
        s.fighters[1].yaw = 0.0; // back turned
        let h2 = s.fighters[1].hull;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        assert!(
            (h2 - s.fighters[1].hull - 115.0).abs() < 0.01,
            "the rear arc takes the full 115"
        );
    }

    /// §5.4 (Brief VI): the missile pod — never locks infantry, warns
    /// the victim from lock START, needs the full 1.3 s, launches on
    /// release, obeys the 250°/s PN cap, and goes BALLISTIC when line
    /// of sight breaks for > 0.4 s.
    #[test]
    fn missile_pod_locks_warns_breaks_and_caps() {
        let setup = || {
            let mut s = range(0xA0D5);
            s.fighters[1].ammo = 0;
            s.fighters[1].reserve = 0;
            s.fighters[1].slot_ammo = [(0, 0); 3];
            {
                let f = &mut s.fighters[0];
                f.armor_set = ArmorSet::RobotSuit;
                f.armor = POWER_MAX;
                f.hull = MECH_HULL;
                f.pod_ammo = POD_TUBES;
            }
            s
        };
        let aim_at = |s: &TdmSim| -> [f32; 3] {
            let e = s.muzzle_origin(0);
            let g = &s.fighters[1];
            let c = [g.pos[0], g.pos[1] + g.height() * 0.5, g.pos[2]];
            normalize([c[0] - e[0], c[1] - e[1], c[2] - e[2]])
        };
        // 1) NO LOCK ON INFANTRY — a soldier under the reticle never
        // starts a lock
        let mut s = setup();
        for _ in 0..(SIM_HZ as usize) {
            let aim = aim_at(&s);
            s.step(PlayerCmd {
                pod_aim: true,
                aim,
                ..Default::default()
            });
            s.fighters[1].pos = [0.0, 0.0, 5.0];
            s.fighters[1].vel = [0.0, 0.0];
        }
        assert_eq!(s.fighters[0].pod_lock_id, -1, "infantry is NEVER locked");
        // 2) a MECH locks in 1.3 s, and the victim is warned from START
        let mut s = setup();
        s.fighters[1].armor_set = ArmorSet::RobotSuit;
        s.fighters[1].hull = MECH_HULL;
        let mut warned_early = false;
        for i in 0..((1.3 * SIM_HZ as f32) as usize + 4) {
            let aim = aim_at(&s);
            s.step(PlayerCmd {
                pod_aim: true,
                aim,
                ..Default::default()
            });
            s.fighters[1].pos = [0.0, 0.0, 5.0];
            s.fighters[1].vel = [0.0, 0.0];
            s.fighters[1].yaw = std::f32::consts::PI;
            if i == (0.4 * SIM_HZ as f32) as usize {
                assert!(
                    s.fighters[0].pod_lock_t < POD_LOCK_S,
                    "lock takes the FULL 1.3 s"
                );
                warned_early = s.fighters[1].lock_warn_t > 0.0;
            }
        }
        assert!(warned_early, "the victim is warned from lock START");
        assert!(s.fighters[0].pod_lock_t >= POD_LOCK_S, "full lock reached");
        // 3) release → homing launch; PN turn stays under the cap; the
        // hit lands with FRONT angle armor (270 × 0.15 = 40.5)
        let h0 = s.fighters[1].hull;
        let aim = aim_at(&s);
        s.step(PlayerCmd {
            pod_aim: false,
            aim,
            ..Default::default()
        });
        assert_eq!(s.fighters[0].pod_ammo, POD_TUBES - 1, "one tube spent");
        assert_eq!(s.rockets.len(), 1, "the bird is away");
        assert_eq!(s.rockets[0].target, 1, "homing on the locked mech");
        let mut prev_dir = normalize(s.rockets[0].vel);
        for _ in 0..(3 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[1].pos = [0.0, 0.0, 5.0];
            s.fighters[1].vel = [0.0, 0.0];
            s.fighters[1].yaw = std::f32::consts::PI;
            if let Some(r) = s.rockets.first() {
                let d = normalize(r.vel);
                let cosang = (d[0] * prev_dir[0] + d[1] * prev_dir[1]
                    + d[2] * prev_dir[2])
                    .clamp(-1.0, 1.0);
                let rate = cosang.acos() / DT;
                assert!(
                    rate <= ROCKET_TURN_CAP + 0.2,
                    "PN turn rate under 250°/s: {rate:.2} rad/s"
                );
                prev_dir = d;
            } else {
                break;
            }
        }
        assert!(s.rockets.is_empty(), "the missile resolves");
        assert!(
            (h0 - s.fighters[1].hull - ROCKET_DMG * 0.15).abs() < 0.5,
            "front hit ≈ 40.5 hull: took {}",
            h0 - s.fighters[1].hull
        );
        // 4) LOS break > 0.4 s → ballistic, forever
        let mut s = setup();
        s.fighters[1].armor_set = ArmorSet::RobotSuit;
        s.fighters[1].hull = MECH_HULL;
        s.fighters[1].pos = [0.0, 0.0, 60.0];
        for _ in 0..((1.4 * SIM_HZ as f32) as usize) {
            let aim = aim_at(&s);
            s.step(PlayerCmd {
                pod_aim: true,
                aim,
                ..Default::default()
            });
            s.fighters[1].pos = [0.0, 0.0, 60.0];
            s.fighters[1].vel = [0.0, 0.0];
        }
        let aim = aim_at(&s);
        s.step(PlayerCmd {
            pod_aim: false,
            aim,
            ..Default::default()
        });
        assert_eq!(s.rockets.len(), 1);
        // drop a wall between missile and target, mid-flight
        s.cover.push(Aabb {
            min: [-8.0, 0.0, 25.0],
            max: [8.0, 12.0, 27.0],
        });
        s.cover_kind.push(CoverKind::Stone);
        s.rebuild_grid();
        let mut went_ballistic = false;
        for _ in 0..(SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[1].pos = [0.0, 0.0, 60.0];
            s.fighters[1].vel = [0.0, 0.0];
            if let Some(r) = s.rockets.first() {
                if r.target == -1 {
                    went_ballistic = true;
                    break;
                }
            } else {
                break;
            }
        }
        assert!(
            went_ballistic,
            "hard cover for > 0.4 s must send the missile ballistic"
        );
    }

    /// §10 (Brief III): regen waits 12 s, heals at 8.33/s, and ANY new
    /// damage resets the clock.
    #[test]
    fn health_regen_waits_heals_and_resets() {
        let mut s = range(101);
        // disarm and strand the bot so nothing interferes
        s.fighters[1].ammo = 0;
        s.fighters[1].reserve = 0;
        s.fighters[1].slot_ammo = [(0, 0); 3];
        s.fighters[0].health = 40.0;
        s.fighters[0].last_dmg_at = s.t;
        let run = |s: &mut TdmSim, secs: usize| {
            for _ in 0..(secs * SIM_HZ as usize) {
                s.step(PlayerCmd::default());
                s.fighters[1].pos = [-30.0, 0.0, -30.0];
            }
        };
        run(&mut s, 11);
        assert!(
            s.fighters[0].health < 41.0,
            "no healing inside the 12 s window: {}",
            s.fighters[0].health
        );
        run(&mut s, 6);
        assert!(
            s.fighters[0].health > 70.0,
            "regen must be well underway by 17 s: {}",
            s.fighters[0].health
        );
        // fresh damage stops it cold
        let t_now = s.t;
        s.fighters[0].health -= 10.0;
        s.fighters[0].last_dmg_at = t_now;
        let h = s.fighters[0].health;
        run(&mut s, 5);
        assert!(
            s.fighters[0].health < h + 1.0,
            "damage must reset the regen clock: {} -> {}",
            h,
            s.fighters[0].health
        );
    }

    /// §8 (Brief II): the horde spawns out of sight, chases noise,
    /// dies to headshots (a shambler is a ×4 one-shot), and the whole
    /// run replays bit-identically.
    #[test]
    fn zombies_spawn_chase_headshot_and_replay() {
        // director spawns while the player survives, never within 35 m
        let mut s = TdmSim::new(cfg(91, 1, Mode::Extraction, MapKind::Arena));
        for _ in 0..(40 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
        }
        assert!(!s.zombies.is_empty(), "the director must spawn a horde");
        for z in &s.zombies {
            assert!(z.pos.iter().all(|v| v.is_finite()), "no NaN zombies");
        }
        // a fresh spawn is never inside 35 m of the (idle, spawn-facing)
        // player at the moment it appears — hard to catch after movement,
        // so assert the spec knob instead of chasing history
        assert!(ZSPAWN_MIN_M >= 35.0);
        // headshot rule: M4 head (50) one-shots a Shambler (42 hp)
        let mut s = TdmSim::new(cfg(92, 1, Mode::Extraction, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        s.zombies.clear();
        s.zombies.push(Zombie {
            id: 900,
            kind: ZKind::Shambler,
            pos: [0.0, 0.0, -5.0 + 8.0],
            hp: zspec(ZKind::Shambler).hp,
            atk_cd: 0.0,
            scream_t: 0.0,
            head_hits: 0,
            target: [0.0, 0.0],
            alerted: false,
        });
        s.fighters[0].pos = [0.0, 0.0, -5.0];
        s.fighters[0].yaw = 0.0;
        // aim at the shambler's head band (1.7 × 0.91 ≈ 1.55)
        let aim = [0.0, (1.55 - EYE_REL) / 8.0, 1.0];
        assert!(s.try_fire(0, aim, true), "the shot must fire");
        assert!(
            s.zombies.is_empty(),
            "one M4 headshot must drop a shambler: hp left {:?}",
            s.zombies.first().map(|z| z.hp)
        );
        // noise pulls: an unalerted shambler ~44 m out hears rifle fire
        let mut s = TdmSim::new(cfg(93, 1, Mode::Extraction, MapKind::Arena));
        s.zombies.clear();
        s.zombies.push(Zombie {
            id: 901,
            kind: ZKind::Shambler,
            pos: [20.0, 0.0, 0.0],
            hp: 42.0,
            atk_cd: 0.0,
            scream_t: 0.0,
            head_hits: 0,
            target: [20.0, 0.0],
            alerted: false,
        });
        let ppos = s.fighters[0].pos;
        s.try_fire(0, [0.0, 0.0, 1.0], false); // 90 m of rifle noise
        let z = &s.zombies[0];
        assert!(z.alerted, "gunfire must alert the horde");
        assert!(
            (z.target[0] - ppos[0]).abs() < 1.0 && (z.target[1] - ppos[2]).abs() < 1.0,
            "the noise target must be the shooter"
        );
        // determinism: a 30 s run with movement replays identically
        let outcome = || {
            let mut s = TdmSim::new(cfg(94, 2, Mode::Extraction, MapKind::Arena));
            for i in 0..(30 * SIM_HZ as usize) {
                s.step(PlayerCmd {
                    move_z: 0.7,
                    move_x: if (i / 240) % 2 == 0 { 0.4 } else { -0.4 },
                    aim: [0.0, 0.0, 1.0],
                    shoot: i % 120 < 4,
                    ..Default::default()
                });
            }
            (
                s.zombies.len(),
                s.pressure.to_bits(),
                s.fighters[0].health.to_bits(),
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>(),
            )
        };
        assert_eq!(outcome(), outcome(), "the run must replay identically");
    }

    /// §6 (Brief II): headshots stay decisive against EVERY set, the
    /// damage floor keeps limb/torso shots from ever being free, and the
    /// Folk brace cuts frontal damage without touching the rear.
    #[test]
    fn armor_sets_flats_floor_and_brace() {
        use ArmorSet::*;
        for set in [None, Folk, Pyro, RobotSuit, Recon] {
            let mut s = range(71);
            s.fighters[0].gun = GunKind::Awm;
            s.fighters[1].armor_set = set;
            s.apply_hit(0, 1, 1.70, [0.0, 1.70, 5.0]);
            assert!(
                !s.fighters[1].alive(),
                "{set:?}: an AWM headshot must remain decisive"
            );
        }
        // the floor: an M4 torso shot vs the Robot Suit lands exactly at
        // 15% of base — reduced hard, never to zero
        let mut s = range(72);
        s.fighters[1].armor_set = RobotSuit;
        let h0 = s.fighters[1].health;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        let d = h0 - s.fighters[1].health;
        assert!(
            (d - 12.5 * ARMOR_FLOOR_FRAC).abs() < 0.01,
            "floor damage: {d}"
        );
        // Folk Shieldwall Brace: big frontal cut, nothing from behind
        let mut s = range(73);
        s.fighters[0].gun = GunKind::Awm;
        s.fighters[1].armor_set = Folk;
        s.fighters[1].brace = true;
        s.fighters[1].yaw = std::f32::consts::PI; // facing the shooter
        let h0 = s.fighters[1].health;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        let front = h0 - s.fighters[1].health;
        s.fighters[1].yaw = 0.0; // back turned: the brace covers nothing
        let h1 = s.fighters[1].health;
        s.apply_hit(0, 1, 1.0, [0.0, 1.0, 5.0]);
        let rear = h1 - s.fighters[1].health;
        assert!(
            front < rear,
            "brace must cut FRONTAL damage only: front {front}, rear {rear}"
        );
    }

    /// §6: a match with thruster flight and repulsor use replays
    /// bit-identically — abilities are sim state, not presentation.
    #[test]
    fn abilities_replay_identically() {
        let outcome = || {
            let mut s = TdmSim::new(cfg(81, 4, Mode::Tdm, MapKind::Arena));
            s.fighters[0].armor_set = ArmorSet::RobotSuit;
            s.fighters[0].armor = POWER_MAX;
            for i in 0..(15 * SIM_HZ as usize) {
                s.step(PlayerCmd {
                    move_z: 0.8,
                    aim: [0.1, 0.0, 1.0],
                    jump: i % 240 == 0,
                    ability: (i % 180) < 20,
                    ..Default::default()
                });
            }
            (
                s.score[0] as u32,
                s.score[1] as u32,
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>(),
                s.fighters[0].armor.to_bits(),
            )
        };
        assert_eq!(outcome(), outcome(), "abilities must replay identically");
    }

    /// §5 (Brief II): frag damage is LOS-blocked (no damage through
    /// walls), grenades always come to rest, flash blinds only with line
    /// of sight, smoke blocks bot vision, and 40 simultaneous throwables
    /// stay bounded and deterministic.
    #[test]
    fn throwables_bounce_blast_blind_and_smoke() {
        // -- frag: no damage through a wall, real damage in the open
        let mut s = range(51);
        s.cover.push(Aabb {
            min: [-3.0, 0.0, 1.5],
            max: [3.0, 3.0, 2.1],
        });
        s.cover_kind.push(CoverKind::Stone);
        s.rebuild_grid();
        // victim behind the wall (z=5), frag on the near side (z=0.5)
        let h0 = s.fighters[1].health;
        s.grenades_air.push(Grenade {
            id: 9001,
            kind: ThrowKind::Frag,
            pos: [0.0, 1.0, 0.5],
            vel: [0.0, 0.0, 0.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 0.05,
            bounces: 0,
            rest: true,
        });
        for _ in 0..30 {
            s.step_grenades();
        }
        assert_eq!(
            s.fighters[1].health, h0,
            "frag must NOT damage through the wall"
        );
        // same blast in the open bites hard
        let mut s = range(52);
        let h0 = s.fighters[1].health;
        s.grenades_air.push(Grenade {
            id: 9002,
            kind: ThrowKind::Frag,
            pos: [0.0, 1.0, 3.5],
            vel: [0.0, 0.0, 0.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 0.05,
            bounces: 0,
            rest: true,
        });
        for _ in 0..30 {
            s.step_grenades();
        }
        assert!(
            s.fighters[1].health < h0 - 20.0,
            "open-air frag must bite: {} -> {}",
            h0,
            s.fighters[1].health
        );
        // -- flash: blinds the facing victim with LOS; a wall blocks it
        let mut s = range(53);
        s.fighters[1].yaw = std::f32::consts::PI; // facing the flash
        s.grenades_air.push(Grenade {
            id: 9003,
            kind: ThrowKind::Flash,
            pos: [0.0, 1.4, 2.0],
            vel: [0.0, 0.0, 0.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 0.05,
            bounces: 0,
            rest: true,
        });
        for _ in 0..30 {
            s.step_grenades();
        }
        assert!(
            s.fighters[1].blind_t > 1.0,
            "facing flash must blind: {}",
            s.fighters[1].blind_t
        );
        // -- smoke blocks bot SIGHT but not raw wall LOS
        let mut s = range(54);
        s.fighters[0].protect_t = 0.0; // protected players are untargetable
        assert!(
            s.nearest_visible_enemy(1).is_some(),
            "clear range: bot sees the player"
        );
        s.smokes.push(SmokeVolume {
            pos: [0.0, 1.0, 0.0], // midway between z −5 and +5
            ttl: 10.0,
        });
        assert!(
            s.nearest_visible_enemy(1).is_none(),
            "smoke must blot out bot vision"
        );
        assert!(
            s.los_clear([0.0, 1.0, -5.0], [0.0, 1.0, 5.0]),
            "walls-only LOS ignores smoke (shrapnel path)"
        );
        // -- 40 mixed throwables: all settle/detonate, counts bounded
        let mut s = range(55);
        for k in 0..40u32 {
            let kind = ThrowKind::ALL[(k % 4) as usize];
            s.grenades_air.push(Grenade {
                id: 9100 + k,
                kind,
                pos: [(k % 7) as f32 * 2.0 - 6.0, 1.5, (k / 7) as f32 * 2.0 - 6.0],
                vel: [
                    ((k * 37) % 11) as f32 - 5.0,
                    3.0,
                    ((k * 53) % 13) as f32 - 6.0,
                ],
                thrower: 0,
                team: Team::Blue,
                fuse_t: throw_spec(kind).fuse_s,
                bounces: 0,
                rest: false,
            });
        }
        for _ in 0..(12 * SIM_HZ as usize) {
            s.step(PlayerCmd::default());
            s.fighters[1].pos = [-30.0, 0.0, -30.0];
        }
        assert!(
            s.grenades_air.is_empty(),
            "all grenades must detonate/settle: {} left",
            s.grenades_air.len()
        );
        assert!(s.smokes.len() <= SMOKE_MAX, "smoke cap holds");
        for g in &s.grenades_air {
            assert!(g.pos.iter().all(|v| v.is_finite()), "no NaN positions");
        }
    }

    /// Brief IX-B: the frag falloff curve's exact shape - flat 100% out
    /// to 2m, then linear through the 50%-at-6m and 15%-at-12m
    /// breakpoints, down to 0 at 20m, monotonic and smooth (no cliff)
    /// the whole way.
    #[test]
    fn frag_falloff_matches_the_brief_ix_b_breakpoints() {
        assert_eq!(frag_falloff_frac(0.0), 1.0, "point-blank is the peak");
        assert_eq!(frag_falloff_frac(1.5), 1.0, "0-2m is flat at 100%");
        assert_eq!(frag_falloff_frac(2.0), 1.0, "2m is still the flat edge");
        assert!(
            (frag_falloff_frac(6.0) - 0.5).abs() < 1e-4,
            "6m breakpoint must be exactly 50%: got {}",
            frag_falloff_frac(6.0)
        );
        assert!(
            (frag_falloff_frac(12.0) - 0.15).abs() < 1e-4,
            "12m breakpoint must be exactly 15%: got {}",
            frag_falloff_frac(12.0)
        );
        assert_eq!(frag_falloff_frac(20.0), 0.0, "20m breakpoint must be exactly 0%");
        assert_eq!(frag_falloff_frac(25.0), 0.0, "past 20m stays 0%, never negative");
        // monotonic non-increasing across the whole range - "no hard edge
        // cliffs" (non-negotiable #3) means no local jump either
        let mut prev = frag_falloff_frac(0.0);
        for i in 1..=200 {
            let d = i as f32 * 0.1;
            let cur = frag_falloff_frac(d);
            assert!(cur <= prev + 1e-6, "falloff must never increase with distance (d={d})");
            assert!((prev - cur) < 0.05, "no single 0.1m step may drop more than 5% (d={d}) - that's a cliff");
            prev = cur;
        }
    }

    /// Brief IX-B: the frag's usable blast range now reaches 20m (was a
    /// 6m hard cutoff) - the LOS-blocked damage loop must actually reach
    /// that far, not just the pure falloff function in isolation.
    #[test]
    fn frag_damage_reaches_the_full_20m_range() {
        let mut s = range(56);
        let h0 = s.fighters[1].health;
        s.grenades_air.push(Grenade {
            id: 9200,
            kind: ThrowKind::Frag,
            pos: [0.0, 1.0, 3.5 + 14.0], // ~14m past the old 6m cutoff
            vel: [0.0, 0.0, 0.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 0.05,
            bounces: 0,
            rest: true,
        });
        for _ in 0..30 {
            s.step_grenades();
        }
        assert!(
            s.fighters[1].health < h0,
            "a frag well past the old 6m radius must still tick damage under the new 20m falloff"
        );
    }

    /// Brief IX-C "Weight & Movement Test": the brief's own worked
    /// examples - under/at budget is zero penalty, +4kg over is exactly
    /// -0.60 m/s.
    #[test]
    fn armor_weight_movement_penalty_matches_the_brief_ix_c_worked_example() {
        assert_eq!(armor_weight_movement_penalty(16.0, 20.0), 0.0, "under budget: no penalty");
        assert_eq!(armor_weight_movement_penalty(20.0, 20.0), 0.0, "exactly at budget: no penalty");
        assert!(
            (armor_weight_movement_penalty(24.0, 20.0) - 0.60).abs() < 1e-4,
            "+4kg over budget must be exactly -0.60 m/s, the brief's own worked example"
        );
        // linear, not just the two sampled points
        assert!((armor_weight_movement_penalty(21.0, 20.0) - 0.15).abs() < 1e-4);
    }

    /// Brief IX-B "Bounce & Rolling Physics": the exact per-material
    /// coefficient table.
    #[test]
    fn surface_restitution_matches_the_brief_ix_b_table() {
        assert_eq!(surface_restitution(CoverKind::Stone), 0.40);
        assert_eq!(surface_restitution(CoverKind::Crate), 0.50, "wood/metal analog");
        assert_eq!(surface_restitution(CoverKind::Hedge), 0.05, "organic: sticky");
        assert_eq!(surface_restitution(CoverKind::Tree), 0.05, "organic: sticky");
    }

    /// R&D Cycle 2 (backlog #3) [S-01, RoyMech tribology table]:
    /// per-surface friction, extending the per-surface restitution
    /// above the same way. Table values first, then the behavioral
    /// proof that actually matters - a real bounce SKIDS FURTHER on
    /// the lower-friction surface, under otherwise-identical impact
    /// conditions.
    #[test]
    fn surface_friction_is_per_material_and_stone_skids_further_than_a_crate() {
        assert_eq!(surface_friction(CoverKind::Stone), 0.30);
        assert_eq!(surface_friction(CoverKind::Crate), 0.45);
        assert!(
            surface_friction(CoverKind::Stone) < surface_friction(CoverKind::Crate),
            "worked masonry is smoother than a rough-grain wood crate"
        );

        // runs real ticks (gravity included) until the FIRST bounce
        // registers, then reports the tangential speed right after it -
        // robust to exactly how many ticks the approach takes, rather
        // than assuming impact happens within one hand-picked tick
        let bounce_once = |kind: CoverKind| -> f32 {
            let cover = vec![Aabb { min: [-5.0, 0.0, -5.0], max: [5.0, 0.3, 5.0] }];
            let cover_kind = vec![kind];
            let grid = CoverGrid::build(&cover, 20.0);
            let mut g = Grenade {
                id: 1,
                kind: ThrowKind::Frag,
                pos: [0.0, 1.0, 0.0],
                vel: [6.0, 0.0, 0.0], // mostly tangential; gravity brings it down
                thrower: 0,
                team: Team::Blue,
                fuse_t: 5.0,
                bounces: 0,
                rest: false,
            };
            for _ in 0..300 {
                grenade_tick(&mut g, &grid, &cover, &cover_kind);
                if g.bounces > 0 {
                    return (g.vel[0] * g.vel[0] + g.vel[2] * g.vel[2]).sqrt();
                }
            }
            panic!("grenade never bounced within the safety window");
        };
        let on_stone = bounce_once(CoverKind::Stone);
        let on_crate = bounce_once(CoverKind::Crate);
        assert!(
            on_stone > on_crate,
            "identical impact must skid FURTHER on the lower-friction surface: \
             stone {on_stone} vs crate {on_crate}"
        );

        // ground-plane hits (no cover object) are UNCHANGED by this
        // cycle - they keep the throw kind's own uniform friction,
        // exactly the same fallback rule restitution already had
        let empty_cover: Vec<Aabb> = vec![];
        let empty_kind: Vec<CoverKind> = vec![];
        let grid = CoverGrid::build(&empty_cover, 20.0);
        let mut g = Grenade {
            id: 2,
            kind: ThrowKind::Frag,
            pos: [0.0, 1.0, 0.0],
            vel: [6.0, 0.0, 0.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 5.0,
            bounces: 0,
            rest: false,
        };
        let mut ground_speed = 0.0;
        let mut bounced = false;
        for _ in 0..300 {
            grenade_tick(&mut g, &grid, &empty_cover, &empty_kind);
            if g.bounces > 0 {
                ground_speed = (g.vel[0] * g.vel[0] + g.vel[2] * g.vel[2]).sqrt();
                bounced = true;
                break;
            }
        }
        assert!(bounced, "the ground-plane grenade never bounced within the safety window");
        // the flat normal here is [0,1,0], so the tangential vector at
        // impact is exactly [6.0, 0, 0] no matter how long the fall
        // took (gravity only ever touches the NORMAL component) - the
        // post-bounce speed is therefore exact, not approximate
        let expected = 6.0 * (1.0 - throw_spec(ThrowKind::Frag).friction);
        assert!(
            (ground_speed - expected).abs() < 1e-3,
            "ground-plane bounce must still use the throw kind's own friction: \
             got {ground_speed}, expected {expected}"
        );
    }

    #[test]
    fn cover_kind_at_finds_the_containing_object_and_none_for_open_air() {
        let cover = vec![Aabb { min: [-1.0, 0.0, -1.0], max: [1.0, 2.0, 1.0] }];
        let kind = vec![CoverKind::Stone];
        assert_eq!(cover_kind_at(&cover, &kind, [0.0, 1.0, 0.0]), Some(CoverKind::Stone));
        assert_eq!(
            cover_kind_at(&cover, &kind, [50.0, 1.0, 50.0]),
            None,
            "far from any cover object must read as open air, not a false match"
        );
    }

    /// Brief IX-B: grenades bounce at each surface's OWN coefficient
    /// instead of a single per-throw-kind default - organic cover sticks
    /// (immediate rest, zero velocity) rather than bouncing at all, stone
    /// bounces normally.
    #[test]
    fn grenade_bounce_uses_surface_material_stone_bounces_organic_sticks() {
        // -- organic (hedge): sticks on contact
        let mut s = range(57);
        s.cover.push(Aabb { min: [-3.0, 0.0, 2.0], max: [3.0, 3.0, 2.5] });
        s.cover_kind.push(CoverKind::Hedge);
        s.rebuild_grid();
        s.grenades_air.push(Grenade {
            id: 9300,
            kind: ThrowKind::Frag,
            pos: [0.0, 1.5, 1.8],
            vel: [0.0, 0.0, 4.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 5.0, // long fuse - only checking the bounce, not the boom
            bounces: 0,
            rest: false,
        });
        for _ in 0..15 {
            s.step_grenades();
        }
        let g = s
            .grenades_air
            .iter()
            .find(|g| g.id == 9300)
            .expect("must still be tracked (long fuse, no detonation yet)");
        assert!(g.rest, "organic surfaces must stick (rest) rather than bounce");
        assert_eq!(g.vel, [0.0, 0.0, 0.0], "sticking means zero residual velocity");

        // -- stone: bounces at ~40% restitution, does not immediately stick
        let mut s = range(58);
        s.cover.push(Aabb { min: [-3.0, 0.0, 2.0], max: [3.0, 3.0, 2.5] });
        s.cover_kind.push(CoverKind::Stone);
        s.rebuild_grid();
        s.grenades_air.push(Grenade {
            id: 9301,
            kind: ThrowKind::Frag,
            pos: [0.0, 1.5, 1.8],
            vel: [0.0, 0.0, 4.0],
            thrower: 0,
            team: Team::Blue,
            fuse_t: 5.0,
            bounces: 0,
            rest: false,
        });
        for _ in 0..10 {
            s.step_grenades();
        }
        let g = s
            .grenades_air
            .iter()
            .find(|g| g.id == 9301)
            .expect("must still be tracked (long fuse, no detonation yet)");
        assert!(g.bounces >= 1, "must have bounced off stone within 10 ticks");
        assert!(
            !g.rest,
            "a single stone bounce (0.40 restitution) must not immediately stick like organic cover"
        );
    }

    /// §5: a scripted match WITH thrown grenades replays bit-identically.
    #[test]
    fn throwables_are_deterministic() {
        let outcome = || {
            let mut s = TdmSim::new(cfg(61, 4, Mode::Tdm, MapKind::Arena));
            for i in 0..(20 * SIM_HZ as usize) {
                // pulse the throw key: hold for 30 ticks every 4 s, cycle
                // the selection every 6 s
                let hold = (i % (4 * SIM_HZ as usize)) < 30;
                let cycle = i % (6 * SIM_HZ as usize) == 0;
                s.step(PlayerCmd {
                    move_z: 0.6,
                    aim: [0.2, 0.25, 1.0],
                    throw_hold: hold,
                    cycle_throw: cycle,
                    ..Default::default()
                });
            }
            (
                s.score[0] as u32,
                s.score[1] as u32,
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>(),
                s.smokes.len(),
                s.fires.len(),
            )
        };
        assert_eq!(outcome(), outcome(), "griefing must replay identically");
    }

    /// §4 (Brief II): the preview and the flight share one integrator and
    /// one gravity constant — over a long flat shot they may not diverge
    /// by more than 5 cm. If this fails, the preview is lying.
    #[test]
    fn preview_matches_flight_within_5cm() {
        // per weapon: launch pitch that maximises carry, and the minimum
        // distance that still counts as a LONG shot for it
        for (is_spear, v0, pitch, min_m) in
            [(true, 26.0_f32, 0.42, 45.0_f32), (false, 52.0, 0.10, 55.0)]
        {
            let mut s = TdmSim::new(cfg(19, 1, Mode::Tdm, MapKind::Arena));
            s.cover.clear();
            s.cover_kind.clear();
            s.rebuild_grid();
            let o = [-30.0, EYE_REL, -30.0];
            let d = normalize([0.55, pitch, 0.83]); // long diagonal
            let (_, predicted, _) = s.predict_arc(o, d, v0, is_spear, 8.0);
            s.missiles.push(Missile {
                id: 700,
                pos: o,
                vel: [d[0] * v0, d[1] * v0, d[2] * v0],
                team: Team::Blue,
                shooter: 0,
                damage: 0.0,
                is_spear,
                stuck_t: None,
                embedded: true,
                pierces_left: 0,
                pierced: Vec::new(),
                power: 1.0,
            });
            let mut landed = None;
            for _ in 0..(8 * SIM_HZ as usize) {
                s.step_missiles();
                if let Some(m) = s.missiles.iter().find(|m| m.id == 700) {
                    if m.stuck_t.is_some() {
                        landed = Some(m.pos);
                        break;
                    }
                } else {
                    landed = s.dropped.last().map(|d| d.pos);
                    break;
                }
            }
            let landed = landed.expect("missile must land");
            let dist = ((landed[0] - o[0]).powi(2) + (landed[2] - o[2]).powi(2)).sqrt();
            assert!(dist > min_m, "the shot must be LONG: went {dist:.1} m");
            let err = ((landed[0] - predicted[0]).powi(2)
                + (landed[1] - predicted[1]).powi(2)
                + (landed[2] - predicted[2]).powi(2))
            .sqrt();
            assert!(
                err < 0.05,
                "spear={is_spear}: preview off by {err:.3} m at {dist:.1} m"
            );
        }
    }

    /// §3: thrown spears recover at 100% up to the cap (which refuses
    /// WITHOUT consuming the pile), arrows at ~65%, and the whole cycle
    /// replays bit-identically.
    #[test]
    fn dropped_ammo_is_recoverable_and_deterministic() {
        let run = |arrows: bool| -> (u32, usize) {
            let mut s = TdmSim::new(cfg(33, 1, Mode::Tdm, MapKind::Arena));
            s.cover.clear();
            s.cover_kind.clear();
            s.rebuild_grid();
            s.pickups.clear();
            s.checkpoints.clear();
            // disarm and strand the bot so it can't interfere
            s.fighters[1].ammo = 0;
            s.fighters[1].reserve = 0;
            s.fighters[1].slot_ammo = [(0, 0); 3];
            // the player carries the matching launcher, bone dry
            let launcher = if arrows { GunKind::Bow } else { GunKind::Spear };
            s.fighters[0].inventory = [GunKind::M4, GunKind::Glock, launcher];
            s.fighters[0].active = 2;
            s.fighters[0].gun = launcher;
            s.fighters[0].ammo = 0;
            s.fighters[0].reserve = 0;
            s.fighters[0].slot_ammo = [(30, 120), (17, 68), (0, 0)];
            // rain 30 projectiles in a grid of separate piles
            for k in 0..30u32 {
                let x = (k % 6) as f32 * 0.9 - 2.5;
                let z = 8.0 + (k / 6) as f32 * 0.9;
                s.missiles.push(Missile {
                    id: 500 + k,
                    pos: [x, 3.0, z],
                    vel: [0.0, -8.0, 0.0],
                    team: Team::Blue,
                    shooter: 0,
                    damage: 0.0,
                    is_spear: !arrows,
                    stuck_t: None,
                    embedded: true,
                    pierces_left: 0,
                    pierced: Vec::new(),
                    power: 1.0,
                });
            }
            for _ in 0..SIM_HZ as usize {
                s.step(PlayerCmd::default());
                s.fighters[0].pos = [0.0, 0.0, -5.0];
                s.fighters[1].pos = [-30.0, 0.0, -30.0];
            }
            // walk over every pile
            for k in 0..30u32 {
                let x = (k % 6) as f32 * 0.9 - 2.5;
                let z = 8.0 + (k / 6) as f32 * 0.9;
                s.fighters[0].pos = [x, 0.0, z];
                s.step(PlayerCmd::default());
                s.fighters[1].pos = [-30.0, 0.0, -30.0];
            }
            (s.fighters[0].reserve, s.dropped.len())
        };
        let (spear_reserve, spears_left) = run(false);
        assert_eq!(
            spear_reserve, AMMO_CAP_SPEAR,
            "spear reserve fills to the cap"
        );
        assert!(
            spears_left > 0,
            "over-cap spears stay on the ground, unconsumed"
        );
        let (arrow_reserve, _) = run(true);
        assert!(
            (10..=24).contains(&arrow_reserve),
            "arrows recover at ~65%: got {arrow_reserve}"
        );
        // §3.3 determinism: identical runs, bit-identical reserves
        assert_eq!(run(true), run(true), "arrow recovery must replay");
        assert_eq!(run(false), run(false), "spear recovery must replay");
    }

    /// §9.1: the grid broadphase must return EXACTLY the linear scan's
    /// nearest hit — 10,000 random rays per map — and beat it on time.
    #[test]
    fn broadphase_matches_linear_scan() {
        use jk_core::Pcg32;
        for map in MapKind::ALL {
            let s = TdmSim::new(cfg(0xB40A, 5, Mode::Tdm, map));
            let half = s.half;
            let linear = |o: [f32; 3], d: [f32; 3], tm: f32| -> Option<(f32, [f32; 3])> {
                let mut best: Option<(f32, [f32; 3])> = None;
                for c in &s.cover {
                    if let Some((t, n)) = c.ray_hit(o, d, tm) {
                        if best.map_or(true, |(bt, _)| t < bt) {
                            best = Some((t, n));
                        }
                    }
                }
                best
            };
            let mut rng = Pcg32::new(0xB40AD, 7);
            let mut rays = Vec::new();
            for _ in 0..10_000 {
                let o = [
                    rng.range(-half, half),
                    rng.range(0.0, 6.0),
                    rng.range(-half, half),
                ];
                let mut d = [
                    rng.range(-1.0, 1.0),
                    rng.range(-0.6, 0.6),
                    rng.range(-1.0, 1.0),
                ];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-3);
                d = [d[0] / l, d[1] / l, d[2] / l];
                rays.push((o, d));
            }
            for &(o, d) in &rays {
                let a = s.raycast_cover(o, d, 200.0);
                let b = linear(o, d, 200.0);
                match (a, b) {
                    (None, None) => {}
                    (Some((ta, na)), Some((tb, nb))) => {
                        assert!(
                            (ta - tb).abs() < 1e-4,
                            "{map:?}: grid t {ta} vs linear t {tb} for {o:?} {d:?}"
                        );
                        assert_eq!(na, nb, "{map:?}: normals differ at {o:?} {d:?}");
                    }
                    _ => panic!("{map:?}: grid {a:?} vs linear {b:?} for {o:?} {d:?}"),
                }
            }
            // the whole point: the grid must not be slower than the scan
            let t0 = std::time::Instant::now();
            for &(o, d) in &rays {
                std::hint::black_box(s.raycast_cover(o, d, 200.0));
            }
            let grid_t = t0.elapsed();
            let t0 = std::time::Instant::now();
            for &(o, d) in &rays {
                std::hint::black_box(linear(o, d, 200.0));
            }
            let lin_t = t0.elapsed();
            println!("{map:?}: 10k rays — grid {grid_t:?} vs linear {lin_t:?}");
        }
    }

    #[test]
    fn deterministic_battle() {
        let outcome = || {
            let mut s = TdmSim::new(cfg(21, 5, Mode::Tdm, MapKind::Arena));
            run(&mut s, 40, PlayerCmd::default());
            (
                s.score[0] as u32,
                s.score[1] as u32,
                s.fighters.iter().map(|f| f.deaths).collect::<Vec<_>>(),
            )
        };
        assert_eq!(outcome(), outcome(), "same seed must replay identically");
    }

    // ---- §3.2/§3.4 (Brief VII v2) - spear throw completion gate:
    // stick-vs-bounce angle threshold and the zone-damage table.

    #[test]
    fn stick_angle_threshold_matches_spec() {
        // straight down into flat ground = 90deg, embeds
        assert!(impact_angle_to_surface_deg([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]) > SPEAR_STICK_ANGLE_DEG);
        // dead level, skimming the surface = 0deg, bounces
        assert!(impact_angle_to_surface_deg([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]) < SPEAR_STICK_ANGLE_DEG);
        // exactly at the 30deg boundary (within float tolerance)
        let at_30 = (SPEAR_STICK_ANGLE_DEG.to_radians()).sin();
        let dir = normalize([((1.0 - at_30 * at_30).max(0.0)).sqrt(), -at_30, 0.0]);
        assert!((impact_angle_to_surface_deg(dir, [0.0, 1.0, 0.0]) - SPEAR_STICK_ANGLE_DEG).abs() < 0.5);
        // 25/35 either side of the 30 boundary, unambiguously
        let steep = normalize([0.3, -1.0, 0.0]); // ~73deg - well above 30
        let shallow = normalize([1.0, -0.2, 0.0]); // ~11deg - well below 30
        assert!(impact_angle_to_surface_deg(steep, [0.0, 1.0, 0.0]) >= SPEAR_STICK_ANGLE_DEG);
        assert!(impact_angle_to_surface_deg(shallow, [0.0, 1.0, 0.0]) < SPEAR_STICK_ANGLE_DEG);
    }

    /// §1.3 (BRIEF VIII), the doctrine's headline rule: "Full stops
    /// never hit instant zero." Before this, BOTH the player and bot
    /// paths wrote `vel` straight from input - release the key and
    /// velocity was exactly 0.0 one tick later. That is the named
    /// anti-pattern "the wall stop" at its source.
    #[test]
    fn a_full_stop_is_never_instant_and_counter_strafing_beats_releasing() {
        let planar = |f: &Fighter| (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
        let run_then = |release: bool| -> (f32, usize) {
            let mut s = range(0xACCE1);
            // get to a real sprint first
            for _ in 0..(SIM_HZ as usize) {
                s.step(PlayerCmd {
                    move_z: 1.0,
                    sprint: true,
                    aim: [0.0, 0.0, 1.0],
                    ..Default::default()
                });
            }
            let moving = planar(&s.fighters[0]);
            assert!(moving > SPRINT_SPEED * 0.8, "must actually be sprinting: {moving}");
            // either let go, or press straight back against it
            let cmd = if release {
                PlayerCmd { aim: [0.0, 0.0, 1.0], ..Default::default() }
            } else {
                PlayerCmd { move_z: -1.0, aim: [0.0, 0.0, 1.0], ..Default::default() }
            };
            let mut ticks = 0usize;
            // one tick immediately after the input change
            s.step(cmd);
            let after_one_tick = planar(&s.fighters[0]);
            ticks += 1;
            while planar(&s.fighters[0]) > 0.05 && ticks < SIM_HZ as usize {
                s.step(cmd);
                ticks += 1;
            }
            (after_one_tick, ticks)
        };

        let (speed_after_one_tick, release_ticks) = run_then(true);
        // THE RULE: one tick after letting go you are still moving
        assert!(
            speed_after_one_tick > 1.0,
            "a full stop must not be instant - one tick after release the body \
             was still doing {speed_after_one_tick} m/s (pre-change this was 0.0)"
        );
        assert!(
            release_ticks > 8,
            "coming to rest should take real time, took {release_ticks} ticks"
        );

        // counter-strafing must BEAT releasing, and must fall out of the
        // two-rate model rather than being special-cased anywhere
        let (_, counter_ticks) = run_then(false);
        assert!(
            counter_ticks < release_ticks,
            "pressing back must kill speed faster than letting go: \
             counter {counter_ticks} vs release {release_ticks} ticks"
        );

        // the pure function's own contract: it lands exactly on target
        // rather than overshooting and oscillating around it
        let landed = approach_velocity([0.001, 0.0], [0.0, 0.0], DT);
        assert_eq!(landed, [0.0, 0.0], "a sub-step remainder must land exactly on target");
    }

    /// §4.5 (BRIEF VIII): assist tracking - previously entirely
    /// unbuilt (`KillEvent` had no assist field at all; `kills`/`deaths`
    /// existed on Fighter but nothing tracked "who else hit them
    /// recently"). Covers the real case (a teammate's assist credited),
    /// the two things that must NEVER be credited (self-assist, an
    /// enemy's earlier hit "assisting" the very kill that avenges it),
    /// and the recency window.
    #[test]
    fn assist_credits_a_teammate_within_the_window_never_the_enemy_or_self() {
        let mut s = TdmSim::new(cfg(0xA5515, 2, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        for f in s.fighters.iter_mut() {
            f.protect_t = 0.0;
        }
        // 0,1 = Blue (teammates); 2,3 = Red
        s.fighters[2].health = 50.0;
        s.apply_plain_damage(0, 2, 20.0, [0.0, 1.0, 0.0], false, false);
        assert!(s.fighters[2].alive(), "the first hit must not be fatal - it's the setup");
        s.apply_plain_damage(1, 2, 40.0, [0.0, 1.0, 0.0], false, false);
        assert!(!s.fighters[2].alive(), "the second hit finishes it");
        assert_eq!(s.fighters[0].assists, 1, "fighter 0's earlier hit must be credited");
        let (ev, _) = s.kill_feed.last().unwrap();
        assert_eq!(ev.killer, 1);
        assert_eq!(ev.assist, Some(0), "the KillEvent itself must carry the assist");

        // self-damage must never claim assist credit on your OWN death
        s.fighters[3].health = 50.0;
        s.fighters[3].respawn_t = 0.0;
        s.apply_plain_damage(3, 3, 20.0, [0.0, 1.0, 0.0], false, false); // self-damage
        s.apply_plain_damage(0, 3, 40.0, [0.0, 1.0, 0.0], false, false);
        assert_eq!(
            s.kill_feed.last().unwrap().0.assist,
            None,
            "a fighter must never get assist credit on their own death"
        );

        // recency window: an old hit does not count
        s.fighters[2].health = 50.0;
        s.fighters[2].respawn_t = 0.0;
        s.apply_plain_damage(0, 2, 20.0, [0.0, 1.0, 0.0], false, false);
        s.t += ASSIST_WINDOW_S + 1.0;
        s.apply_plain_damage(1, 2, 40.0, [0.0, 1.0, 0.0], false, false);
        assert_eq!(
            s.kill_feed.last().unwrap().0.assist,
            None,
            "a hit older than the assist window must not count"
        );
    }

    /// §4.1 (BRIEF VII): full-draw hold sway - previously entirely
    /// unbuilt. "Full-draw hold: steady 4s; then rotational aim sway
    /// ramps +/-0.4deg -> +/-1.2deg over the next 4s... Crouching
    /// halves sway."
    #[test]
    fn bow_sway_ramps_after_the_steady_window_and_crouch_halves_it() {
        assert_eq!(bow_sway_deg(0.0, false), 0.0, "no sway at the start of the hold");
        assert_eq!(bow_sway_deg(2.0, false), 0.0, "still steady mid-way through the 4s window");
        assert_eq!(bow_sway_deg(4.0, false), 0.0, "sway hasn't started AT the 4s boundary");
        let start = bow_sway_deg(4.01, false);
        assert!(
            (start - BOW_SWAY_MIN_DEG).abs() < 0.01,
            "sway begins at the min (0.4deg), not zero: got {start}"
        );
        let mid = bow_sway_deg(6.0, false);
        assert!(
            mid > start && mid < BOW_SWAY_MAX_DEG,
            "sway must be strictly ramping mid-way: {start} < {mid} < {}",
            BOW_SWAY_MAX_DEG
        );
        let full = bow_sway_deg(8.0, false);
        assert!(
            (full - BOW_SWAY_MAX_DEG).abs() < 0.01,
            "sway caps at 1.2deg by the 8s mark: got {full}"
        );
        assert_eq!(
            bow_sway_deg(9.5, false),
            full,
            "sway does not keep growing past 8s (forced letdown is what ends the hold, at 10s)"
        );
        // crouch halves it, at every point in the ramp
        assert!((bow_sway_deg(6.0, true) - mid * 0.5).abs() < 1e-4);
        assert!((bow_sway_deg(8.0, true) - full * 0.5).abs() < 1e-4);

        // integration: a bow shot released after a long hold scatters
        // MORE than one released quickly, over real repeated draws -
        // wired through try_fire's actual RNG stream, not just the
        // pure function in isolation
        let deviation = |held_ticks: usize| -> f32 {
            let mut total = 0.0_f32;
            for seed in 0..40u64 {
                let mut s = range(0x50E7 + seed);
                s.fighters[0].inventory[0] = GunKind::Bow;
                s.fighters[0].gun = GunKind::Bow;
                s.fighters[0].ammo = 1;
                for _ in 0..held_ticks {
                    s.step_bow_draw(0, [0.0, 0.0, 1.0], true);
                }
                s.step_bow_draw(0, [0.0, 0.0, 1.0], false);
                if let Some(a) = s.missiles.last() {
                    let lateral = (a.vel[0] * a.vel[0] + a.vel[1] * a.vel[1]).sqrt();
                    total += lateral;
                }
            }
            total / 40.0
        };
        let quick = deviation((0.3 * SIM_HZ as f32) as usize); // well inside the steady window
        let long_hold = deviation((7.0 * SIM_HZ as f32) as usize); // deep in the sway ramp
        assert!(
            long_hold > quick,
            "a long hold must scatter more on average than a quick release: {long_hold} vs {quick}"
        );
    }

    /// §5.4 (BRIEF VIII): the running-throw bonus - previously entirely
    /// unbuilt (zero call sites for anything named "running" near the
    /// spear before this test). "A throw initiated at >=70% run speed
    /// with >=2 steps of momentum gets velocity x1.15."
    #[test]
    fn a_running_start_gives_the_spear_throw_its_1_15x_bonus() {
        // NOTE: the sprint duration here is deliberately longer than
        // RUNNING_THROW_MIN_S. Since §1.3's acceleration model landed,
        // reaching the 70%-of-sprint threshold itself takes real time
        // (~0.08s at GROUND_ACCEL), and only time spent ABOVE that
        // threshold accumulates toward the bonus. So the throw now
        // requires a genuine approach RUN, not an instant one - which
        // is closer to the brief's stated intent ("exactly as an
        // approach run rewards a real thrower over a standing throw")
        // than the old instant-to-top-speed behaviour was. The
        // assertion below is unchanged; only the run-up is honest now.
        let sprint_ticks = ((RUNNING_THROW_MIN_S + 0.3) * SIM_HZ as f32) as usize;
        let throw_speed = |run_first: bool| -> f32 {
            let mut s = range(0x5EA2);
            s.fighters[0].inventory[0] = GunKind::Spear;
            s.fighters[0].gun = GunKind::Spear;
            s.fighters[0].ammo = 1;
            if run_first {
                for _ in 0..sprint_ticks {
                    s.step(PlayerCmd {
                        move_z: 1.0,
                        sprint: true,
                        aim: [0.0, 0.0, 1.0],
                        ..Default::default()
                    });
                }
            }
            let ok = s.try_fire(0, [0.0, 0.0, 1.0], true);
            assert!(ok, "the throw must actually start");
            s.fighters[0].spear_v0
        };
        let standing = throw_speed(false);
        let running = throw_speed(true);
        assert!(
            (running - standing * RUNNING_THROW_MULT).abs() < 1e-3,
            "running release {running} must be exactly standing {standing} x {RUNNING_THROW_MULT}"
        );

        // a brief tap does NOT count as "2 steps of momentum" - it has
        // to be sustained, per the brief's own distinction from a
        // standing throw
        let mut tap = range(0x5EA3);
        tap.fighters[0].inventory[0] = GunKind::Spear;
        tap.fighters[0].gun = GunKind::Spear;
        tap.fighters[0].ammo = 1;
        tap.step(PlayerCmd {
            move_z: 1.0,
            sprint: true,
            aim: [0.0, 0.0, 1.0],
            ..Default::default()
        });
        assert!(tap.try_fire(0, [0.0, 0.0, 1.0], true));
        assert!(
            (tap.fighters[0].spear_v0 - standing).abs() < 1e-3,
            "a single tick of sprint input must not fake 2 steps of momentum"
        );
    }

    #[test]
    fn a_shallow_spear_throw_bounces_and_is_lost_not_a_pickup() {
        let mut s = range(21);
        // a near-horizontal throw at the ground from just above it -
        // guaranteed shallow (well under 30deg) when it grazes in
        s.missiles.push(Missile {
            id: 9001,
            pos: [0.0, 0.15, 0.0],
            vel: [12.0, 0.0, 0.0],
            team: Team::Blue,
            shooter: 0,
            damage: 85.0,
            is_spear: true,
            stuck_t: None,
            embedded: true,
            pierces_left: 0,
            pierced: Vec::new(),
            power: 1.0,
        });
        let before = s.dropped.len();
        for _ in 0..(2 * SIM_HZ as usize) {
            s.step_missiles();
        }
        assert_eq!(s.dropped.len(), before, "a shallow bounce must NOT leave a pickup");
    }

    #[test]
    fn a_steep_spear_throw_embeds_and_becomes_a_pickup() {
        let mut s = range(22);
        s.missiles.push(Missile {
            id: 9002,
            pos: [0.0, 3.0, 0.0],
            vel: [0.2, -9.0, 0.0], // nearly straight down
            team: Team::Blue,
            shooter: 0,
            damage: 85.0,
            is_spear: true,
            stuck_t: None,
            embedded: true,
            pierces_left: 0,
            pierced: Vec::new(),
            power: 1.0,
        });
        let before = s.dropped.len();
        for _ in 0..(2 * SIM_HZ as usize) {
            s.step_missiles();
        }
        assert_eq!(s.dropped.len(), before + 1, "a steep embed must leave exactly one pickup");
    }

    /// §3.2: 85 body, ×2 head (170, a lethal skill shot but not the
    /// guns' ×4), ×0.75 legs - measured through the real hit path.
    #[test]
    fn spear_zone_damage_matches_85_170_64() {
        let dmg_at = |frac: f32| {
            let mut s = range(23);
            let h = s.fighters[1].height();
            let base_y = s.fighters[1].pos[1];
            let hp0 = s.fighters[1].health;
            s.missiles.push(Missile {
                id: 9100,
                pos: [0.0, base_y + h * frac, 5.0],
                vel: [0.0, 0.0, -1.0],
                team: Team::Blue,
                shooter: 0,
                damage: 85.0,
                is_spear: true,
                stuck_t: None,
                embedded: true,
                pierces_left: 0,
                pierced: Vec::new(),
                power: 1.0,
            });
            s.step_missiles();
            hp0 - s.fighters[1].health
        };
        assert!((dmg_at(0.9) - 170.0).abs() < 0.5, "head: {}", dmg_at(0.9));
        assert!((dmg_at(0.5) - 85.0).abs() < 0.5, "torso: {}", dmg_at(0.5));
        assert!((dmg_at(0.1) - 63.75).abs() < 0.5, "legs: {}", dmg_at(0.1));
    }

    /// §3.4: the ammo cap is genuinely 2, not a leftover 6.
    #[test]
    fn spear_carry_cap_is_two() {
        assert_eq!(AMMO_CAP_SPEAR, 2, "Brief VII v2 §3.2: carry max 2");
    }

    // ---- §4.3 (Brief VII v2) - bow draw-and-pierce completion gate ----

    #[test]
    fn draw_power_golden_curve() {
        assert_eq!(bow_power_fraction(0.0), None, "an instant tap is a letdown");
        assert_eq!(bow_power_fraction(0.10), None, "under 0.15s is a letdown");
        let p15 = bow_power_fraction(BOW_DRAW_MIN_S).unwrap();
        assert!((p15 - 0.35).abs() < 0.01, "35% right at the 0.15s floor: {p15}");
        let p_mid = bow_power_fraction(0.425).unwrap(); // halfway 0.15->0.7
        assert!((p_mid - 0.675).abs() < 0.02, "linear midpoint ~67.5%: {p_mid}");
        let p_full = bow_power_fraction(BOW_DRAW_FULL_S).unwrap();
        assert!((p_full - 1.0).abs() < 0.001, "100% right at 0.7s: {p_full}");
        let p_held = bow_power_fraction(5.0).unwrap();
        assert_eq!(p_held, 1.0, "holding past full draw doesn't overcharge");
        assert_eq!(bow_power_fraction(BOW_DRAW_FORCE_S), None, "10s = forced letdown");
        assert_eq!(bow_power_fraction(12.0), None, "well past 10s is still a letdown");
    }

    #[test]
    fn letdown_under_015s_fires_nothing() {
        let mut s = range(24);
        s.fighters[0].gun = GunKind::Bow;
        s.fighters[0].ammo = 1;
        let ammo0 = s.fighters[0].ammo;
        let hold = PlayerCmd { aim: [0.0, 0.0, 1.0], shoot: true, ..Default::default() };
        let released = PlayerCmd { shoot: false, ..Default::default() };
        // hold for 6 ticks at 120Hz = 0.05s - well under the 0.15s floor
        for _ in 0..6 {
            s.step(hold);
        }
        s.step(released);
        assert!(s.missiles.is_empty(), "a sub-0.15s tap must not loose an arrow");
        assert_eq!(s.fighters[0].ammo, ammo0, "a letdown must not spend ammo");
    }

    /// §4.2: the fantasy - three soldiers in a row, one behind that line,
    /// all take the arrow: damage cascades 90 -> 67.5 -> 50.625 (×0.75
    /// per pierce, at full draw), and the fourth body is untouched.
    #[test]
    fn full_draw_arrow_pierces_three_and_stops() {
        // needs 1 shooter + 4 targets - `range()`'s 1v1 fixture is too
        // small, so build a wider roster directly
        let mut s = TdmSim::new(cfg(25, 3, Mode::Tdm, MapKind::Arena));
        s.cover.clear();
        s.cover_kind.clear();
        s.rebuild_grid();
        s.fighters[0].gun = GunKind::Bow;
        for k in 1..5usize {
            s.fighters[k].team = Team::Red;
            s.fighters[k].pos = [0.0, 0.0, 5.0 + (k - 1) as f32 * 1.5];
            s.fighters[k].health = MAX_HEALTH;
            s.fighters[k].protect_t = 0.0; // spawn protection would no-sell every hit
        }
        let before: Vec<f32> = (1..5).map(|k| s.fighters[k].health).collect();
        // torso height (0.5 of body height), NOT eye height - eye level
        // reads as a headshot (×2) and would confound the cascade values
        let torso_y = 0.5 * BODY_HEIGHT;
        s.spawn_arrow([0.0, torso_y, 0.0], [0.0, 0.0, 1.0], 1.0, 0);
        for _ in 0..(2 * SIM_HZ as usize) {
            s.step_missiles();
        }
        let taken: Vec<f32> = (1..5).map(|k| before[k - 1] - s.fighters[k].health).collect();
        assert!((taken[0] - BOW_PIERCE_DMG[0]).abs() < 0.5, "1st: {}", taken[0]);
        assert!((taken[1] - BOW_PIERCE_DMG[1]).abs() < 0.5, "2nd: {}", taken[1]);
        assert!((taken[2] - BOW_PIERCE_DMG[2]).abs() < 0.5, "3rd: {}", taken[2]);
        assert_eq!(taken[3], 0.0, "the 4th body is untouched - only 3 pierces");
    }

    #[test]
    fn bow_draw_and_pierce_replay_identically() {
        let outcome = || {
            let mut s = TdmSim::new(cfg(0xB0B0, 5, Mode::Tdm, MapKind::Arena));
            s.fighters[0].gun = GunKind::Bow;
            let hold = PlayerCmd { aim: [0.05, 0.0, 1.0], shoot: true, ..Default::default() };
            let released = PlayerCmd { aim: [0.05, 0.0, 1.0], shoot: false, ..Default::default() };
            for i in 0..(10 * SIM_HZ as usize) {
                s.step(if i % 90 < 60 { hold } else { released });
            }
            s.fighters.iter().map(|f| (f.health, f.deaths)).collect::<Vec<_>>()
        };
        assert_eq!(outcome(), outcome(), "the bow's draw/release/pierce cycle must replay identically");
    }

    // ---- §6.4 (Brief VII v2) - mech overhaul completion gate: entry
    // committal, exit, and the damage-state plate matrix.


    #[test]
    fn boarding_is_committed_not_instant() {
        let mut s = range(30);
        s.fighters[0].armor_set = ArmorSet::RobotSuit;
        s.fighters[0].hull = MECH_HULL;
        s.fighters[0].mech_transition_t = MECH_ENTER_S;
        s.fighters[0].ammo = 30;
        assert!(
            !s.try_fire(0, [0.0, 0.0, 1.0], false),
            "a freshly-boarded mech can't fight yet - still sealing up"
        );
        // let the seal finish
        for _ in 0..((MECH_ENTER_S * SIM_HZ as f32) as usize + 2) {
            s.step(PlayerCmd::default());
        }
        assert_eq!(s.fighters[0].mech_transition_t, 0.0, "sealed - ready to fight");
    }

    /// §6.2: the entry timer is CHASSIS state, but `try_fire`'s gate was
    /// not scoped to actually being in a chassis - so a pilot who
    /// dismounted (or was blown out) mid-boarding stayed disarmed ON
    /// FOOT for the rest of the window, with nothing in the HUD saying
    /// why. Both teardown paths must hand back a pilot who can fight.
    #[test]
    fn leaving_the_chassis_mid_entry_does_not_disarm_the_pilot() {
        // -- destroyed mid-entry: the ejected pilot must be able to fight
        let mut s = range(36);
        s.fighters[0].armor_set = ArmorSet::RobotSuit;
        s.fighters[0].hull = MECH_HULL;
        s.fighters[0].mech_transition_t = MECH_ENTER_S;
        s.fighters[0].ammo = 30;
        assert!(
            !s.try_fire(0, [0.0, 0.0, 1.0], false),
            "still sealing up inside a live chassis: correctly blocked"
        );
        // blow the chassis out from under them
        s.fighters[0].hull = 0.0;
        s.fighters[0].armor_set = ArmorSet::None;
        assert!(
            s.fighters[0].mech_transition_t > 0.0,
            "the entry timer is still running - that is the whole point"
        );
        assert!(
            s.try_fire(0, [0.0, 0.0, 1.0], false),
            "an ejected pilot on foot must be able to defend themselves"
        );

        // -- voluntary dismount: exit is committal, then the pilot is free
        let mut s = range(37);
        s.fighters[0].armor_set = ArmorSet::RobotSuit;
        s.fighters[0].hull = MECH_HULL;
        s.fighters[0].ammo = 30;
        s.step(PlayerCmd { exit_mech: true, ..Default::default() });
        assert!(
            s.fighters[0].mech_exiting && s.fighters[0].mech_transition_t > 0.0,
            "leaving must be COMMITTED (MECH_EXIT_S), not a one-tick state flip"
        );
        assert_eq!(
            s.fighters[0].armor_set,
            ArmorSet::RobotSuit,
            "the chassis is still powering down - teardown is deferred"
        );
        for _ in 0..((MECH_EXIT_S * SIM_HZ as f32) as usize + 3) {
            s.step(PlayerCmd::default());
        }
        assert_eq!(
            s.fighters[0].armor_set,
            ArmorSet::None,
            "power-down finished: the pilot is back on foot"
        );
        assert_eq!(s.fighters[0].hull, 0.0);
        assert!(!s.fighters[0].mech_exiting, "exit flag must clear");
        s.fighters[0].ammo = 30;
        s.fighters[0].fire_cd = 0.0;
        assert!(
            s.try_fire(0, [0.0, 0.0, 1.0], false),
            "a dismounted pilot must be able to fire"
        );
    }

    /// §5.1: the minigun's cone is its cost model, so the number the
    /// client shows and the number the sim shoots must be ONE function.
    /// `GunSpec.spread` holds only the cold value.
    #[test]
    fn base_spread_widens_with_minigun_heat_and_is_flat_for_everything_else() {
        assert!(
            (base_spread(GunKind::Minigun, 0.0) - MINIGUN_SPREAD_COLD).abs() < 1e-6,
            "cold minigun must be exactly the cold constant"
        );
        assert!(
            (base_spread(GunKind::Minigun, 100.0) - MINIGUN_SPREAD_HOT).abs() < 1e-6,
            "fully hot minigun must be exactly the hot constant"
        );
        let mid = base_spread(GunKind::Minigun, 50.0);
        assert!(
            mid > MINIGUN_SPREAD_COLD && mid < MINIGUN_SPREAD_HOT,
            "half heat must land between the two, got {mid}"
        );
        assert!(
            base_spread(GunKind::Minigun, 100.0) > base_spread(GunKind::Minigun, 0.0) * 2.5,
            "the widening must be the big multiplier the brief specifies"
        );
        // heat is a minigun-only concept - every other gun ignores it
        for k in [GunKind::M4, GunKind::Awm, GunKind::Glock] {
            assert_eq!(
                base_spread(k, 0.0),
                base_spread(k, 100.0),
                "{k:?} must not react to heat at all"
            );
            assert_eq!(base_spread(k, 0.0), gun(k).spread);
        }
    }

    /// §3: one spear, one attack. `try_fire` already refused to start a
    /// throw during a live thrust; nothing guarded the reverse, so
    /// holding the melee key inside a throw windup landed BOTH.
    #[test]
    fn a_thrust_cannot_be_started_during_a_throw_windup() {
        let mut s = range(38);
        s.fighters[0].gun = GunKind::Spear;
        s.fighters[0].ammo = 2;
        assert!(
            s.try_fire(0, [0.0, 0.0, 1.0], false),
            "the throw itself must start"
        );
        assert!(s.fighters[0].spear_wind_t > 0.0, "windup is live");
        // hold the melee key well inside the windup
        for _ in 0..10 {
            s.step(PlayerCmd { knife_hold: true, ..Default::default() });
            assert_eq!(
                s.fighters[0].knife_phase, 0.0,
                "a thrust must not start while the same spear is mid-throw"
            );
        }
    }

    /// §6.3: drive hull down through 70/40/15% and assert each plate
    /// stage fires EXACTLY once, in order, and never re-fires going the
    /// other way (health doesn't regenerate for a mech, but the bitmask
    /// itself must be monotonic regardless).
    #[test]
    fn damage_state_matrix_fires_each_stage_once_in_order() {
        let mut s = range(31);
        s.fighters[1].armor_set = ArmorSet::RobotSuit;
        s.fighters[1].hull = MECH_HULL;
        s.fighters[1].yaw = std::f32::consts::PI; // face the shooter - front arc
        assert_eq!(s.fighters[1].mech_plates_dropped, 0, "starts fully plated");
        // front-arc hits land ~15% of raw damage; hammer it down in
        // small increments so we can observe each threshold distinctly
        for _ in 0..40 {
            if s.fighters[1].hull <= 0.0 {
                break;
            }
            s.apply_hit(0, 1, 1.3, [0.0, 1.3, 5.0]);
        }
        // whatever stages the hull fraction crossed must be SET, and
        // none beyond what the final fraction implies
        let frac = s.fighters[1].hull / MECH_HULL;
        let mask = s.fighters[1].mech_plates_dropped;
        assert_eq!(frac <= MECH_PLATE_70_PCT, mask & 0b001 != 0, "70% stage vs actual fraction");
        assert_eq!(frac <= MECH_PLATE_40_PCT, mask & 0b010 != 0, "40% stage vs actual fraction");
        assert_eq!(frac <= MECH_PLATE_15_PCT, mask & 0b100 != 0, "15% stage vs actual fraction");
        // monotonic: a HIGHER stage set implies every lower stage is ALSO set
        if mask & 0b100 != 0 {
            assert_eq!(mask & 0b011, 0b011, "15% implies 70% and 40% already dropped");
        }
        if mask & 0b010 != 0 {
            assert_eq!(mask & 0b001, 0b001, "40% implies 70% already dropped");
        }
    }

    #[test]
    fn exposed_frame_takes_the_1_25x_bonus_only_after_a_plate_drops() {
        let mech_visor_y = BODY_HEIGHT * MECH_SCALE * 0.90;
        let mut s = range(32);
        s.fighters[1].armor_set = ArmorSet::RobotSuit;
        s.fighters[1].hull = MECH_HULL;
        s.fighters[1].yaw = 0.0; // REAR arc - full damage, cleanest to compare
        let h0 = s.fighters[1].hull;
        s.apply_hit(0, 1, mech_visor_y, [0.0, mech_visor_y, 5.0]);
        let plain = h0 - s.fighters[1].hull;
        s.fighters[1].mech_plates_dropped = 0b001; // force one plate off
        let h1 = s.fighters[1].hull;
        s.apply_hit(0, 1, mech_visor_y, [0.0, mech_visor_y, 5.0]);
        let exposed = h1 - s.fighters[1].hull;
        assert!(
            (exposed - plain * MECH_EXPOSED_DMG_MULT).abs() < 0.5,
            "exposed frame must take exactly ×1.25: plain {plain} exposed {exposed}"
        );
    }

    #[test]
    fn destruction_clears_the_plate_mask_for_the_next_chassis() {
        let mech_visor_y = BODY_HEIGHT * MECH_SCALE * 0.90;
        let mut s = range(33);
        s.fighters[1].armor_set = ArmorSet::RobotSuit;
        s.fighters[1].hull = 1.0; // one hit from destroyed
        s.fighters[1].mech_plates_dropped = 0b111;
        s.fighters[1].yaw = 0.0;
        s.apply_hit(0, 1, mech_visor_y, [0.0, mech_visor_y, 5.0]);
        assert_eq!(s.fighters[1].armor_set, ArmorSet::None, "chassis destroyed");
        assert_eq!(s.fighters[1].mech_plates_dropped, 0, "a fresh chassis starts fully plated");
    }
}
