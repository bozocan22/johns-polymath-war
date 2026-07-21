// debug: (1) per-rank push/compression profile at depth 4;
//        (2) timeline of a front-rank strike-down in the 40v40.
use jk_core::timestep::SIM_HZ;
use jk_wall::{Side, WallSim, WallSimConfig};

fn main() {
    // ---- (1) chain profile --------------------------------------------
    let mut sim = WallSim::new(WallSimConfig {
        files: 4,
        ranks_a: 4,
        ranks_b: 0,
        seed: 0xA11A + 4,
        start_gap_m: 1.6,
        advance_speed: 0.8,
        ..Default::default()
    });
    for _ in 0..(30 * SIM_HZ) {
        sim.step();
    }
    println!("rank | mean z | push N | compression N | stamina");
    for rank in 0..4 {
        let (mut z, mut p, mut c, mut s, mut n) = (0.0, 0.0, 0.0, 0.0, 0);
        for a in &sim.agents {
            if a.rank == rank {
                z += sim.position(a).1;
                p += a.applied_push_n;
                c += a.compression_n;
                s += a.stamina.output_fraction();
                n += 1;
            }
        }
        let nf = n as f32;
        println!(
            "{rank}    | {:6.2} | {:6.0} | {:6.0} | {:.2}",
            z / nf,
            p / nf,
            c / nf,
            s / nf
        );
    }
    let f = sim.telemetry.steady_interface_force(5.0, 1.0 / SIM_HZ as f32);
    println!("interface force: {f:.0} N");

    // ---- (2) strike timeline ------------------------------------------
    println!("\nstrike timeline (3 A front-rankers down at t=8):");
    println!("t | omegaA mean/min | worst_gap | frontA_z | frontB_z | ifc kN | breaches");
    let mut sim = WallSim::new(WallSimConfig {
        seed: 1234,
        ..Default::default()
    });
    for _ in 0..(8 * SIM_HZ) {
        sim.step();
    }
    sim.strike_down(Side::A, 0, 3);
    sim.strike_down(Side::A, 0, 4);
    sim.strike_down(Side::A, 0, 5);
    for _ in 0..24 {
        for _ in 0..(SIM_HZ / 2) {
            sim.step();
        }
        let m = sim.telemetry.steps.last().unwrap();
        println!(
            "{:4.1} | {:.3}/{:.3} | {:4.2} | {:6.2} | {:6.2} | {:5.1} | {}",
            m.t,
            m.cohesion[0].mean_omega,
            m.cohesion[0].min_omega,
            m.cohesion[0].worst_gap_m,
            m.front_z[0],
            m.front_z[1],
            m.interface_force_n / 1000.0,
            sim.telemetry.total_breaches()
        );
    }

    // ---- (3) ground truth of the third-person demo at t=10 -------------
    use jk_wall::{PlayerInput, SquadCommand};
    println!("\nthird-person demo probe at t=10:");
    let mut sim = WallSim::new(WallSimConfig {
        seed: 0x3D,
        ..Default::default()
    });
    let player = sim.take_player(Side::A, 0, 4).unwrap();
    sim.set_player_input(PlayerInput { move_x: 0.0, move_z: 1.1, push: 0.3, strike: false, ..Default::default() });
    for tick in 0..(10 * SIM_HZ as usize) {
        if tick == 3 * SIM_HZ as usize {
            sim.set_command(Side::B, SquadCommand::Charge);
        }
        if tick == (3.4 * SIM_HZ as f32) as usize {
            sim.set_command(Side::A, SquadCommand::Brace);
            sim.set_player_input(PlayerInput { move_x: 0.0, move_z: 0.0, push: 1.0, strike: false, ..Default::default() });
        }
        sim.step();
    }
    let (px, pz) = sim.position(&sim.agents[player]);
    println!("player: x {px:.2} z {pz:.2} downed {}", sim.agents[player].downed);
    let mut cam = jk_spike::tp::Camera {
        eye: [px, 3.1, pz - 3.2],
        yaw: 0.0,
    };
    cam.follow([px, 0.0, pz], 0.0, 10.0);
    jk_spike::tp::render(&sim, &cam)
        .save("../output/dbg_tp10.png")
        .unwrap();
    println!("wrote ../output/dbg_tp10.png");
    println!("camera eye {:?}", cam.eye);
    for (label, idx) in [("player(A r0)", player), ("A r4 f4", 36), ("B r0 f4", 44)] {
        let a = &sim.agents[idx];
        let (x, z) = sim.position(a);
        let foot = cam.project([x, 0.0, z]);
        let head = cam.project([x, 1.72, z]);
        println!(
            "{label}: side {:?} rank {} world ({x:.2},{z:.2}) foot {foot:?} head {head:?}",
            a.side, a.rank
        );
    }
    for side in [Side::A, Side::B] {
        let (mut zs, mut n, mut zmin, mut zmax) = (0.0, 0, f32::MAX, f32::MIN);
        for a in sim.agents.iter().filter(|a| a.side == side && !a.downed) {
            let z = sim.position(a).1;
            zs += z; n += 1; zmin = zmin.min(z); zmax = zmax.max(z);
        }
        println!("side {side:?}: n {n}, z mean {:.2}, span [{zmin:.2}, {zmax:.2}]", zs / n as f32);
    }
}
