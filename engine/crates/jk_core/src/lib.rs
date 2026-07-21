//! jk_core — deterministic foundations: fixed timestep, seeded RNG,
//! and the calibrated constants inherited from the SHIELDWALL: REFORGED
//! research program (projects/shieldwall_reforged/07_CONSTANTS.md).

pub mod constants;
pub mod rng;
pub mod timestep;

pub use rng::Pcg32;
pub use timestep::FixedTimestep;
