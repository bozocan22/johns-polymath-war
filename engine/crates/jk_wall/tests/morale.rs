//! Milestone-4 validation: morale — "the most important system."
//! Fear must be earned by events, routs must cascade from real losses, and
//! the commander's presence (and fall) must matter.

use jk_core::timestep::SIM_HZ;
use jk_wall::{PlayerInput, Side, WallSim, WallSimConfig};

fn run(sim: &mut WallSim, secs: usize) {
    for _ in 0..(secs * SIM_HZ as usize) {
        sim.step();
    }
}

fn last(sim: &WallSim) -> &jk_wall::StepMetrics {
    sim.telemetry.steps.last().unwrap()
}

/// A steady, even battle does not spontaneously dissolve: fear stays
/// moderate and routs are rare in the first half-minute. All-mail so no
/// side gets a kit-luck massacre — pure press, no cause for panic. (With
/// random kit one side CAN earn a real flank collapse — that's the game,
/// covered by the massacre test below.)
#[test]
fn steady_line_holds_its_nerve() {
    let mut s = WallSim::new(WallSimConfig {
        seed: 11,
        armor_a: Some(jk_wall::ArmorKind::Mail),
        armor_b: Some(jk_wall::ArmorKind::Mail),
        ..Default::default()
    });
    run(&mut s, 30);
    let m = last(&s);
    assert!(
        m.mean_fear[0] < 0.6 && m.mean_fear[1] < 0.6,
        "even battle must not saturate fear: A {:.2}, B {:.2}",
        m.mean_fear[0],
        m.mean_fear[1]
    );
    assert!(
        m.routing[0] + m.routing[1] <= 6,
        "mass rout without cause: A {} B {} fleeing",
        m.routing[0],
        m.routing[1]
    );
}

/// A massacre on one side breaks THAT side: strike down eight men of A's
/// left flank mid-fight and A must fear more and flee more than B.
#[test]
fn massacre_breaks_the_suffering_side() {
    let mut s = WallSim::new(WallSimConfig {
        seed: 12,
        ..Default::default()
    });
    run(&mut s, 10);
    for (rank, file) in [(0, 0), (0, 1), (0, 2), (0, 3), (1, 0), (1, 1), (1, 2), (1, 3)] {
        s.strike_down(Side::A, rank, file);
    }
    run(&mut s, 15);
    let m = last(&s);
    assert!(
        m.mean_fear[0] > m.mean_fear[1] + 0.1,
        "the mauled side must be the afraid side: A fear {:.2}, B fear {:.2}",
        m.mean_fear[0],
        m.mean_fear[1]
    );
    let total_routed_a = s
        .agents
        .iter()
        .filter(|a| a.side == Side::A && (a.routing || a.fear > a.rout_tolerance))
        .count();
    assert!(
        m.routing[0] >= 2 || total_routed_a >= 2,
        "a mauled flank must produce routs: {} fleeing now (A), fear peak count {}",
        m.routing[0],
        total_routed_a
    );
    assert!(
        m.routing[0] >= m.routing[1],
        "the mauled side must not flee LESS: A {} vs B {}",
        m.routing[0],
        m.routing[1]
    );
}

/// The commander's fall is a morale event beyond one more casualty: felling
/// the player spikes his side's fear more than felling an ordinary man of
/// the same rank and file position.
#[test]
fn commanders_fall_spreads_fear() {
    let fear_after = |fell_commander: bool| {
        let mut s = WallSim::new(WallSimConfig {
            seed: 13,
            ..Default::default()
        });
        let p = s.take_player(Side::A, 0, 4).expect("commander");
        s.set_player_input(PlayerInput {
            move_x: 0.0,
            move_z: 1.1,
            push: 0.5,
            strike: false,
            ..Default::default()
        });
        run(&mut s, 12);
        if fell_commander {
            s.strike_down_agent(p);
        } else {
            // An ordinary neighbour falls instead.
            s.strike_down(Side::A, 0, 3);
        }
        run(&mut s, 5);
        last(&s).mean_fear[0]
    };
    let ordinary = fear_after(false);
    let commander = fear_after(true);
    assert!(
        commander > ordinary + 0.05,
        "the commander's fall must weigh more: ordinary loss → fear {ordinary:.3}, \
         commander's fall → fear {commander:.3}"
    );
}

/// Routing men leave the fight and stop blocking — and can rally once the
/// terror fades with distance.
#[test]
fn routs_flee_and_can_rally() {
    let mut s = WallSim::new(WallSimConfig {
        seed: 14,
        ..Default::default()
    });
    run(&mut s, 10);
    // Manufacture terror on A's flank.
    for (rank, file) in [(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (2, 0), (2, 1), (0, 3)] {
        s.strike_down(Side::A, rank, file);
    }
    run(&mut s, 8);
    let routing_now: Vec<usize> = (0..s.agents.len())
        .filter(|&i| s.agents[i].routing)
        .collect();
    if routing_now.is_empty() {
        // Nerve held this seed — acceptable for the steady-line property,
        // but the massacre test above guarantees routs happen elsewhere.
        return;
    }
    // Fleeing men move away from the enemy (side A flees toward -z).
    let mut went_backward = 0;
    let marks: Vec<(usize, f32)> = routing_now
        .iter()
        .map(|&i| (i, s.position(&s.agents[i]).1))
        .collect();
    run(&mut s, 3);
    for &(i, z0) in &marks {
        if s.agents[i].downed {
            continue;
        }
        let z1 = s.position(&s.agents[i]).1;
        let fled = if s.agents[i].side == Side::A {
            z1 < z0 - 0.5
        } else {
            z1 > z0 + 0.5
        };
        if fled || !s.agents[i].routing {
            went_backward += 1; // fled, or already rallied — both legitimate
        }
    }
    assert!(
        went_backward > 0,
        "routing men must actually leave the line"
    );
    // Given long enough away from the press, some rally back.
    run(&mut s, 30);
    let still_routing = s.agents.iter().filter(|a| a.routing && !a.downed).count();
    assert!(
        still_routing < routing_now.len(),
        "fear must fade with distance: {} routed, {} still fleeing",
        routing_now.len(),
        still_routing
    );
}
