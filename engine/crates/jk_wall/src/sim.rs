//! WallSim — two shield walls (or one wall vs an instrumented static wall)
//! in a Rapier world, stepped at the fixed 120 Hz combat tick.
//!
//! Behavioral model per active agent, per tick:
//!   1. lateral PD force to hold the file slot (formation discipline),
//!   2. forward drive toward the enemy: velocity-servo clamped by
//!      stamina-modulated max push — when blocked by contact this saturates
//!      and IS the othismos push,
//!   3. metabolic cost drains the two-pool stamina,
//!   4. sustained over-threshold chest load accumulates toward a crush-down.
//!
//! Nothing here applies an attenuation constant. α is measured, not authored.

use crate::agent::{Agent, DownCause, Side};
use crate::cohesion::{line_cohesion, LineCohesion};
use crate::combat::{resolve_strike, ArmorKind, StrikeOutcome, Weapon};
use crate::command::{PlayerInput, SquadCommand};
use crate::metrics::{StepMetrics, Telemetry};
use crate::stamina::Stamina;
use jk_core::constants as k;
use jk_core::timestep::DT;
use jk_core::Pcg32;
use rapier3d::prelude::*;

/// Who is carrying what — the army-composition knob.
///
/// The defaults reproduce the Era-1 historical distribution exactly, so
/// `WallSimConfig::default()` behaves bit-for-bit as it did before this
/// existed. A campaign layer that equips its own armies overrides these
/// rather than editing the spawn code.
///
/// Every field is a CUMULATIVE threshold against one `rng.next_f32()`
/// draw, matching the comparison the spawner actually performs. Storing
/// cumulative edges rather than per-class shares keeps the arithmetic
/// identical to the literals these replaced — it avoids re-deriving an
/// edge by addition, which is a rounding difference waiting to happen if
/// the numbers are ever retuned, and a changed edge means a changed
/// weapon for some man and a broken replay.
#[derive(Clone, Copy, Debug)]
pub struct KitDistribution {
    /// P(mail) in the front rank. Mail for the rich, and the rich stand in front.
    pub mail_front: f32,
    /// P(mail) in every rank behind the front.
    pub mail_rear: f32,
    /// Width of the gambeson band ABOVE the mail probability: a man is
    /// gambesoned when `r < p_mail + gambeson_span`. The remainder is cloth.
    pub gambeson_span: f32,
    /// Upper edge of the spear band: `wr < spear_max` carries a spear.
    pub spear_max: f32,
    /// Upper edge of the sword band: `wr < sword_max` carries a sword,
    /// anything above it carries an axe. Must be >= `spear_max`.
    pub sword_max: f32,
}

impl Default for KitDistribution {
    fn default() -> Self {
        KitDistribution {
            mail_front: k::MAIL_FRACTION_FRONT,
            mail_rear: k::MAIL_FRACTION_REAR,
            gambeson_span: 0.5,
            spear_max: 0.7,
            sword_max: 0.9,
        }
    }
}

pub struct WallSimConfig {
    pub files: usize,
    pub ranks_a: usize,
    /// 0 → side B is a fixed instrumented wall (α-measurement mode).
    pub ranks_b: usize,
    pub seed: u64,
    /// Gap between the two front ranks at spawn (m).
    pub start_gap_m: f32,
    /// March speed target before contact (m/s).
    pub advance_speed: f32,
    /// Force one side's armor kit (None = historical rng distribution).
    pub armor_a: Option<ArmorKind>,
    pub armor_b: Option<ArmorKind>,
    /// Army composition. Default = the Era-1 historical mix.
    pub kit: KitDistribution,
}

impl Default for WallSimConfig {
    fn default() -> Self {
        WallSimConfig {
            files: 8,
            ranks_a: 5,
            ranks_b: 5,
            seed: 0xC0FFEE,
            start_gap_m: 6.0,
            advance_speed: 1.1,
            armor_a: None,
            armor_b: None,
            kit: KitDistribution::default(),
        }
    }
}

/// A thrown spear in flight (or stuck in the field).
#[derive(Clone, Debug)]
pub struct Projectile {
    pub id: u32,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub side: Side,
    /// Some(t_stuck) once it's in the ground.
    pub stuck_t: Option<f32>,
    thrower: Option<usize>,
}

pub struct WallSim {
    pub cfg: WallSimConfig,
    pub agents: Vec<Agent>,
    pub telemetry: Telemetry,
    pub t: f32,
    /// Standing order per side (indexed A=0, B=1).
    pub side_command: [SquadCommand; 2],
    /// Index of the player-controlled agent, if any.
    pub player: Option<usize>,
    player_input: PlayerInput,
    /// Seeded stream for combat resolution (block rolls) — deterministic.
    combat_rng: Pcg32,
    /// Rotation drill bookkeeping per side: when it ends, what to resume,
    /// and the front-plane anchor the choreography works against (captured
    /// at drill start so the targets don't chase a drifting mean).
    rotate_until: [f32; 2],
    rotate_revert: [SquadCommand; 2],
    rotate_plane: [f32; 2],
    /// Sticky per-drill roles (1 = stepping back, 2 = stepping up) — set
    /// once when the drill starts; recomputing mid-pass jams the pair
    /// abreast the moment they draw level.
    drill_role: Vec<u8>,
    drill_assigned: [bool; 2],
    /// Falls scripted between ticks, queued so the next morale pass
    /// witnesses them: (side, x, z, was_the_commander).
    pending_witness: Vec<(Side, f32, f32, bool)>,
    /// Thrown spears in flight and stuck in the field.
    pub projectiles: Vec<Projectile>,
    next_projectile_id: u32,
    auto_volleyed: [bool; 2],

    bodies: RigidBodySet,
    colliders: ColliderSet,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd: CCDSolver,
    params: IntegrationParameters,

    /// Collider of the static wall in α-measurement mode.
    static_wall: Option<ColliderHandle>,
    /// Per-B-agent breach flags (so each man breaches once).
    breached_flags: Vec<bool>,
}

const AGENT_TAG_OFFSET: u128 = 1; // user_data = index + 1; 0 = static geometry

// Collision groups: bodies collide with everything except OWN-side shields;
// shields collide only with the enemy (and static geometry). Overlapping your
// neighbour's shield must be free — that overlap IS the wall.
const G_BODY_A: Group = Group::GROUP_1;
const G_SHIELD_A: Group = Group::GROUP_2;
const G_BODY_B: Group = Group::GROUP_3;
const G_SHIELD_B: Group = Group::GROUP_4;
const G_STATIC: Group = Group::GROUP_5;

