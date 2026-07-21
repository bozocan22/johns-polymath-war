//! Squad commands — the vocabulary the player issues to their wall — and the
//! player-input channel for the third-person controlled soldier.
//!
//! The original *Shieldwall* (Nezon) shipped three commands (follow, charge,
//! testudo) and reviewers found the tactics shallow. Our vocabulary maps to
//! the sim's real physics levers instead of stat toggles:
//! each command reweights speed, spacing, push authority, and stamina spend —
//! the outcomes (ground taken, cohesion held, men crushed) stay emergent.

use jk_core::constants as k;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SquadCommand {
    /// March on the enemy; push when blocked. The baseline.
    #[default]
    Advance,
    /// Stand fast: no forward drive, recover stamina, hold the line.
    Hold,
    /// Shields tight: close files, plant feet. Slower to yield ground,
    /// higher crush tolerance, no advance, minimal stamina spend.
    Brace,
    /// Burst forward at full effort. Momentum on impact, heavy stamina cost.
    Charge,
    /// Relieve the front: tired file leaders slide back through the seam
    /// while the second man steps up. Auto-reverts to the previous order
    /// after `ROTATE_DURATION_S`. This is how a wall fights for hours —
    /// the payoff of the two-pool stamina model. Files must OPEN to pass,
    /// so rotate out of contact: Withdraw first.
    Rotate,
    /// Fighting retreat: step back in order, faces to the enemy, shields
    /// up. Breaks contact for a rotation, refuses a flank, or trades ground
    /// for time. The wall's most demanding drill after the rotation itself.
    Withdraw,
}

/// How long a rotation drill runs before the standing order resumes —
/// long enough to open files, pass, and close again.
pub const ROTATE_DURATION_S: f32 = 6.0;

/// Per-tick behavior weights derived from a command.
#[derive(Clone, Copy, Debug)]
pub struct CommandProfile {
    /// Target march speed (m/s) along own forward axis.
    pub speed: f32,
    /// Multiplier on file spacing (1.0 = drill spacing; <1 = tighter).
    pub spacing_mult: f32,
    /// Multiplier on stamina-limited push authority.
    pub push_mult: f32,
    /// Multiplier on lateral hold budget (formation tightness under load).
    pub lateral_mult: f32,
    /// Multiplier on effective brace limit (compression tolerance while
    /// planted — shields locked, feet set).
    pub brace_mult: f32,
    /// Extra baseline metabolic load (W) on top of carry cost.
    pub extra_power_w: f32,
}

impl SquadCommand {
    pub fn profile(self, advance_speed: f32) -> CommandProfile {
        match self {
            SquadCommand::Advance => CommandProfile {
                speed: advance_speed,
                spacing_mult: 1.0,
                push_mult: 1.0,
                lateral_mult: 1.0,
                brace_mult: 1.0,
                extra_power_w: 0.0,
            },
            SquadCommand::Hold => CommandProfile {
                speed: 0.0,
                spacing_mult: 1.0,
                push_mult: 0.6,
                lateral_mult: 1.0,
                brace_mult: 1.0,
                extra_power_w: -50.0, // resting posture
            },
            SquadCommand::Brace => CommandProfile {
                speed: 0.0,
                spacing_mult: 0.8,
                push_mult: 0.5,
                lateral_mult: 1.8,
                brace_mult: 1.5,
                extra_power_w: -30.0,
            },
            SquadCommand::Charge => CommandProfile {
                speed: k::CHARGE_SPEED_M_S,
                spacing_mult: 1.0,
                push_mult: 1.25,
                lateral_mult: 0.7, // charging lines loosen — that's the risk
                brace_mult: 1.0,
                extra_power_w: 250.0,
            },
            SquadCommand::Withdraw => CommandProfile {
                speed: -0.5, // backward, in step, still facing the enemy
                spacing_mult: 1.0,
                push_mult: 0.8, // shields still working
                lateral_mult: 1.3,
                brace_mult: 1.0,
                extra_power_w: 40.0,
            },
            SquadCommand::Rotate => CommandProfile {
                speed: 0.0,
                // The pass is DIAGONAL: the pair offset ±0.35 laterally and
                // exchange depth positions, so they never need simultaneous
                // lateral clearance and the formation stays closed. Push
                // drops during the drill — rotate in a lull (Withdraw
                // first); under press the enemy exploits it.
                spacing_mult: 1.0,
                push_mult: 0.4,
                lateral_mult: 1.2,
                brace_mult: 1.0,
                extra_power_w: 0.0,
            },
        }
    }
}

/// External input for the player-controlled soldier, consumed each tick.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerInput {
    /// Desired velocity in world XZ (m/s); the sim clamps to what legs and
    /// stamina allow. The player is a body in the crush like everyone else.
    pub move_x: f32,
    pub move_z: f32,
    /// Extra push effort 0..1 (shoulder into the shield).
    pub push: f32,
    /// Commit a thrust this tick (edge-triggered by the client).
    pub strike: bool,
    /// Cast a javelin this tick toward the aim point (edge-triggered).
    pub throw: bool,
    /// Horizontal aim direction (unit-ish) and distance for the throw.
    pub aim_x: f32,
    pub aim_z: f32,
    pub aim_dist: f32,
}
