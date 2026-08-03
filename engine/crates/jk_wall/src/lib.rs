//! jk_wall — the shield wall as a physical object.
//!
//! Design contract (Pillar P2): cohesion, pressure, and collapse EMERGE from
//! the solver + per-agent behavior. No unit HP bar, no authored attenuation.
//! The per-rank attenuation α is *measured* from this sim, never applied.

pub mod agent;
pub mod cohesion;
pub mod combat;
pub mod command;
pub mod metrics;
pub mod sim;
pub mod stamina;

pub use agent::{Agent, DownCause, Side};
pub use combat::{ArmorKind, StrikeOutcome, Weapon};
pub use command::{PlayerInput, SquadCommand};
pub use metrics::{StepMetrics, Telemetry, LIVE_CLIENT_RETENTION_STEPS};
pub use sim::{KitDistribution, WallSim, WallSimConfig};