fn body_groups(side: Side) -> InteractionGroups {
    match side {
        Side::A => InteractionGroups::new(
            G_BODY_A,
            G_BODY_A | G_BODY_B | G_SHIELD_B | G_STATIC,
        ),
        Side::B => InteractionGroups::new(
            G_BODY_B,
            G_BODY_B | G_BODY_A | G_SHIELD_A | G_STATIC,
        ),
    }
}

fn shield_groups(side: Side) -> InteractionGroups {
    match side {
        Side::A => InteractionGroups::new(G_SHIELD_A, G_BODY_B | G_SHIELD_B | G_STATIC),
        Side::B => InteractionGroups::new(G_SHIELD_B, G_BODY_A | G_SHIELD_A | G_STATIC),
    }
}

impl WallSim {
    pub fn new(cfg: WallSimConfig) -> Self {
        let mut rng = Pcg32::new(cfg.seed, 0x5EED);
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut agents = Vec::new();

        let half_gap = cfg.start_gap_m * 0.5;
        let spawn_side = |side: Side,
                              ranks: usize,
                              agents: &mut Vec<Agent>,
                              bodies: &mut RigidBodySet,
                              colliders: &mut ColliderSet,
                              rng: &mut Pcg32| {
            for rank in 0..ranks {
                for file in 0..cfg.files {
                    let slot_x =
                        (file as f32 - (cfg.files as f32 - 1.0) * 0.5) * k::FILE_SPACING_M;
                    let z0 = side.forward_sign()
                        * -(half_gap + rank as f32 * k::RANK_SPACING_M);
                    // Kit: mail for the rich, and the rich stand in front.
                    let forced = match side {
                        Side::A => cfg.armor_a,
                        Side::B => cfg.armor_b,
                    };
                    let mail_p = if rank == 0 {
                        cfg.kit.mail_front
                    } else {
                        cfg.kit.mail_rear
                    };
                    let armor = forced.unwrap_or_else(|| {
                        let r = rng.next_f32();
                        if r < mail_p {
                            ArmorKind::Mail
                        } else if r < mail_p + cfg.kit.gambeson_span {
                            ArmorKind::Gambeson
                        } else {
                            ArmorKind::Cloth
                        }
                    });
                    // shield ~3 kg + spear ~1.5 kg on top of body + armor
                    let mass = rng.range(k::BODY_MASS_KG.0, k::BODY_MASS_KG.1)
                        + armor.mass_kg()
                        + 4.5;
                    let body = RigidBodyBuilder::dynamic()
                        .translation(vector![
                            slot_x + rng.range(-0.06, 0.06),
                            0.0,
                            z0 + rng.range(-0.05, 0.05)
                        ])
                        .lock_rotations()
                        .enabled_translations(true, false, true)
                        .linear_damping(1.5)
                        .build();
                    let bh = bodies.insert(body);
                    let coll = ColliderBuilder::capsule_y(
                        (k::BODY_CAPSULE_HEIGHT_M - 2.0 * k::BODY_CAPSULE_RADIUS_M) * 0.5,
                        k::BODY_CAPSULE_RADIUS_M,
                    )
                    .friction(0.2)
                    .restitution(0.0)
                    .mass(mass) // exact body+gear mass, not density-derived
                    .collision_groups(body_groups(side))
                    .user_data(agents.len() as u128 + AGENT_TAG_OFFSET)
                    .build();
                    let ch = colliders.insert_with_parent(coll, bh, bodies);

                    // The shield: wide flat board held forward. Mass 0 — the
                    // shield's weight is already in gear mass.
                    // Half-thickness 0.10: the board plus the braced arm
                    // behind it — also the anti-tunneling margin for charges.
                    let shield = ColliderBuilder::cuboid(
                        0.5 * k::SHIELD_WIDTH_M,
                        0.5,
                        0.10,
                    )
                    .translation(vector![
                        0.0,
                        0.1,
                        side.forward_sign() * (k::BODY_CAPSULE_RADIUS_M + 0.10)
                    ])
                    .friction(0.3)
                    .restitution(0.0)
                    .mass(0.0)
                    .collision_groups(shield_groups(side))
                    .user_data(agents.len() as u128 + AGENT_TAG_OFFSET)
                    .build();
                    let sh = colliders.insert_with_parent(shield, bh, bodies);

                    let aerobic = k::AEROBIC_POWER_W * rng.range(0.85, 1.15);
                    let pool = k::ANAEROBIC_POOL_J * rng.range(0.85, 1.15);
                    // Weapon mix: spears rule the wall; some swords; a few
                    // axemen, who open armour the others cannot. NOT reach —
                    // the axe is the SHORTEST of the three (1.3 m against the
                    // spear's 1.9 m); what it brings is ~100 J against mail's
                    // 100 J threshold, where a spear lands 16.9 J.
                    let wr = rng.next_f32();
                    let weapon = if wr < cfg.kit.spear_max {
                        Weapon::spear()
                    } else if wr < cfg.kit.sword_max {
                        Weapon::sword()
                    } else {
                        Weapon::axe()
                    };
                    agents.push(Agent {
                        side,
                        rank,
                        file,
                        mass_kg: mass,
                        body: bh,
                        collider: ch,
                        shield: sh,
                        stamina: Stamina::new(aerobic, pool),
                        slot_x,
                        downed: false,
                        down_cause: None,
                        weapon,
                        armor,
                        wounds_j: 0.0,
                        strike_cooldown_s: rng.range(0.5, 2.0), // stagger openings
                        javelins: k::JAVELIN_COUNT,
                        throw_cooldown_s: 0.0,
                        strike_period_s: weapon.period_mult
                            * rng.range(k::STRIKE_PERIOD_S.0, k::STRIKE_PERIOD_S.1),
                        fear: 0.0,
                        rout_tolerance: rng
                            .range(k::ROUT_TOLERANCE.0, k::ROUT_TOLERANCE.1),
                        routing: false,
                        outnumber: 0.0,
                        kills: 0,
                        crush_exposure_s: 0.0,
                        crush_tolerance_s: rng
                            .range(k::CRUSH_TOLERANCE_S.0, k::CRUSH_TOLERANCE_S.1),
                        applied_push_n: 0.0,
                        frontal_contact_n: 0.0,
                        compression_n: 0.0,
                    });
                }
            }
        };

        spawn_side(Side::A, cfg.ranks_a, &mut agents, &mut bodies, &mut colliders, &mut rng);
        let static_wall = if cfg.ranks_b == 0 {
            // Instrumented fixed wall at z = 0 (α-measurement mode).
            let wall_body = bodies.insert(RigidBodyBuilder::fixed().build());
            let wall = ColliderBuilder::cuboid(
                cfg.files as f32 * k::FILE_SPACING_M,
                1.2,
                0.15,
            )
            .translation(vector![0.0, 0.0, 0.15])
            .friction(0.2)
            .collision_groups(InteractionGroups::new(G_STATIC, Group::ALL))
            .user_data(0)
            .build();
            Some(colliders.insert_with_parent(wall, wall_body, &mut bodies))
        } else {
            spawn_side(Side::B, cfg.ranks_b, &mut agents, &mut bodies, &mut colliders, &mut rng);
            None
        };

        let mut params = IntegrationParameters::default();
        params.dt = DT;
        // Bodies are flesh, not steel: soften contacts so files compress like
        // the brief's §2.2 nonlinear spring-damper chain instead of a rigid
        // train of billiard balls. 25 Hz keeps compliance without letting a
        // 3+ m/s charge tunnel through shield colliders; extra iterations
        // stabilize the tall contact stacks a two-wall scrum creates.
        params.contact_natural_frequency = 25.0;
        params.num_solver_iterations = std::num::NonZeroUsize::new(8).unwrap();

        let n_agents = agents.len();
        let combat_rng = Pcg32::new(cfg.seed, 0xC0B47);
        WallSim {
            cfg,
            agents,
            telemetry: Telemetry::default(),
            t: 0.0,
            side_command: [SquadCommand::Advance; 2],
            player: None,
            player_input: PlayerInput::default(),
            combat_rng,
            rotate_until: [0.0; 2],
            rotate_revert: [SquadCommand::Advance; 2],
            rotate_plane: [f32::NAN; 2],
            drill_role: vec![0; n_agents],
            drill_assigned: [false; 2],
            pending_witness: Vec::new(),
            projectiles: Vec::new(),
            next_projectile_id: 0,
            auto_volleyed: [false; 2],
            bodies,
            colliders,
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            params,
            static_wall,
            breached_flags: vec![false; n_agents],
        }
    }

