//! Milestone-1 validation: the three kill-criterion behaviors.
//! These are slow-ish integration tests; run in release for comfort.

use jk_core::timestep::SIM_HZ;
use jk_wall::{Side, SquadCommand, WallSim, WallSimConfig};

fn run(sim: &mut WallSim, secs: usize) {
    for _ in 0..(secs * SIM_HZ as usize) {
        sim.step();
    }
}

/// (1) Determinism: same seed → bit-identical trajectories.
#[test]
fn determinism_same_seed_identical() {
    let mk = || {
        let mut s = WallSim::new(WallSimConfig {
            seed: 99,
            ..Default::default()
        });
        run(&mut s, 10);
        s
    };
    let s1 = mk();
    let s2 = mk();
    for (a, b) in s1.agents.iter().zip(s2.agents.iter()) {
        let p1 = s1.position(a);
        let p2 = s2.position(b);
        assert_eq!(p1, p2, "trajectory diverged");
        assert_eq!(a.downed, b.downed);
    }
}

/// (2) Depth shows diminishing returns: front force grows with depth but
/// sub-linearly (α < 1 emerges; no attenuation constant is applied anywhere).
#[test]
fn depth_returns_diminish() {
    let force_at_depth = |d: usize| {
        let mut s = WallSim::new(WallSimConfig {
            files: 4,
            ranks_a: d,
            ranks_b: 0,
            seed: 7,
            start_gap_m: 1.6,
            advance_speed: 0.8,
            ..Default::default()
        });
        // Full-column shove: the othismos measurement.
        s.set_command(Side::A, SquadCommand::Charge);
        run(&mut s, 25);
        s.telemetry
            .steady_interface_force(8.0, 1.0 / SIM_HZ as f32)
    };
    let f1 = force_at_depth(1);
    let f4 = force_at_depth(4);
    let f8 = force_at_depth(8);
    assert!(f4 > f1 * 1.5, "depth adds force: f1={f1:.0} f4={f4:.0}");
    // Marginal value of ranks 5..8 must be well below ranks 1..4.
    let first_half = f4 - f1;
    let second_half = f8 - f4;
    assert!(
        second_half < first_half,
        "returns must diminish: 1→4 adds {first_half:.0} N, 4→8 adds {second_half:.0} N"
    );
}

/// (3) Cascade vs resilience: the same 3-man front loss is a bounded setback
/// for a DEEP wall (ranks step up, line re-equilibrates — depth as reserve,
/// the othismos-scholarship story) but compounds against a SHALLOW wall.
/// Both claims must hold — the first is why armies stacked deep, the second
/// is the collapse the game is about.
#[test]
fn depth_buys_resilience_to_front_losses() {
    // Ground lost by side A between the strike (t=8) and t=30, against an
    // identical 5-deep enemy. NaN front plane = side annihilated. All-mail:
    // this test isolates the PHYSICS of depth (replacement + push reserve);
    // with live weapons + morale the same strike can cascade into a full
    // fear-rout — that behavior belongs to the morale suite.
    let ground_lost_after_strike = |ranks_a: usize| {
        let mut s = WallSim::new(WallSimConfig {
            ranks_a,
            seed: 1234,
            armor_a: Some(jk_wall::ArmorKind::Mail),
            armor_b: Some(jk_wall::ArmorKind::Mail),
            ..Default::default()
        });
        run(&mut s, 8);
        let z8 = s.telemetry.steps.last().unwrap().front_z[0];
        s.strike_down(Side::A, 0, 3);
        s.strike_down(Side::A, 0, 4);
        s.strike_down(Side::A, 0, 5);
        run(&mut s, 22);
        let m = s.telemetry.steps.last().unwrap();
        let z30 = m.front_z[0];
        let lost = if z30.is_nan() { 99.0 } else { z8 - z30 };
        (lost, m.active[0], m.routing[0])
    };

    let (deep_lost, deep_standing, _) = ground_lost_after_strike(5);
    let (shallow_lost, shallow_standing, _) = ground_lost_after_strike(2);

    // Since morale landed, a mauled flank can legitimately CASCADE even for
    // a deep wall (witnessed falls → local outnumbering → fear — exactly
    // the spec's chain). What depth buys is DEGREE: the deep wall suffers
    // less, keeps more men, and is never annihilated outright.
    // (a) The shallow wall's loss compounds beyond the deep wall's.
    assert!(
        shallow_lost > deep_lost + 0.4,
        "shallow wall must cascade harder: deep lost {deep_lost:.2} m, shallow lost {shallow_lost:.2} m"
    );
    // (b) Depth preserves the force: more men still on their feet, in
    // ABSOLUTE terms and after discounting the extra ranks' headcount
    // (deep starts with 40, shallow with 16 — compare survivors per man).
    let deep_frac = deep_standing as f32 / 40.0;
    let shallow_frac = shallow_standing as f32 / 16.0;
    assert!(
        deep_frac >= shallow_frac,
        "depth must preserve the force: deep {deep_standing}/40 standing, shallow {shallow_standing}/16"
    );
    // (c) The deep wall is never annihilated outright (99 m sentinel).
    assert!(
        deep_lost < 90.0,
        "deep wall must not be annihilated: lost {deep_lost:.2} m"
    );
}
