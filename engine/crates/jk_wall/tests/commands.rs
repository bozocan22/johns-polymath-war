//! Milestone-2 validation: squad commands and the player-controlled soldier
//! must change outcomes through the physics, not through stat toggles.

use jk_core::timestep::SIM_HZ;
use jk_wall::{ArmorKind, PlayerInput, Side, SquadCommand, WallSim, WallSimConfig};

fn run(sim: &mut WallSim, secs: usize) {
    for _ in 0..(secs * SIM_HZ as usize) {
        sim.step();
    }
}

fn cfg(seed: u64) -> WallSimConfig {
    WallSimConfig {
        seed,
        ..Default::default()
    }
}

/// All-mail: spears cannot pierce, so the battle is a pure grinding press —
/// the right lens for stamina/rotation properties without casualty churn.
fn cfg_grind(seed: u64) -> WallSimConfig {
    WallSimConfig {
        seed,
        armor_a: Some(ArmorKind::Mail),
        armor_b: Some(ArmorKind::Mail),
        ..Default::default()
    }
}

/// A charging wall hits harder than an advancing one: peak interface force
/// (the collision) and early ground gained must both rise.
#[test]
fn charge_hits_harder_than_advance() {
    // Peak over the collision window only — later casualty surges would
    // drown the comparison.
    let peak_force = |cmd: SquadCommand| {
        let mut s = WallSim::new(cfg(42));
        s.set_command(Side::A, cmd);
        run(&mut s, 6);
        s.telemetry
            .steps
            .iter()
            .map(|m| m.interface_force_n)
            .fold(0.0_f32, f32::max)
    };
    let advance = peak_force(SquadCommand::Advance);
    let charge = peak_force(SquadCommand::Charge);
    assert!(
        charge > advance * 1.2,
        "charge must spike impact force: advance peak {advance:.0} N, charge peak {charge:.0} N"
    );
}

/// A braced wall receiving a charge yields less ground than one merely
/// holding: planted feet and locked shields must matter physically.
#[test]
fn brace_resists_charge_better_than_hold() {
    let ground_lost = |cmd: SquadCommand| {
        let mut s = WallSim::new(cfg(77));
        s.set_command(Side::A, cmd);
        s.set_command(Side::B, SquadCommand::Charge);
        run(&mut s, 20);
        // Side A faces +z; more negative front plane = driven back further.
        -s.telemetry.steps.last().unwrap().front_z[0]
    };
    let hold = ground_lost(SquadCommand::Hold);
    let brace = ground_lost(SquadCommand::Brace);
    assert!(
        brace < hold - 0.1,
        "brace must yield less ground: hold lost {hold:.2} m, brace lost {brace:.2} m"
    );
}

/// Mean stamina of side A's CURRENT front-line men (per file, the most
/// forward standing man) — the men doing the fighting.
fn front_stamina(s: &WallSim) -> f32 {
    let mut best: Vec<Option<(usize, f32)>> = vec![None; s.cfg.files];
    for (i, a) in s.agents.iter().enumerate() {
        if a.side != Side::A || a.downed {
            continue;
        }
        let z = s.position(a).1;
        match best[a.file] {
            Some((_, bz)) if bz >= z => {}
            _ => best[a.file] = Some((i, z)),
        }
    }
    let leaders: Vec<usize> = best.iter().flatten().map(|&(i, _)| i).collect();
    leaders
        .iter()
        .map(|&i| s.agents[i].stamina.output_fraction())
        .sum::<f32>()
        / leaders.len().max(1) as f32
}

/// Holding recovers stamina relative to shoving — measured at the FRONT,
/// where the difference lives (rear leaners barely tire either way).
#[test]
fn hold_recovers_stamina() {
    let stamina_after = |cmd: SquadCommand| {
        let mut s = WallSim::new(cfg_grind(5));
        run(&mut s, 20); // fight under Advance — the front drains
        s.set_command(Side::A, cmd);
        run(&mut s, 20);
        front_stamina(&s)
    };
    let keep_pushing = stamina_after(SquadCommand::Advance);
    let hold = stamina_after(SquadCommand::Hold);
    assert!(
        hold > keep_pushing + 0.02,
        "hold must recover the front: advance {keep_pushing:.3}, hold {hold:.3}"
    );
}