    pub fn position(&self, a: &Agent) -> (f32, f32) {
        let b = &self.bodies[a.body];
        (b.translation().x, b.translation().z)
    }

    pub fn velocity(&self, a: &Agent) -> (f32, f32) {
        let b = &self.bodies[a.body];
        (b.linvel().x, b.linvel().z)
    }

    /// One fixed 120 Hz tick.
    pub fn step(&mut self) {
        // ---- -1. rotation drills expire -------------------------------
        for s in 0..2 {
            if self.side_command[s] == SquadCommand::Rotate && self.t >= self.rotate_until[s]
            {
                self.side_command[s] = self.rotate_revert[s];
                self.rotate_plane[s] = f32::NAN;
                self.drill_assigned[s] = false;
                let side = if s == 0 { Side::A } else { Side::B };
                for i in 0..self.agents.len() {
                    if self.agents[i].side == side {
                        self.drill_role[i] = 0;
                    }
                }
            }
        }

        // ---- 0. file discipline: who follows whom -----------------------
        // Dynamic rank: within each (side, file), the most-forward active man
        // leads; everyone else stays behind the man ahead and pushes HIM
        // rather than flowing around him. (Without this, the column dissolves
        // into a fluid and "rank" stops meaning anything.)
        // Rotation drill roles: 1 = tired leader stepping back, 2 = fresher
        // second stepping up. Decided PER FILE by comparing the two men's
        // stamina — once they swap, the comparison flips and the file
        // settles (no oscillation).
        let mut follow: Vec<Option<(f32, f32)>> = vec![None; self.agents.len()];
        for side in [Side::A, Side::B] {
            let s_idx = if side == Side::A { 0 } else { 1 };
            let assign_roles = self.side_command[s_idx] == SquadCommand::Rotate
                && !self.drill_assigned[s_idx];
            for file in 0..self.cfg.files {
                let mut men: Vec<(usize, f32)> = self
                    .agents
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.side == side && a.file == file && !a.downed)
                    .map(|(i, a)| (i, self.position(a).1 * side.forward_sign()))
                    .collect();
                men.sort_by(|p, q| q.1.partial_cmp(&p.1).unwrap());
                if assign_roles && men.len() >= 2 {
                    let (lead, second) = (men[0].0, men[1].0);
                    let lead_st = self.agents[lead].stamina.output_fraction();
                    let second_st = self.agents[second].stamina.output_fraction();
                    if lead_st + 0.05 < second_st {
                        self.drill_role[lead] = 1;
                        self.drill_role[second] = 2;
                    }
                }
                for w in men.windows(2) {
                    let (ahead_i, ahead_fwd) = w[0];
                    let (me_i, me_fwd) = w[1];
                    let vz_ahead = self.bodies[self.agents[ahead_i].body].linvel().z;
                    follow[me_i] = Some((vz_ahead, ahead_fwd - me_fwd));
                }
            }
            if assign_roles {
                self.drill_assigned[s_idx] = true;
            }
        }

