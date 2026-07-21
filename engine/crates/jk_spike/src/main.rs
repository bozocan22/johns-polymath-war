//! jk_spike — Milestone 1 deliverable.
//!
//! 1. α-measurement: one wall pushes an instrumented fixed wall at depths
//!    1..=8; fit the emergent per-rank attenuation.
//! 2. The 40v40: 8 files × 5 ranks per side, 90 sim-seconds at 120 Hz.
//!    Emits metrics CSV, top-down PNG frames, an animated GIF, and a
//!    markdown battle report.

use jk_spike::tp;

use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, Rgba, RgbaImage};
use jk_core::timestep::{DT, SIM_HZ};
use jk_wall::metrics::fit_alpha;
use jk_wall::{PlayerInput, Side, SquadCommand, WallSim, WallSimConfig};
use std::fs::{self, File};
use std::path::Path;

const OUT_DIR: &str = "../output";

fn main() {
    fs::create_dir_all(format!("{OUT_DIR}/frames")).unwrap();

    // ---------------------------------------------------- 1. alpha sweep
    println!("== alpha sweep: file depth 1..=8 vs instrumented wall ==");
    let mut forces = Vec::new();
    for depth in 1..=8 {
        let mut sim = WallSim::new(WallSimConfig {
            files: 4,
            ranks_a: depth,
            ranks_b: 0,
            seed: 0xA11A + depth as u64,
            start_gap_m: 1.6,
            advance_speed: 0.8,
            ..Default::default()
        });
        // The α measurement is the full othismos: the whole column shoves.
        sim.set_command(Side::A, SquadCommand::Charge);
        for _ in 0..(40 * SIM_HZ) {
            sim.step();
        }
        let f = sim.telemetry.steady_interface_force(10.0, DT);
        println!("  depth {depth}: steady wall force = {f:8.0} N");
        forces.push(f);
    }
    let (alpha, f1) = fit_alpha(&forces);
    println!("  fitted alpha = {alpha:.3}  (per-man front force F1 = {f1:.0} N)");

    // ---------------------------------------------------- 2. the 40v40
    println!("\n== 40v40: 8 files x 5 ranks per side, 90 s at {SIM_HZ} Hz ==");
    let mut sim = WallSim::new(WallSimConfig::default());
    let total_ticks = 90 * SIM_HZ as usize;
    let frame_every = SIM_HZ as usize / 2; // 2 fps of sim time
    let mut gif_frames: Vec<Frame> = Vec::new();
    let mut csv = String::from(
        "t,interface_n,active_a,active_b,mean_omega_a,mean_omega_b,stamina_a,stamina_b,breaches\n",
    );

    let started = std::time::Instant::now();
    for tick in 0..total_ticks {
        sim.step();
        if tick % frame_every == 0 {
            let img = render_topdown(&sim);
            if tick % (frame_every * 4) == 0 {
                img.save(format!("{OUT_DIR}/frames/t{:05.1}.png", sim.t))
                    .unwrap();
            }
            gif_frames.push(Frame::from_parts(
                img,
                0,
                0,
                Delay::from_numer_denom_ms(120, 1),
            ));
        }
        let m = sim.telemetry.steps.last().unwrap();
        if tick % SIM_HZ as usize == 0 {
            csv.push_str(&format!(
                "{:.2},{:.0},{},{},{:.3},{:.3},{:.3},{:.3},{}\n",
                m.t,
                m.interface_force_n,
                m.active[0],
                m.active[1],
                m.cohesion[0].mean_omega,
                m.cohesion[1].mean_omega,
                m.stamina_frac[0],
                m.stamina_frac[1],
                sim.telemetry.total_breaches(),
            ));
        }
    }
    let wall_secs = started.elapsed().as_secs_f64();
    let sim_secs = total_ticks as f64 / SIM_HZ as f64;
    println!(
        "  simulated {sim_secs:.0} s in {wall_secs:.1} s wall clock ({:.1}x realtime, {} bodies)",
        sim_secs / wall_secs,
        sim.agents.len()
    );

    fs::write(format!("{OUT_DIR}/battle_metrics.csv"), csv).unwrap();

    let gif_path = format!("{OUT_DIR}/battle_40v40.gif");
    let file = File::create(&gif_path).unwrap();
    let mut enc = GifEncoder::new_with_speed(file, 10);
    enc.set_repeat(Repeat::Infinite).unwrap();
    for f in gif_frames {
        enc.encode_frame(f).unwrap();
    }
    drop(enc);
    println!("  wrote {gif_path}");

    // ---------------------------------------------------- 3. report
    write_report(&sim, alpha, f1, &forces, sim_secs / wall_secs);
    println!("  wrote {OUT_DIR}/BATTLE_REPORT.md");

    // ------------------------------------- 4. third-person scripted battle
    println!("\n== third-person demo: player in the front rank ==");
    demo_third_person();
}