/// Rotation relieves the front: after a long grind, ordering Rotate during
/// a lull must put fresher men on the line for the next clash — without the
/// wall falling apart. (Rotating under full press lets the enemy pour into
/// the opened seams; the sim punishes it, and history agrees — you rotate
/// when the pulse ebbs.)
#[test]
fn rotation_relieves_the_front() {
    // The full historical drill: WITHDRAW to break contact, ROTATE in the
    // gap (files must open — a man is wider than the closed seam), close
    // ranks, and meet the next clash with a fresh front. Both scenarios
    // share the same withdraw-lull; only the rotation differs.
    let scenario = |rotate: bool| {
        let mut s = WallSim::new(cfg_grind(31));
        run(&mut s, 60); // grind the front down
        s.set_command(Side::A, SquadCommand::Withdraw); // step back in order
        s.set_command(Side::B, SquadCommand::Hold); // the pulse ebbs
        run(&mut s, 3);
        s.set_command(Side::A, SquadCommand::Hold);
        if rotate {
            s.set_command(Side::A, SquadCommand::Rotate); // reverts to Hold
        }
        run(&mut s, 9); // drill (6 s) + files closing, out of contact
        // The moment that matters: who is holding the front as battle
        // resumes? (Given a LONG lull everyone would heal — rotation's edge
        // is a fresh front for the NEXT pulse, not eventually.)
        let stamina_at_reengage = front_stamina(&s);
        s.set_command(Side::A, SquadCommand::Advance); // battle resumes
        s.set_command(Side::B, SquadCommand::Advance);
        run(&mut s, 6);
        (
            stamina_at_reengage,
            s.telemetry.steps.last().unwrap().cohesion[0].mean_omega,
        )
    };

    let (straight, control_omega) = scenario(false);
    let (rotated, rotated_omega) = scenario(true);
    assert!(
        rotated > straight + 0.03,
        "rotation must relieve the front: straight-through {straight:.3}, rotated {rotated:.3}"
    );
    // Re-engagement is ragged in ANY scenario (a resumed battle is a
    // collision, not a parade); the drill must simply not leave the line
    // materially worse than not drilling.
    assert!(
        rotated_omega > control_omega - 0.1,
        "the drill must not wreck the line: control Ω {control_omega:.3}, rotated Ω {rotated_omega:.3}"
    );
}

/// The player is a real body: input moves them, determinism holds with a
/// scripted input, and walking into the crush gets them compressed.
#[test]
fn player_is_a_body_in_the_crush() {
    let run_player = || {
        let mut s = WallSim::new(cfg(9));
        let p = s.take_player(Side::A, 0, 4).expect("player exists");
        // March with the line, shoulder in.
        s.set_player_input(PlayerInput {
            move_x: 0.0,
            move_z: 1.1,
            push: 1.0,
            strike: false,
            ..Default::default()
        });
        run(&mut s, 15);
        (
            s.position(&s.agents[p]),
            s.agents[p].compression_n,
            s.agents[p].downed,
        )
    };
    let ((x1, z1), c1, d1) = run_player();
    let ((x2, z2), c2, d2) = run_player();
    // Determinism with scripted input.
    assert_eq!(
        (x1, z1, c1, d1),
        (x2, z2, c2, d2),
        "player run must be deterministic"
    );
    // They advanced with the wall and the battle touched them: either the
    // press has them under load, or the spears found them. A player who
    // walks into the front rank and feels NOTHING would be the bug.
    assert!(z1 > -2.0, "player must have advanced: z {z1:.2}");
    assert!(x1.abs() < 2.0, "player stayed near their file: x {x1:.2}");
    assert!(
        c1 > 200.0 || d1,
        "front-rank player must feel the battle: {c1:.0} N, downed {d1}"
    );
}
