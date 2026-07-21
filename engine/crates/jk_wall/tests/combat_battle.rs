//! Milestone-3 validation: melee outcomes must flow from energy vs armor —
//! the consequence chain the whole project exists for (brief §3.5).

use jk_core::timestep::SIM_HZ;
use jk_wall::{ArmorKind, DownCause, Side, WallSim, WallSimConfig};

fn run(sim: &mut WallSim, secs: usize) {
    for _ in 0..(secs * SIM_HZ as usize) {
        sim.step();
    }
}

fn wound_downs(sim: &WallSim, side: Side) -> usize {
    sim.agents
        .iter()
        .filter(|a| a.side == side && a.down_cause == Some(DownCause::Wound))
        .count()
}

/// No spear reaches across a 6 m gap: zero wounds before contact.
#[test]
fn no_wounds_before_contact() {
    let mut s = WallSim::new(WallSimConfig {
        seed: 3,
        ..Default::default()
    });
    run(&mut s, 2); // gap closes at ~2.2 m/s; still apart at t=2
    assert_eq!(
        wound_downs(&s, Side::A) + wound_downs(&s, Side::B),
        0,
        "wounds before contact"
    );
}

/// Armor is the metallurgy's voice in combat: an all-mail wall bleeds far
/// less than an all-cloth wall under identical spears. (Mail requires ~100 J
/// to defeat by a point; a spear thrust carries ~10–30 J — research/02.)
#[test]
fn mail_side_outlasts_cloth_side() {
    let mut s = WallSim::new(WallSimConfig {
        seed: 21,
        armor_a: Some(ArmorKind::Mail),
        armor_b: Some(ArmorKind::Cloth),
        ..Default::default()
    });
    run(&mut s, 60);
    let a_wounds = wound_downs(&s, Side::A);
    let b_wounds = wound_downs(&s, Side::B);
    assert!(
        b_wounds >= a_wounds.saturating_mul(3).max(3),
        "cloth must bleed: mail side lost {a_wounds} to wounds, cloth side {b_wounds}"
    );
}

/// Same seed, same battle, same wounds — combat randomness is seeded.
#[test]
fn combat_is_deterministic() {
    let outcome = || {
        let mut s = WallSim::new(WallSimConfig {
            seed: 77,
            ..Default::default()
        });
        run(&mut s, 30);
        let m = s.telemetry.steps.last().unwrap();
        (
            m.active[0],
            m.active[1],
            wound_downs(&s, Side::A),
            wound_downs(&s, Side::B),
        )
    };
    assert_eq!(outcome(), outcome(), "combat diverged across identical runs");
}