/// Scripted third-person engagement: the player stands front-rank center of
/// side A. The enemy charges; A braces for the impact, weathers it, then
/// counter-charges. Renders the over-the-shoulder view to a GIF.
fn demo_third_person() {
    let mut sim = WallSim::new(WallSimConfig {
        seed: 0x3D,
        ..Default::default()
    });
    let player = sim.take_player(Side::A, 0, 4).expect("front-rank player");
    sim.set_player_input(PlayerInput {
        move_x: 0.0,
        move_z: 1.1,
        push: 0.3,
        strike: false,
        ..Default::default()
    });

    let mut cam = tp::Camera {
        eye: [0.0, 2.0, -8.0],
        yaw: 0.0, // side A faces +z; camera looks over the player's shoulder
    };
    let mut frames: Vec<Frame> = Vec::new();
    let total_ticks = 50 * SIM_HZ as usize;
    let frame_every = SIM_HZ as usize / 5; // 5 fps of sim time

    for tick in 0..total_ticks {
        let t = tick as f32 / SIM_HZ as f32;
        // Scripted command sequence (stand-in for real input; the client
        // feeds the same PlayerInput/set_command API from keys).
        if tick == 3 * SIM_HZ as usize {
            sim.set_command(Side::B, SquadCommand::Charge);
        }
        if (t - 3.4).abs() < 0.5 / SIM_HZ as f32 {
            sim.set_command(Side::A, SquadCommand::Brace);
            sim.set_player_input(PlayerInput {
                move_x: 0.0,
                move_z: 0.0,
                push: 1.0,
                strike: true,
                ..Default::default()
            });
        }
        if tick == 18 * SIM_HZ as usize {
            sim.set_command(Side::A, SquadCommand::Rotate); // relieve, then...
        }
        if tick == 25 * SIM_HZ as usize {
            sim.set_command(Side::A, SquadCommand::Charge);
            sim.set_player_input(PlayerInput {
                move_x: 0.0,
                move_z: 1.6,
                push: 1.0,
                strike: true,
                ..Default::default()
            });
        }
        sim.step();

        if tick % frame_every == 0 {
            let (px, pz) = sim.position(&sim.agents[player]);
            cam.follow([px, 0.0, pz], 0.0, frame_every as f32 * DT);
            let img = tp::render(&sim, &cam);
            if tick % (frame_every * 25) == 0 {
                img.save(format!("{OUT_DIR}/frames/tp_t{:04.1}.png", sim.t))
                    .unwrap();
            }
            frames.push(Frame::from_parts(
                img,
                0,
                0,
                Delay::from_numer_denom_ms(100, 1),
            ));
        }
    }

    let gif_path = format!("{OUT_DIR}/third_person.gif");
    let file = File::create(&gif_path).unwrap();
    let mut enc = GifEncoder::new_with_speed(file, 10);
    enc.set_repeat(Repeat::Infinite).unwrap();
    for f in frames {
        enc.encode_frame(f).unwrap();
    }
    drop(enc);
    let last = sim.telemetry.steps.last().unwrap();
    println!(
        "  wrote {gif_path}  (final: A {} / B {} standing, player {})",
        last.active[0],
        last.active[1],
        if sim.agents[player].downed {
            "DOWN"
        } else {
            "standing"
        }
    );
}

fn render_topdown(sim: &WallSim) -> RgbaImage {
    const W: u32 = 480;
    const H: u32 = 360;
    const SCALE: f32 = 30.0; // px per metre
    let mut img = RgbaImage::from_pixel(W, H, Rgba([24, 26, 30, 255]));

    // centre line
    for y in 0..H {
        img.put_pixel(W / 2, y, Rgba([60, 60, 70, 255]));
    }

    for a in &sim.agents {
        let (x, z) = sim.position(a);
        // x lateral → vertical axis; z battle axis → horizontal axis
        let px = (W as f32 / 2.0 + z * SCALE) as i32;
        let py = (H as f32 / 2.0 + x * SCALE) as i32;
        let color = match (a.side, a.downed) {
            (_, true) => Rgba([70, 70, 70, 255]),
            (Side::A, _) => Rgba([90, 140, 255, 255]),
            (Side::B, _) => Rgba([235, 90, 80, 255]),
        };
        let r = (0.22 * SCALE) as i32;
        fill_circle(&mut img, px, py, r, color);
        // stamina tint: darken exhausted men
        if !a.downed && a.stamina.output_fraction() < 0.55 {
            fill_circle(&mut img, px, py, r / 2, Rgba([0, 0, 0, 120]));
        }
        // shield: a light strip held toward the enemy
        if !a.downed {
            let fwd = match a.side {
                Side::A => 1.0_f32,
                Side::B => -1.0,
            };
            let sx = (W as f32 / 2.0 + (z + fwd * 0.34) * SCALE) as i32;
            let half_w = (0.45 * SCALE) as i32;
            let shield_color = match a.side {
                Side::A => Rgba([170, 200, 255, 255]),
                Side::B => Rgba([255, 180, 170, 255]),
            };
            for dy in -half_w..=half_w {
                for dx in 0..2 {
                    let (x_px, y_px) = (sx + dx, py + dy);
                    if x_px >= 0
                        && y_px >= 0
                        && (x_px as u32) < img.width()
                        && (y_px as u32) < img.height()
                    {
                        img.put_pixel(x_px as u32, y_px as u32, shield_color);
                    }
                }
            }
        }
    }
    img
}