        // ---- 1. behavior: forces --------------------------------------
        let profiles = [
            self.side_command[0].profile(self.cfg.advance_speed),
            self.side_command[1].profile(self.cfg.advance_speed),
        ];
        // Engagement planes: nobody sprints PAST the enemy line — you close
        // to it, then you hold and push. (Without this, chargers overrun the
        // flanks of a stationary wall and the battle dissolves.)
        let mean_front = |pts: &[(f32, f32)]| {
            if pts.is_empty() {
                f32::NAN
            } else {
                pts.iter().map(|p| p.1).sum::<f32>() / pts.len() as f32
            }
        };
        let enemy_plane = [
            mean_front(&self.front_line_points(Side::B)), // what A closes on
            mean_front(&self.front_line_points(Side::A)), // what B closes on
        ];
        // Anchor a starting drill to its side's CURRENT front plane, once.
        for s in 0..2 {
            if self.side_command[s] == SquadCommand::Rotate && self.rotate_plane[s].is_nan()
            {
                self.rotate_plane[s] = enemy_plane[1 - s]; // own front
            }
        }
        let player_input = self.player_input;
        for i in 0..self.agents.len() {
            let (x, z) = self.position(&self.agents[i]);
            let (vx, vz) = self.velocity(&self.agents[i]);
            let is_player = self.player == Some(i);
            let a = &mut self.agents[i];
            if a.downed {
                continue;
            }
            let m = a.mass_kg;
            let fwd = a.side.forward_sign();
            let prof = profiles[if a.side == Side::A { 0 } else { 1 }];

            // Brace degradation: pushing and withstanding compression share
            // one postural budget (crowd-crush physiology, research/01).
            // Bracing (feet set, shields locked) raises the tolerance.
            let brace_limit = k::BRACE_LIMIT_N * prof.brace_mult;
            let overload = ((a.compression_n - brace_limit) / brace_limit).max(0.0);
            let brace_factor = (1.0 - k::BRACE_DEGRADATION_SLOPE * overload)
                .clamp(k::BRACE_PUSH_FLOOR, 1.0);

            let (f_x, f_z, p_out);
            if is_player {
                // Third-person controlled soldier: a body in the crush like
                // everyone else. Input is a desired velocity; legs + stamina
                // + compression decide what actually happens. Baseline
                // authority is high — steering must feel crisp in open
                // ground; the CROWD is what resists you, not your own legs.
                let effort = 0.85 + 0.45 * player_input.push.clamp(0.0, 1.0);
                let f_cap = a.stamina.output_fraction()
                    * brace_factor
                    * k::PUSH_FORCE_BURST_N
                    * effort;
                let dvx = player_input.move_x - vx;
                let dvz = player_input.move_z - vz;
                let (mut fx, mut fz) = (m * k::SERVO_GAIN * dvx, m * k::SERVO_GAIN * dvz);
                let mag = (fx * fx + fz * fz).sqrt();
                if mag > f_cap {
                    fx *= f_cap / mag;
                    fz *= f_cap / mag;
                }
                a.applied_push_n = (fz * fwd).max(0.0);
                (f_x, f_z, p_out) = (fx, fz, 100.0 + (fx.abs() + fz.abs()) * 0.9);
            } else if a.routing {
                // ROUT: away from the enemy at flight speed. No push, no
                // slot, no order obeyed — only distance. The gap he leaves
                // is the cascade.
                let v_err = -k::ROUT_SPEED_M_S * fwd - vz;
                let cap = k::MAX_FOOT_FORCE_PER_KG * m;
                let fz = (m * k::SERVO_GAIN * v_err).clamp(-cap, cap);
                let fx = -m * k::PD_KD * vx; // just don't trip
                a.applied_push_n = 0.0;
                (f_x, f_z, p_out) = (fx, fz, 350.0); // sprinting scared
            } else {
                // Rotation drill: the tired leader offsets into the seam and
                // steps back; his second offsets the other way and presses
                // up. Once the second is more forward, the dynamic-rank
                // follow pass makes HIM the leader — the swap completes
                // itself. Fresh leaders (stamina > 0.6) stand fast.
                let rotating = self.side_command[if a.side == Side::A { 0 } else { 1 }]
                    == SquadCommand::Rotate;
                let mut slot_off = 0.0;
                let mut rot_v: Option<f32> = None;
                if rotating {
                    // Roles already carry the per-file freshness decision.
                    // The pair pass DIAGONALLY (offset ±0.35, exchanging
                    // depth) against the plane captured at drill start —
                    // positional targets, so nobody chases a drifting mean
                    // and nobody marches off alone. Servo: v = 2·Δz capped.
                    let anchor = self.rotate_plane[if a.side == Side::A { 0 } else { 1 }];
                    if anchor.is_finite() {
                        if self.drill_role[i] == 1 {
                            // Tired man: one rank back from the anchor.
                            slot_off = 0.35;
                            let dz = (anchor - k::RANK_SPACING_M * fwd - z) * fwd;
                            rot_v = Some((2.0 * dz).clamp(-0.6, 0.1));
                        } else if self.drill_role[i] == 2 {
                            // Fresh man: into the front slot at the anchor.
                            slot_off = -0.35;
                            let dz = (anchor - z) * fwd;
                            rot_v = Some((2.0 * dz).clamp(-0.1, 0.6));
                        }
                    }
                }

                // Lateral slot-holding PD (formation discipline). The budget
                // is finite on purpose: compressed columns must be able to
                // buckle sideways — that leakage is a real attenuation
                // mechanism. Brace tightens both the slots and the hold.
                let lat_cap =
                    prof.lateral_mult * k::LATERAL_HOLD_FRACTION * k::MAX_FOOT_FORCE_PER_KG * m;
                let slot = a.slot_x * prof.spacing_mult + slot_off;
                let f_lat =
                    (m * (k::PD_KP * (slot - x) - k::PD_KD * vx)).clamp(-lat_cap, lat_cap);

                // Forward drive: velocity servo with a gain high enough that
                // it SATURATES at the stamina- and brace-limited push when
                // blocked. The saturation is the othismos; the servo only
                // shapes marching. Command scales speed and push authority.
                // Followers LEAN (sustainable ~250 N) rather than max-shove —
                // that's what makes deep ranks a fresh RESERVE. A CHARGE
                // order opens the whole column to full effort: the surge.
                let charging = self.side_command
                    [if a.side == Side::A { 0 } else { 1 }]
                    == SquadCommand::Charge;
                // The man stepping up in a rotation needs his legs, not a
                // lean — exempt him from the follower cap for the drill.
                let is_follower = follow[i].is_some() && self.drill_role[i] != 2;
                let push_max = if is_follower && !charging {
                    k::FOLLOWER_LEAN_FORCE_N
                } else {
                    a.stamina.output_fraction()
                        * brace_factor
                        * prof.push_mult
                        * k::PUSH_FORCE_BURST_N
                };
                // Followers match the man ahead plus a lean-in that keeps
                // them pressed into his back; standing orders lean less.
                let lean_in = if prof.speed > 0.1 { 0.15 } else { 0.03 };
                let mut v_des_fwd = match (rot_v, follow[i]) {
                    (Some(v), _) => v, // rotation choreography overrides
                    (None, Some((vz_ahead, gap))) if gap < k::RANK_SPACING_M * 1.3 => {
                        vz_ahead * fwd + lean_in
                    }
                    _ => prof.speed,
                };
                // Anti-overrun: a man AT the enemy plane with NO ONE against
                // his shield is slipping past a flank — slow him. A man in
                // contact drives on; his servo saturating against the body
                // in front of him IS the push.
                let plane = enemy_plane[if a.side == Side::A { 0 } else { 1 }];
                if plane.is_finite()
                    && (plane - z) * fwd < 0.35
                    && a.frontal_contact_n < 150.0
                {
                    v_des_fwd = v_des_fwd.min(0.2);
                }
                let v_err = v_des_fwd * fwd - vz;
                let f_fwd_raw = m * k::SERVO_GAIN * v_err;
                // Push (toward enemy) limited by stamina; braking by legs.
                let f_fwd = if f_fwd_raw * fwd > 0.0 {
                    f_fwd_raw.clamp(-push_max, push_max)
                } else {
                    f_fwd_raw.clamp(
                        -k::MAX_FOOT_FORCE_PER_KG * m,
                        k::MAX_FOOT_FORCE_PER_KG * m,
                    )
                };
                a.applied_push_n = (f_fwd * fwd).max(0.0);
                (f_x, f_z, p_out) = (
                    f_lat,
                    f_fwd,
                    (100.0
                        + prof.extra_power_w
                        + a.applied_push_n * 0.9
                        + a.compression_n * k::COMPRESSION_POWER_W_PER_N)
                        .max(20.0),
                );
            }

            a.stamina.step(p_out, DT);
            let body = self.bodies.get_mut(a.body).unwrap();
            body.reset_forces(true);
            body.add_force(vector![f_x, 0.0, f_z], true);
        }

