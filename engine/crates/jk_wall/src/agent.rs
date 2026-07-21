//! Per-soldier state. The physics body lives in the backend; this is the
//! behavioral shell around it (slot assignment, stamina, injury accumulator).

use crate::combat::{ArmorKind, Weapon};
use crate::stamina::Stamina;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownCause {
    Crush,
    Wound,
    Scripted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    A,
    B,
}

impl Side {
    /// Forward direction along +z for A, -z for B.
    pub fn forward_sign(self) -> f32 {
        match self {
            Side::A => 1.0,
            Side::B => -1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub side: Side,
    /// 0 = front rank.
    pub rank: usize,
    pub file: usize,
    pub mass_kg: f32,
    pub body: RigidBodyHandle,
    pub collider: ColliderHandle,
    /// The shield: a wide flat collider on the same body, enemy-only
    /// collision groups — overlapping own-side shields is the point.
    pub shield: ColliderHandle,
    pub stamina: Stamina,
    /// Lateral slot coordinate (x) this agent tries to hold.
    pub slot_x: f32,
    pub downed: bool,
    pub down_cause: Option<DownCause>,
    /// What they carry and wear — the metallurgy's socket into combat.
    pub weapon: Weapon,
    pub armor: ArmorKind,
    /// Penetrated strike energy accumulated (J).
    pub wounds_j: f32,
    /// Seconds until the next committed strike is possible.
    pub strike_cooldown_s: f32,
    /// Throwing spears carried (cast in volleys or by the player's hand).
    pub javelins: u8,
    /// Player-only: seconds until the next aimed throw.
    pub throw_cooldown_s: f32,
    /// Individual strike tempo (s), seeded per man.
    pub strike_period_s: f32,
    pub kills: u32,
    /// Fear 0..~1.2 — the morale field (spec: "the most important system").
    pub fear: f32,
    /// This man's nerve: fear level at which he breaks.
    pub rout_tolerance: f32,
    /// Currently fleeing. Routing men don't push, barely block, and leave
    /// the front — the gaps they open are how armies actually collapse.
    pub routing: bool,
    /// Cached local outnumbering (enemy − ally within LOCAL_RADIUS),
    /// refreshed at 10 Hz like the rest of the slow AI.
    pub outnumber: f32,
    /// Seconds of sustained over-threshold chest load accumulated.
    pub crush_exposure_s: f32,
    /// Individual crush tolerance (varies per man, seeded).
    pub crush_tolerance_s: f32,
    /// Telemetry: forward push force applied this tick (N).
    pub applied_push_n: f32,
    /// Telemetry: contact force received from ENEMY bodies this tick (N).
    pub frontal_contact_n: f32,
    /// Telemetry: total body compression this tick — enemy AND friendly
    /// contact (N). Drives brace degradation and crush injury.
    pub compression_n: f32,
}

impl Agent {
    pub fn is_active(&self) -> bool {
        !self.downed
    }
}
