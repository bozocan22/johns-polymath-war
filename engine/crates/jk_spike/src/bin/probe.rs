// probe: why don't front leaders tire in a mail-grind battle?
use jk_core::timestep::SIM_HZ;
use jk_wall::{ArmorKind, Side, WallSim, WallSimConfig};

fn probe_rotation() {
    use jk_wall::SquadCommand;
    let mut s = WallSim::new(WallSimConfig {
        seed: 31,
        armor_a: Some(ArmorKind::Mail),
        armor_b: Some(ArmorKind::Mail),
        ..Default::default()
    });
    for _ in 0..(60 * SIM_HZ as usize) { s.step(); }
    let leaders_of = |s: &WallSim| -> Vec<(usize, f32)> {
        let mut best: Vec<Option<(usize, f32)>> = vec![None; s.cfg.files];
        for (i, a) in s.agents.iter().enumerate() {
            if a.side != Side::A || a.downed { continue; }
            let z = s.position(a).1;
            match best[a.file] { Some((_, bz)) if bz >= z => {} _ => best[a.file] = Some((i, z)) }
        }
        best.iter().flatten().map(|&(i, _)| (i, s.agents[i].stamina.output_fraction())).collect()
    };
    println!("before rotate: {:?}", leaders_of(&s));
    for rank in 0..5 {
        let (mut st, mut comp, mut n) = (0.0, 0.0, 0);
        for a in s.agents.iter().filter(|a| a.side == Side::A && a.rank == rank && !a.downed) {
            st += a.stamina.output_fraction();
            comp += a.compression_n;
            n += 1;
        }
        println!(
            "  spawn-rank {rank}: mean stamina {:.2}, mean compression {:.0} N ({n} men)",
            st / n as f32,
            comp / n as f32
        );
    }
    s.set_command(Side::A, SquadCommand::Rotate);
    for tag in ["+2s", "+4s", "+10s"] {
        let n = if tag == "+10s" { 6 } else { 2 };
        for _ in 0..(n * SIM_HZ as usize) { s.step(); }
        println!("{tag}: {:?}", leaders_of(&s));
    }
}

fn probe_lull_rotate() {
    use jk_wall::SquadCommand;
    let mut s = WallSim::new(WallSimConfig {
        seed: 31,
        armor_a: Some(ArmorKind::Mail),
        armor_b: Some(ArmorKind::Mail),
        ..Default::default()
    });
    for _ in 0..(60 * SIM_HZ as usize) {
        s.step();
    }
    s.set_command(Side::A, SquadCommand::Hold);
    s.set_command(Side::B, SquadCommand::Hold);
    for _ in 0..SIM_HZ as usize {
        s.step();
    }
    s.set_command(Side::A, SquadCommand::Rotate);
    println!("\nlull-rotate timeline:");
    println!("t | omegaA | worst gap | frontA_z | frontB_z | ifc kN | Astand Bstand");
    for step in 0..(17 * SIM_HZ as usize) {
        if step == 7 * SIM_HZ as usize {
            s.set_command(Side::A, SquadCommand::Advance);
            s.set_command(Side::B, SquadCommand::Advance);
        }
        s.step();
        if step % (2 * SIM_HZ as usize) == 0 {
            let m = s.telemetry.steps.last().unwrap();
            println!(
                "{:5.1} | {:.3} | {:4.2} | {:6.2} | {:6.2} | {:5.1} | {} {}",
                m.t,
                m.cohesion[0].mean_omega,
                m.cohesion[0].worst_gap_m,
                m.front_z[0],
                m.front_z[1],
                m.interface_force_n / 1000.0,
                m.active[0],
                m.active[1]
            );
        }
    }
}

fn probe_drill_shape() {
    use jk_wall::SquadCommand;
    let mut s = WallSim::new(WallSimConfig {
        seed: 31,
        armor_a: Some(ArmorKind::Mail),
        armor_b: Some(ArmorKind::Mail),
        ..Default::default()
    });
    for _ in 0..(60 * SIM_HZ as usize) {
        s.step();
    }
    s.set_command(Side::A, SquadCommand::Withdraw);
    s.set_command(Side::B, SquadCommand::Hold);
    for _ in 0..(3 * SIM_HZ as usize) {
        s.step();
    }
    s.set_command(Side::A, SquadCommand::Hold);
    s.set_command(Side::A, SquadCommand::Rotate);
    let dump = |s: &WallSim, tag: &str| {
        println!("\n{tag} — side A men (file: rank z x | ...):");
        for file in 0..s.cfg.files {
            let mut men: Vec<(usize, f32, f32)> = s
                .agents
                .iter()
                .filter(|a| a.side == Side::A && a.file == file && !a.downed)
                .map(|a| {
                    let (x, z) = s.position(a);
                    (a.rank, x, z)
                })
                .map(|(r, x, z)| (r, z, x))
                .collect();
            men.sort_by(|p, q| q.1.partial_cmp(&p.1).unwrap());
            let row: Vec<String> = men
                .iter()
                .map(|&(r, z, x)| format!("r{r} z{z:5.2} x{x:5.2}"))
                .collect();
            println!("  f{file}: {}", row.join(" | "));
        }
    };
    for _ in 0..(9 * SIM_HZ as usize) {
        s.step();
    }
    dump(&s, "t=72 (drill done, closed?)");
    s.set_command(Side::A, SquadCommand::Advance);
    s.set_command(Side::B, SquadCommand::Advance);
    for _ in 0..(6 * SIM_HZ as usize) {
        s.step();
    }
    dump(&s, "t=78 (re-engaged)");
}

fn main() {
    probe_drill_shape();
    probe_rotation();
    probe_lull_rotate();
    let mut s = WallSim::new(WallSimConfig {
        seed: 31,
        armor_a: Some(ArmorKind::Mail),
        armor_b: Some(ArmorKind::Mail),
        ..Default::default()
    });
    println!("t | leaderA push N | comp N | stamina | pool J | ifc kN");
    for step in 0..(70 * SIM_HZ as usize) {
        s.step();
        if step % (10 * SIM_HZ as usize) == 0 {
            // current side-A file leaders = most-forward per file
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
            let ids: Vec<usize> = best.iter().flatten().map(|&(i, _)| i).collect();
            let n = ids.len() as f32;
            let push: f32 = ids.iter().map(|&i| s.agents[i].applied_push_n).sum::<f32>() / n;
            let comp: f32 = ids.iter().map(|&i| s.agents[i].compression_n).sum::<f32>() / n;
            let stam: f32 =
                ids.iter().map(|&i| s.agents[i].stamina.output_fraction()).sum::<f32>() / n;
            let pool: f32 = ids.iter().map(|&i| s.agents[i].stamina.pool_j).sum::<f32>() / n;
            let m = s.telemetry.steps.last().unwrap();
            println!(
                "{:3.0} | {:6.0} | {:6.0} | {:.3} | {:6.0} | {:5.1}",
                m.t,
                push,
                comp,
                stam,
                pool,
                m.interface_force_n / 1000.0
            );
        }
    }
}