        // ---- 2. physics ------------------------------------------------
        self.pipeline.step(
            &vector![0.0, 0.0, 0.0],
            &self.params,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd,
            None,
            &(),
            &(),
        );
        self.t += DT;

        // ---- 3. read contact forces across the interface ---------------
        for a in &mut self.agents {
            a.frontal_contact_n = 0.0;
            a.compression_n = 0.0;
        }
        let mut interface_force = 0.0_f32;
        for pair in self.narrow_phase.contact_pairs() {
            let ud1 = self.colliders[pair.collider1].user_data;
            let ud2 = self.colliders[pair.collider2].user_data;
            let mut impulse_sum = 0.0_f32;
            for manifold in &pair.manifolds {
                for pt in &manifold.points {
                    impulse_sum += pt.data.impulse.abs();
                }
            }
            if impulse_sum == 0.0 {
                continue;
            }
            let force = impulse_sum / DT;
            let idx1 = ud1.checked_sub(AGENT_TAG_OFFSET).map(|v| v as usize);
            let idx2 = ud2.checked_sub(AGENT_TAG_OFFSET).map(|v| v as usize);
            match (idx1, idx2) {
                (Some(i), Some(j)) => {
                    self.agents[i].compression_n += force;
                    self.agents[j].compression_n += force;
                    if self.agents[i].side != self.agents[j].side {
                        interface_force += force;
                        self.agents[i].frontal_contact_n += force;
                        self.agents[j].frontal_contact_n += force;
                    }
                }
                (Some(i), None) | (None, Some(i)) => {
                    // Agent vs static wall (α-measurement mode).
                    interface_force += force;
                    self.agents[i].frontal_contact_n += force;
                    self.agents[i].compression_n += force;
                }
                (None, None) => {}
            }
        }

        // Snapshot for the morale pass: who was standing before the killing.
        let downed_before: Vec<bool> = self.agents.iter().map(|a| a.downed).collect();

        // ---- 3a. javelins: auto-volley, player throw, flight, impact ----
        // Both sides loose one volley of their own accord as the lines close
        // — the exchange of spears before the clash. Further volleys are the
        // commander's call (sim.volley / the client's key).
        for (s, side) in [(0usize, Side::A), (1usize, Side::B)] {
            if !self.auto_volleyed[s]
                && enemy_plane[s].is_finite()
                && enemy_plane[1 - s].is_finite()
                && (enemy_plane[s] - enemy_plane[1 - s]).abs() < k::VOLLEY_AUTO_RANGE_M
            {
                self.auto_volleyed[s] = true;
                self.volley(side);
            }
        }
        // The player's own aimed cast.
        if let Some(p) = self.player {
            self.agents[p].throw_cooldown_s =
                (self.agents[p].throw_cooldown_s - DT).max(0.0);
            if player_input.throw
                && !self.agents[p].downed
                && self.agents[p].javelins > 0
                && self.agents[p].throw_cooldown_s <= 0.0
            {
                let (x, z) = self.position(&self.agents[p]);
                let mag = (player_input.aim_x * player_input.aim_x
                    + player_input.aim_z * player_input.aim_z)
                    .sqrt()
                    .max(0.001);
                let d = player_input
                    .aim_dist
                    .clamp(k::JAVELIN_MIN_RANGE_M, k::JAVELIN_MAX_RANGE_M);
                let (tx, tz) = (
                    x + player_input.aim_x / mag * d,
                    z + player_input.aim_z / mag * d,
                );
                let side = self.agents[p].side;
                self.agents[p].javelins -= 1;
                self.agents[p].throw_cooldown_s = k::THROW_COOLDOWN_S;
                self.launch_javelin([x, 2.0, z], tx, tz, side, Some(p));
            }
        }
        // Flight + impact. Positions snapshot first (borrow discipline).
        let agent_snap: Vec<(Side, f32, f32, bool)> = self
            .agents
            .iter()
            .map(|a| {
                let (x, z) = self.position(a);
                (a.side, x, z, a.downed)
            })
            .collect();
        let braced_now = [
            self.side_command[0] == SquadCommand::Brace,
            self.side_command[1] == SquadCommand::Brace,
        ];
        let mut hits: Vec<(usize, f32, Option<usize>)> = Vec::new();
        let t_now = self.t;
        for pr in &mut self.projectiles {
            if pr.stuck_t.is_some() {
                continue;
            }
            pr.vel[1] -= k::GRAVITY_M_S2 * DT;
            pr.pos[0] += pr.vel[0] * DT;
            pr.pos[1] += pr.vel[1] * DT;
            pr.pos[2] += pr.vel[2] * DT;
            // descending through man-height: look for a body in the way
            if pr.vel[1] < 0.0 && pr.pos[1] < k::BODY_CAPSULE_HEIGHT_M {
                let mut best: Option<(usize, f32)> = None;
                for (j, &(js, jx, jz, jd)) in agent_snap.iter().enumerate() {
                    if jd || js == pr.side {
                        continue;
                    }
                    let dd = (jx - pr.pos[0]).powi(2) + (jz - pr.pos[2]).powi(2);
                    if dd < 0.16 && best.map_or(true, |(_, b)| dd < b) {
                        best = Some((j, dd));
                    }
                }
                if let Some((j, _)) = best {
                    let v2 = pr.vel[0] * pr.vel[0]
                        + pr.vel[1] * pr.vel[1]
                        + pr.vel[2] * pr.vel[2];
                    let energy = 0.5 * k::JAVELIN_MASS_KG * v2;
                    hits.push((j, energy, pr.thrower));
                    pr.stuck_t = Some(t_now); // spent — sticks where it hit
                    pr.pos[1] = pr.pos[1].max(0.05);
                    continue;
                }
            }
            if pr.pos[1] <= 0.0 {
                pr.pos[1] = 0.0;
                pr.stuck_t = Some(t_now);
            }
        }
        let ttl = k::JAVELIN_STUCK_TTL_S;
        let t_cur = self.t;
        self.projectiles
            .retain(|p| p.stuck_t.map_or(true, |s| t_cur - s < ttl));
        for (j, energy, thrower) in hits {
            if self.agents[j].downed {
                continue;
            }
            let overload = ((self.agents[j].compression_n - k::BRACE_LIMIT_N)
                / k::BRACE_LIMIT_N)
                .max(0.0);
            let side_idx = if self.agents[j].side == Side::A { 0 } else { 1 };
            let outcome = resolve_strike(
                energy,
                self.agents[j].armor,
                self.combat_rng.next_f32(),
                overload,
                braced_now[side_idx],
                self.agents[j].stamina.output_fraction(),
                self.agents[j].routing,
            );
            if let StrikeOutcome::Wound { penetrated_j } = outcome {
                self.agents[j].wounds_j += penetrated_j;
                if self.agents[j].wounds_j >= k::WOUND_DOWN_J {
                    self.agents[j].downed = true;
                    self.agents[j].down_cause = Some(DownCause::Wound);
                    if let Some(i) = thrower {
                        self.agents[i].kills += 1;
                    }
                }
            }
        }