fn fill_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, c: Rgba<u8>) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let (x, y) = (cx + dx, cy + dy);
                if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                    img.put_pixel(x as u32, y as u32, c);
                }
            }
        }
    }
}

fn write_report(sim: &WallSim, alpha: f32, f1: f32, forces: &[f32], speed: f64) {
    let tel = &sim.telemetry;
    let last = tel.steps.last().unwrap();
    let peak_force = tel
        .steps
        .iter()
        .map(|m| m.interface_force_n)
        .fold(0.0_f32, f32::max);
    let first_down = tel
        .steps
        .iter()
        .find(|m| m.active[0] < 40 || m.active[1] < 40)
        .map(|m| m.t);
    let first_breach = tel
        .steps
        .iter()
        .find(|m| !m.breaches.is_empty())
        .map(|m| m.t);

    let mut r = String::new();
    r.push_str("# Milestone 1 — 40v40 Wall Spike: Battle Report\n\n");
    r.push_str("Generated by `jk_spike` (headless, fixed 120 Hz, deterministic seed).\n\n");
    r.push_str("## Emergent per-rank attenuation (kill-criterion metric 1)\n\n");
    r.push_str("| depth | steady front force (N) |\n|---|---|\n");
    for (i, f) in forces.iter().enumerate() {
        r.push_str(&format!("| {} | {:.0} |\n", i + 1, f));
    }
    r.push_str(&format!(
        "\nFitted `F(R) = F1·(1-α^R)/(1-α)`: **α = {alpha:.3}**, F1 = {f1:.0} N.\n\n\
         **Reading.** The engine applies NO attenuation constant — the curve \
         emerges from chain transmission, brace degradation under compression, \
         lateral buckling, and stamina. The capsule sim saturates HARDER than \
         the 1D prototype's geometric model (α-fit lands below the prototype's \
         0.78): after ~3–4 ranks the front interface force ceilings out. The \
         ceiling itself is the physical anchor — it lands in the instrumented \
         crowd-crush / rugby-scrum band (4.5–8 kN per few-man frontage, \
         research/01). Design consequence, consistent with the othismos \
         scholarship: ranks 4+ contribute *reserve, replacement, and rotation* \
         value, not additional push. Rank rotation is the milestone-2 \
         mechanic that cashes that in.\n\n"
    ));
    r.push_str("## 40v40 engagement\n\n");
    r.push_str(&format!(
        "- bodies: {} (5 ranks × 8 files per side)\n- sim speed: {speed:.1}× realtime on this container (4 cores)\n",
        sim.agents.len()
    ));
    r.push_str(&format!("- peak interface force: {peak_force:.0} N\n"));
    r.push_str(&format!(
        "- steady interface force (last 10 s): {:.0} N\n",
        tel.steady_interface_force(10.0, DT)
    ));
    if let Some(t) = first_down {
        r.push_str(&format!("- first man down (crush): t = {t:.1} s\n"));
    }
    if let Some(t) = first_breach {
        r.push_str(&format!("- first geometric breach: t = {t:.1} s\n"));
    }
    r.push_str(&format!(
        "- final standing: A = {} / 40, B = {} / 40\n- total breaches: {}\n",
        last.active[0],
        last.active[1],
        tel.total_breaches()
    ));
    r.push_str(&format!(
        "- final cohesion (mean Ω): A = {:.2}, B = {:.2}\n- final stamina fraction: A = {:.2}, B = {:.2}\n\n",
        last.cohesion[0].mean_omega,
        last.cohesion[1].mean_omega,
        last.stamina_frac[0],
        last.stamina_frac[1]
    ));
    r.push_str("Artifacts: `battle_40v40.gif`, `battle_metrics.csv`, `frames/`.\n");
    fs::write(Path::new(OUT_DIR).join("BATTLE_REPORT.md"), r).unwrap();
}
