// bench — scale check toward the spec's 100/250/500/1000+ ladder.
// Full-fidelity bodies only (no LOD yet); research/01 puts the full-physics
// ceiling at ~400–700 bodies on 8 cores, with LOD beyond.
use jk_core::timestep::SIM_HZ;
use jk_wall::{WallSim, WallSimConfig};
use std::time::Instant;

fn main() {
    println!("bodies | files x ranks/side | sim 15 s in | x realtime");
    for (files, ranks) in [(8usize, 5usize), (16, 8), (25, 10), (32, 16)] {
        let mut sim = WallSim::new(WallSimConfig {
            files,
            ranks_a: ranks,
            ranks_b: ranks,
            seed: 0xBE7C,
            start_gap_m: 6.0,
            advance_speed: 1.1,
            ..Default::default()
        });
        let bodies = sim.agents.len();
        let t0 = Instant::now();
        for _ in 0..(15 * SIM_HZ as usize) {
            sim.step();
        }
        let wall = t0.elapsed().as_secs_f64();
        println!(
            "{bodies:6} | {files:2} x {ranks:2}          | {wall:7.2} s | {:6.1}x",
            15.0 / wall
        );
    }
}