        // ---- 3b. melee: committed strikes resolved by energy vs armor ---
        // (brief §2.5 — the metallurgy's socket into the battle.)
        let braced = [
            self.side_command[0] == SquadCommand::Brace,
            self.side_command[1] == SquadCommand::Brace,
        ];
        for i in 0..self.agents.len() {
            if self.agents[i].downed || self.agents[i].routing {
                continue; // routing men don't fight
            }
            self.agents[i].strike_cooldown_s -= DT;
            if self.agents[i].strike_cooldown_s > 0.0 {
                continue;
            }
            let is_player = self.player == Some(i);
            if is_player && !player_input.strike {
                continue; // the player strikes on input, not on a timer
            }
            // A spent man cannot commit a strike.
            let pool_cost = self.agents[i].weapon.pool_cost_j;
            if self.agents[i].stamina.pool_j < pool_cost {
                continue;
            }
            // Target: nearest standing enemy within reach, to the front.
            let (ax, az) = self.position(&self.agents[i]);
            let side = self.agents[i].side;
            let fwd = side.forward_sign();
            let reach = self.agents[i].weapon.reach_m;
            let mut target: Option<(usize, f32)> = None;
            for (j, d) in self.agents.iter().enumerate() {
                if d.side == side || d.downed {
                    continue;
                }
                let (dx, dz) = self.position(d);
                let (rx, rz) = (dx - ax, dz - az);
                if rz * fwd < 0.1 {
                    continue; // behind or beside
                }
                let dist = (rx * rx + rz * rz).sqrt();
                if dist <= reach && target.map_or(true, |(_, best)| dist < best) {
                    target = Some((j, dist));
                }
            }
            let Some((j, _)) = target else { continue };

            // Striker condition scales the thrust; defender condition and
            // protection decide what it does.
            let overload_att = ((self.agents[i].compression_n - k::BRACE_LIMIT_N)
                / k::BRACE_LIMIT_N)
                .max(0.0);
            let effort = self.agents[i].stamina.output_fraction()
                * (1.0 - 0.3 * overload_att.min(1.0));
            let energy = self.agents[i].weapon.strike_energy_j(effort);
            let overload_def = ((self.agents[j].compression_n - k::BRACE_LIMIT_N)
                / k::BRACE_LIMIT_N)
                .max(0.0);
            let def_side_idx = if self.agents[j].side == Side::A { 0 } else { 1 };
            let outcome = resolve_strike(
                energy,
                self.agents[j].armor,
                self.combat_rng.next_f32(),
                overload_def,
                braced[def_side_idx],
                self.agents[j].stamina.output_fraction(),
                self.agents[j].routing,
            );
            if let StrikeOutcome::Wound { penetrated_j } = outcome {
                self.agents[j].wounds_j += penetrated_j;
                if self.agents[j].wounds_j >= k::WOUND_DOWN_J {
                    self.agents[j].downed = true;
                    self.agents[j].down_cause = Some(DownCause::Wound);
                    self.agents[i].kills += 1;
                }
            }
            // The strike costs wind either way, and resets the tempo.
            self.agents[i].stamina.pool_j =
                (self.agents[i].stamina.pool_j - pool_cost).max(0.0);
            self.agents[i].strike_cooldown_s = self.agents[i].strike_period_s;
        }

        // ---- 4. crush injury accumulation -------------------------------
        for a in &mut self.agents {
            if a.downed {
                continue;
            }
            if a.compression_n > k::CRUSH_DOWN_FORCE_N {
                a.crush_exposure_s += DT;
            } else {
                a.crush_exposure_s = (a.crush_exposure_s - 0.5 * DT).max(0.0);
            }
            if a.crush_exposure_s > a.crush_tolerance_s {
                a.downed = true;
                a.down_cause = Some(DownCause::Crush);
            }
        }
        // ---- 4b. morale — "the most important system" -------------------
        // Fear flows from what a man SEES and FEELS: comrades falling beside
        // him, the crush on his chest, being locally outnumbered. Nerve
        // returns with quiet, distance, and the commander's presence.
        // Rout is per-man; the cascade is emergent.
        let mut newly_down: Vec<(Side, f32, f32, bool)> = (0..self.agents.len())
            .filter(|&i| !downed_before[i] && self.agents[i].downed)
            .map(|i| {
                let (x, z) = self.position(&self.agents[i]);
                (self.agents[i].side, x, z, self.player == Some(i))
            })
            .collect();
        newly_down.append(&mut std::mem::take(&mut self.pending_witness));

        // Local outnumbering refresh at 10 Hz (the "slow AI" tier).
        if (self.t / DT) as u64 % 12 == 0 {
            let positions: Vec<(Side, f32, f32, bool)> = self
                .agents
                .iter()
                .map(|a| {
                    let (x, z) = self.position(a);
                    (a.side, x, z, a.downed)
                })
                .collect();
            let r2 = k::LOCAL_RADIUS_M * k::LOCAL_RADIUS_M;
            for i in 0..self.agents.len() {
                if self.agents[i].downed {
                    continue;
                }
                let (side, x, z, _) = positions[i];
                let (mut allies, mut enemies) = (0.0_f32, 0.0_f32);
                for &(s2, x2, z2, down2) in &positions {
                    if down2 {
                        continue;
                    }
                    let (dx, dz) = (x2 - x, z2 - z);
                    if dx * dx + dz * dz < r2 {
                        if s2 == side {
                            allies += 1.0;
                        } else {
                            enemies += 1.0;
                        }
                    }
                }
                self.agents[i].outnumber = (enemies - allies).max(0.0);
            }
        }

        // Commander presence (the player, standing, is the aura source).
        let commander = self.player.filter(|&p| !self.agents[p].downed).map(|p| {
            let (x, z) = self.position(&self.agents[p]);
            (self.agents[p].side, x, z)
        });

