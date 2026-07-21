//! Javelin volleys: ballistic, deterministic, and deadly to the unarmored.

use jk_core::timestep::SIM_HZ;
use jk_wall::{ArmorKind, DownCause, Side, SquadCommand, WallSim, WallSimConfig};

fn run(sim: &mut WallSim, secs: usize) {
    for _ in 0..(secs * SIM_HZ as usize) {
        sim.step();
    }
}

fn volley_scenario() -> WallSim {
    let mut s = WallSim::new(WallSimConfig {
        seed: 99,
        start_gap_m: 20.0, // inside javelin range, outside auto-volley range
        armor_a: Some(ArmorKind::Mail),
        armor_b: Some(ArmorKind::Cloth),
        ..Default::default()
    });
    s.set_command(Side::A, SquadCommand::Hold);
    s.set_command(Side::B, SquadCommand::Hold);
    run(&mut s, 1);
    s
}

/// The wall casts together: most men launch, spears arc, the unarmored
/// suffer. And the same seed produces the same slaughter.
#[test]
fn volley_flies_and_wounds() {
    let outcome = || {
        let mut s = volley_scenario();
        let launched = s.volley(Side::A);
        run(&mut s, 5); // flight + landings
        let b_wounded = s
            .agents
            .iter()
            .filter(|a| a.side == Side::B && a.down_cause == Some(DownCause::Wound))
            .count();
        (launched, b_wounded)
    };
    let (launched, b_wounded) = outcome();
    assert!(
        launched >= 30,
        "most of the 40 should be in range and armed: {launched} launched"
    );
    assert!(
        b_wounded >= 2,
        "a volley into cloth must draw blood: {b_wounded} down"
    );
    assert_eq!(outcome(), (launched, b_wounded), "volley must be deterministic");
}

/// Two javelins per man: the third volley is empty hands.
#[test]
fn javelins_run_out() {
    let mut s = volley_scenario();
    let v1 = s.volley(Side::A);
    run(&mut s, 3);
    let v2 = s.volley(Side::A);
    run(&mut s, 3);
    let v3 = s.volley(Side::A);
    assert!(v1 >= 30 && v2 >= 20, "two real volleys: {v1}, {v2}");
    assert_eq!(v3, 0, "no third volley: {v3} launched");
}