        for i in 0..self.agents.len() {
            if self.agents[i].downed || self.player == Some(i) {
                continue; // the player's nerve is the human's problem
            }
            let (x, z) = self.position(&self.agents[i]);
            let side = self.agents[i].side;
            let a = &mut self.agents[i];

            // Witnessed falls.
            for &(ds, dx, dz, was_player) in &newly_down {
                let dist = ((dx - x).powi(2) + (dz - z).powi(2)).sqrt();
                if ds == side {
                    if dist < k::FEAR_WITNESS_RADIUS_M {
                        a.fear +=
                            k::FEAR_WITNESS_DOWN * (1.0 - dist / k::FEAR_WITNESS_RADIUS_M);
                    }
                    if was_player && dist < k::COMMANDER_DOWN_RADIUS_M {
                        a.fear += k::COMMANDER_DOWN_FEAR; // the banner wavers
                    }
                } else if dist < k::FEAR_WITNESS_RADIUS_M {
                    a.fear -=
                        k::CHEER_ENEMY_DOWN * (1.0 - dist / k::FEAR_WITNESS_RADIUS_M);
                }
            }
            // The crush and the odds.
            if a.compression_n > k::BRACE_LIMIT_N {
                a.fear += k::FEAR_COMPRESSION_PER_S * DT;
            }
            a.fear += k::FEAR_OUTNUMBER_PER_S * a.outnumber * DT;

            // Nerve recovers — faster near the commander.
            let mut recovery = k::FEAR_RECOVERY_PER_S;
            if let Some((cs, cx, cz)) = commander {
                if cs == side
                    && (cx - x).powi(2) + (cz - z).powi(2)
                        < k::COMMANDER_AURA_M * k::COMMANDER_AURA_M
                {
                    recovery *= k::COMMANDER_AURA_RECOVERY_MULT;
                }
            }
            if a.routing {
                recovery = k::FEAR_RECOVERY_PER_S * 3.0; // distance calms
            }
            a.fear = (a.fear - recovery * DT).clamp(0.0, 1.2);

            // Break / rally.
            if !a.routing && a.fear > a.rout_tolerance {
                a.routing = true;
            } else if a.routing && a.fear < k::RALLY_FEAR {
                a.routing = false; // shame and duty pull him back
            }
        }

        // Physically remove the newly downed (trampled underfoot — a gap).
        for i in 0..self.agents.len() {
            if self.agents[i].downed {
                if let Some(b) = self.bodies.get_mut(self.agents[i].body) {
                    if b.is_enabled() {
                        b.set_enabled(false);
                    }
                }
            }
        }

        // ---- 5. cohesion + breach detection -----------------------------
        let metrics = self.collect_metrics(interface_force);
        // Through `push`, not `steps.push`: a host that steps forever caps
        // history via `Telemetry::set_retention`, and a direct push would
        // silently bypass that.
        self.telemetry.push(metrics);
    }

    /// Current front line: per file, the most-forward active man; returns
    /// (agent_index, (x, z)) sorted by x.
    fn front_line(&self, side: Side) -> Vec<(usize, (f32, f32))> {
        let mut best: Vec<Option<(usize, f32, f32, f32)>> = vec![None; self.cfg.files];
        for (i, a) in self.agents.iter().enumerate() {
            if a.side != side || a.downed {
                continue;
            }
            let (x, z) = self.position(a);
            let fwdness = z * side.forward_sign();
            match best[a.file] {
                Some((_, _, _, bf)) if bf >= fwdness => {}
                _ => best[a.file] = Some((i, x, z, fwdness)),
            }
        }
        let mut pts: Vec<(usize, (f32, f32))> = best
            .iter()
            .flatten()
            .map(|&(i, x, z, _)| (i, (x, z)))
            .collect();
        pts.sort_by(|p, q| p.1 .0.partial_cmp(&q.1 .0).unwrap());
        pts
    }

    fn front_line_points(&self, side: Side) -> Vec<(f32, f32)> {
        self.front_line(side).into_iter().map(|(_, p)| p).collect()
    }

    fn collect_metrics(&mut self, interface_force: f32) -> StepMetrics {
        let max_ranks = self.cfg.ranks_a.max(self.cfg.ranks_b.max(1));
        let mut rank_push = [vec![0.0_f32; max_ranks], vec![0.0_f32; max_ranks]];
        let mut active = [0usize; 2];
        let mut stam = [0.0_f32; 2];
        let mut stam_n = [0usize; 2];
        let mut routing = [0usize; 2];
        let mut fear = [0.0_f32; 2];

        for a in &self.agents {
            let s = if a.side == Side::A { 0 } else { 1 };
            if !a.downed {
                active[s] += 1;
                rank_push[s][a.rank] += a.applied_push_n;
                stam[s] += a.stamina.output_fraction();
                fear[s] += a.fear;
                stam_n[s] += 1;
                if a.routing {
                    routing[s] += 1;
                }
            }
        }
        for s in 0..2 {
            if stam_n[s] > 0 {
                stam[s] /= stam_n[s] as f32;
                fear[s] /= stam_n[s] as f32;
            }
        }

        // The wall plane follows the CURRENT front line (whoever stands
        // there now), not the spawn-time rank-0 roster.
        let line_a = self.front_line(Side::A);
        let line_b = self.front_line(Side::B);
        let pts_a: Vec<(f32, f32)> = line_a.iter().map(|&(_, p)| p).collect();
        let pts_b: Vec<(f32, f32)> = line_b.iter().map(|&(_, p)| p).collect();
        let mean_z = |pts: &[(f32, f32)]| {
            if pts.is_empty() {
                f32::NAN
            } else {
                pts.iter().map(|p| p.1).sum::<f32>() / pts.len() as f32
            }
        };
        let front_z = [mean_z(&pts_a), mean_z(&pts_b)];
        // Load carried by the men actually holding the front.
        let mean_comp = |line: &[(usize, (f32, f32))]| {
            if line.is_empty() {
                0.0
            } else {
                line.iter()
                    .map(|&(i, _)| self.agents[i].compression_n)
                    .sum::<f32>()
                    / line.len() as f32
            }
        };
        let front_compression = [mean_comp(&line_a), mean_comp(&line_b)];

        let coh_a = line_cohesion(&pts_a);
        let coh_b = if self.cfg.ranks_b > 0 {
            line_cohesion(&pts_b)
        } else {
            LineCohesion::default()
        };

        // Breach: an enemy centroid past the defender's front plane with no
        // standing defender front-man within half a shield laterally.
        let mut breaches = Vec::new();
        if self.cfg.ranks_b > 0 {
            for (def_idx, def_side) in [(0usize, Side::A), (1usize, Side::B)] {
                let def_front = self.front_line_points(def_side);
                let plane = front_z[def_idx];
                if plane.is_nan() {
                    continue; // side annihilated — no wall left to breach
                }
                for (i, a) in self.agents.iter().enumerate() {
                    if a.side == def_side || a.downed || self.breached_flags[i] {
                        continue;
                    }
                    let (x, z) = self.position(a);
                    let behind =
                        (z - plane) * def_side.forward_sign() < -0.5;
                    if behind {
                        let covered = def_front
                            .iter()
                            .any(|&(dx, _)| (dx - x).abs() < 0.5 * k::SHIELD_WIDTH_M);
                        if !covered {
                            breaches.push((def_idx, x, z));
                            self.breached_flags[i] = true;
                        }
                    }
                }
            }
        }

        StepMetrics {
            t: self.t,
            interface_force_n: interface_force,
            rank_push_n: rank_push,
            cohesion: [coh_a, coh_b],
            active,
            front_z,
            front_compression,
            breaches,
            stamina_frac: stam,
            routing,
            mean_fear: fear,
        }
    }

    /// Launch one javelin from `from` toward `(tx, tz)` with deterministic
    /// scatter. High arc (clears own ranks); release speed solved from the
    /// distance, capped at a human throw.
    fn launch_javelin(&mut self, from: [f32; 3], tx: f32, tz: f32, side: Side, thrower: Option<usize>) {
        let sx = tx + self.combat_rng.range(-k::JAVELIN_SCATTER_M, k::JAVELIN_SCATTER_M);
        let sz = tz + self.combat_rng.range(-k::JAVELIN_SCATTER_M, k::JAVELIN_SCATTER_M);
        let (dx, dz) = (sx - from[0], sz - from[2]);
        let d = (dx * dx + dz * dz).sqrt().max(1.0);
        let (ux, uz) = (dx / d, dz / d);
        let theta = k::JAVELIN_ARC_RAD;
        let v0 = (k::GRAVITY_M_S2 * d / (2.0 * theta).sin())
            .sqrt()
            .min(k::JAVELIN_V0_MAX_MS);
        let (vy, vh) = (v0 * theta.sin(), v0 * theta.cos());
        let id = self.next_projectile_id;
        self.next_projectile_id += 1;
        self.projectiles.push(Projectile {
            id,
            pos: from,
            vel: [vh * ux, vy, vh * uz],
            side,
            stuck_t: None,
            thrower,
        });
    }

    /// The whole side casts together: every standing man with a javelin and
    /// an enemy line in range throws at it. Returns spears launched.
    pub fn volley(&mut self, side: Side) -> usize {
        let enemy = match side {
            Side::A => Side::B,
            Side::B => Side::A,
        };
        let enemy_front = self.front_line_points(enemy);
        if enemy_front.is_empty() {
            return 0;
        }
        let mut throws: Vec<([f32; 3], f32, f32, Option<usize>)> = Vec::new();
        for (i, a) in self.agents.iter().enumerate() {
            if a.side != side || a.downed || a.routing || a.javelins == 0 {
                continue;
            }
            let (x, z) = self.position(a);
            // aim at the nearest enemy front point
            let Some(&(tx, tz)) = enemy_front.iter().min_by(|p, q| {
                let dp = (p.0 - x).powi(2) + (p.1 - z).powi(2);
                let dq = (q.0 - x).powi(2) + (q.1 - z).powi(2);
                dp.partial_cmp(&dq).unwrap()
            }) else {
                continue;
            };
            let d = ((tx - x).powi(2) + (tz - z).powi(2)).sqrt();
            if !(k::JAVELIN_MIN_RANGE_M..=k::JAVELIN_MAX_RANGE_M).contains(&d) {
                continue;
            }
            throws.push(([x, 2.0, z], tx, tz, Some(i)));
        }
        let n = throws.len();
        for (from, tx, tz, thrower) in throws {
            if let Some(i) = thrower {
                self.agents[i].javelins -= 1;
            }
            self.launch_javelin(from, tx, tz, side, thrower);
        }
        n
    }

    /// Issue a standing order to one side's wall. `Rotate` is a timed drill:
    /// it runs for `ROTATE_DURATION_S`, then the previous order resumes.
    pub fn set_command(&mut self, side: Side, cmd: SquadCommand) {
        let s = if side == Side::A { 0 } else { 1 };
        if cmd == SquadCommand::Rotate && self.side_command[s] != SquadCommand::Rotate {
            self.rotate_revert[s] = self.side_command[s];
            self.rotate_until[s] = self.t + crate::command::ROTATE_DURATION_S;
            self.rotate_plane[s] = f32::NAN; // captured on the next tick
            self.drill_assigned[s] = false;
            let side = if s == 0 { Side::A } else { Side::B };
            for (i, a) in self.agents.iter().enumerate() {
                if a.side == side {
                    self.drill_role[i] = 0;
                }
            }
        }
        self.side_command[s] = cmd;
    }

    /// Take direct control of the man at (side, rank, file). Returns the
    /// agent index, or None if no such standing man exists.
    pub fn take_player(&mut self, side: Side, rank: usize, file: usize) -> Option<usize> {
        let idx = self
            .agents
            .iter()
            .position(|a| a.side == side && a.rank == rank && a.file == file && !a.downed)?;
        self.player = Some(idx);
        Some(idx)
    }

    /// Set the player's input for subsequent ticks (kept until changed).
    pub fn set_player_input(&mut self, input: PlayerInput) {
        self.player_input = input;
    }

    /// Force a specific agent down — scenario scripting (used by tests to
    /// fell the commander, among other cruelties).
    pub fn strike_down_agent(&mut self, idx: usize) {
        if idx < self.agents.len() && !self.agents[idx].downed {
            let (x, z) = self.position(&self.agents[idx]);
            self.pending_witness.push((
                self.agents[idx].side,
                x,
                z,
                self.player == Some(idx),
            ));
            self.agents[idx].downed = true;
            self.agents[idx].down_cause = Some(DownCause::Scripted);
        }
    }

    /// Force the man at (side, rank, file) down — scenario scripting for the
    /// cascade-collapse validation.
    pub fn strike_down(&mut self, side: Side, rank: usize, file: usize) {
        for i in 0..self.agents.len() {
            let a = &self.agents[i];
            if a.side == side && a.rank == rank && a.file == file && !a.downed {
                self.strike_down_agent(i);
                if let Some(b) = self.bodies.get_mut(self.agents[i].body) {
                    b.set_enabled(false);
                }
                return;
            }
        }
    }
}
