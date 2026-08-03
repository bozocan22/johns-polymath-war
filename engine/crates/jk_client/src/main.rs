//! jk_client — the playable 3D third-person client.
//!
//! You are a soldier in the front rank of a 40v40 shield wall. Fight the
//! press with your body; command the wall with one hand.
//!
//! Controls (per research/05 recommendations):
//!   mouse        look (yaw)
//!   W/A/S/D      move (camera-relative)
//!   LEFT SHIFT   shoulder into the shield (full push effort)
//!   1/2/3/4      squad order: Advance / Hold / Brace / Charge
//!   SPACE        when down: take over the nearest standing man
//!   ESC          release the mouse
//!
//! The sim runs the same deterministic 120 Hz tick as the headless spike;
//! rendering interpolates nothing yet (M3 will add state interpolation).

use jk_core::timestep::DT;
use jk_wall::{
    PlayerInput, Side, SquadCommand, WallSim, WallSimConfig, LIVE_CLIENT_RETENTION_STEPS,
};
use macroquad::prelude::*;

const CAM_DIST: f32 = 3.8;
const CAM_HEIGHT: f32 = 3.8;
const CAM_STIFFNESS: f32 = 4.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "John Kingdom Game — Shield Wall".to_string(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

struct ClientCam {
    eye: Vec3,
    yaw: f32,
}

impl ClientCam {
    fn follow(&mut self, target: Vec3, dt: f32) {
        let desired = vec3(
            target.x - self.yaw.sin() * CAM_DIST,
            target.y + CAM_HEIGHT,
            target.z - self.yaw.cos() * CAM_DIST,
        );
        self.eye += (desired - self.eye) * (CAM_STIFFNESS * dt).min(1.0);
    }

    fn camera3d(&self, target: Vec3) -> Camera3D {
        Camera3D {
            position: self.eye,
            target: vec3(target.x, 1.0, target.z),
            up: vec3(0.0, 1.0, 0.0),
            ..Default::default()
        }
    }
}

fn side_color(side: Side, is_player: bool, downed: bool, routing: bool) -> Color {
    if downed {
        return Color::from_rgba(60, 58, 55, 255);
    }
    if is_player {
        return Color::from_rgba(235, 190, 70, 255);
    }
    if routing {
        // fear has a color: drained, jaundiced, running
        return match side {
            Side::A => Color::from_rgba(120, 130, 160, 255),
            Side::B => Color::from_rgba(160, 120, 110, 255),
        };
    }
    match side {
        Side::A => Color::from_rgba(85, 125, 215, 255),
        Side::B => Color::from_rgba(200, 75, 65, 255),
    }
}

fn shield_color(side: Side) -> Color {
    match side {
        Side::A => Color::from_rgba(150, 175, 230, 255),
        Side::B => Color::from_rgba(225, 150, 135, 255),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut sim = WallSim::new(WallSimConfig::default());
    // This client never stops stepping — cap history or it grows without
    // bound for as long as the window is open.
    sim.telemetry
        .set_retention(Some(LIVE_CLIENT_RETENTION_STEPS));
    let mut player = sim.take_player(Side::A, 0, 4).expect("player slot");
    let mut cam = ClientCam {
        eye: vec3(0.0, CAM_HEIGHT, -8.0),
        yaw: 0.0, // side A faces +z
    };
    let mut accumulator = 0.0_f32;
    let mut grabbed = true;
    set_cursor_grab(true);
    show_mouse(false);
    let mut last_mouse = mouse_position();

    loop {
        // ------------------------------------------------ input
        if is_key_pressed(KeyCode::Escape) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }
        let (mx, _my) = mouse_position();
        if grabbed {
            let dx = mx - last_mouse.0;
            cam.yaw -= dx * 0.003;
        }
        last_mouse = mouse_position();

        // squad orders
        if is_key_pressed(KeyCode::Key1) {
            sim.set_command(Side::A, SquadCommand::Advance);
        }
        if is_key_pressed(KeyCode::Key2) {
            sim.set_command(Side::A, SquadCommand::Hold);
        }
        if is_key_pressed(KeyCode::Key3) {
            sim.set_command(Side::A, SquadCommand::Brace);
        }
        if is_key_pressed(KeyCode::Key4) {
            sim.set_command(Side::A, SquadCommand::Charge);
        }
        if is_key_pressed(KeyCode::Key5) {
            sim.set_command(Side::A, SquadCommand::Rotate); // auto-reverts
        }
        if is_key_pressed(KeyCode::Key6) {
            sim.set_command(Side::A, SquadCommand::Withdraw);
        }
        if is_key_pressed(KeyCode::F) {
            sim.volley(Side::A); // the wall casts together
        }

        // movement: camera-relative WASD
        let mut mv = vec2(0.0, 0.0);
        if is_key_down(KeyCode::W) {
            mv.y += 1.0;
        }
        if is_key_down(KeyCode::S) {
            mv.y -= 1.0;
        }
        if is_key_down(KeyCode::A) {
            mv.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            mv.x += 1.0;
        }
        let speed = jk_core::constants::PLAYER_MOVE_SPEED_M_S;
        let (s, c) = (cam.yaw.sin(), cam.yaw.cos());
        let world = vec2(
            (mv.x * c + mv.y * s) * speed,
            (-mv.x * s + mv.y * c) * speed,
        );
        let push = if is_key_down(KeyCode::LeftShift) { 1.0 } else { 0.3 };
        sim.set_player_input(PlayerInput {
            move_x: world.x,
            move_z: world.y,
            push,
            strike: is_mouse_button_pressed(MouseButton::Left),
            throw: is_mouse_button_pressed(MouseButton::Right),
            aim_x: 0.0,
            aim_z: 1.0,
            aim_dist: 16.0,
        });

        // take over the nearest standing man when down
        if sim.agents[player].downed && is_key_pressed(KeyCode::Space) {
            let (px, pz) = sim.position(&sim.agents[player]);
            let next = sim
                .agents
                .iter()
                .enumerate()
                .filter(|(_, a)| a.side == Side::A && !a.downed)
                .min_by(|(_, a), (_, b)| {
                    let da = {
                        let (x, z) = sim.position(a);
                        (x - px).powi(2) + (z - pz).powi(2)
                    };
                    let db = {
                        let (x, z) = sim.position(b);
                        (x - px).powi(2) + (z - pz).powi(2)
                    };
                    da.partial_cmp(&db).unwrap()
                })
                .map(|(i, _)| i);
            if let Some(i) = next {
                sim.player = Some(i);
                player = i;
            }
        }

        // ------------------------------------------------ fixed-step sim
        accumulator += get_frame_time().min(0.25);
        let mut steps = 0;
        while accumulator >= DT && steps < 8 {
            sim.step();
            accumulator -= DT;
            steps += 1;
        }

        // ------------------------------------------------ render 3D
        let (px, pz) = sim.position(&sim.agents[player]);
        let target = vec3(px, 0.0, pz);
        cam.follow(target, get_frame_time());
        clear_background(Color::from_rgba(126, 138, 162, 255));
        set_camera(&cam.camera3d(target));

        draw_grid(60, 1.0, DARKGRAY, GRAY);

        // occlusion cull: never draw a body between the lens and the player
        let occl2 = {
            let d = cam.eye - vec3(px, 0.9, pz);
            d.length() - 0.35
        };
        for (i, a) in sim.agents.iter().enumerate() {
            let (x, z) = sim.position(a);
            let is_player = i == player;
            let to_body = (vec3(x, 0.9, z) - cam.eye).length();
            if !is_player && to_body < occl2 {
                continue;
            }
            if a.downed {
                draw_cube(
                    vec3(x, 0.15, z),
                    vec3(0.7, 0.25, 0.7),
                    None,
                    side_color(a.side, false, true, false),
                );
                continue;
            }
            // torso
            draw_cube(
                vec3(x, 0.82, z),
                vec3(0.44, 1.5, 0.30),
                None,
                side_color(a.side, is_player, false, a.routing),
            );
            // head
            draw_sphere(vec3(x, 1.62, z), 0.13, None, Color::from_rgba(205, 170, 140, 255));
            // shield
            let fwd = a.side.forward_sign();
            draw_cube(
                vec3(x, 0.85, z + fwd * 0.34),
                vec3(0.9, 1.0, 0.07),
                None,
                shield_color(a.side),
            );
            if is_player {
                draw_sphere(vec3(x, 2.05, z), 0.07, None, GOLD);
            }
        }

        // ------------------------------------------------ HUD
        set_default_camera();
        let m = sim.telemetry.steps.last();
        let cmd = format!("{:?}", sim.side_command[0]);
        let stamina = sim.agents[player].stamina.output_fraction();
        let comp = sim.agents[player].compression_n;
        draw_text(
            &format!("ORDER: {cmd}  [1 Advance 2 Hold 3 Brace 4 Charge 5 Rotate 6 Withdraw]"),
            16.0,
            28.0,
            26.0,
            WHITE,
        );
        draw_text(
            &format!(
                "you: stamina {:>3.0}%  chest load {:>4.0} N {}",
                stamina * 100.0,
                comp,
                if sim.agents[player].downed {
                    "— DOWN (SPACE: take next man)"
                } else if comp > 3000.0 {
                    "— CRUSHED, GET OUT"
                } else {
                    ""
                }
            ),
            16.0,
            56.0,
            24.0,
            if comp > 3000.0 { RED } else { WHITE },
        );
        if let Some(m) = m {
            draw_text(
                &format!(
                    "wall: {} vs {}   cohesion {:.2}   press {:>4.1} kN",
                    m.active[0],
                    m.active[1],
                    m.cohesion[0].mean_omega,
                    m.interface_force_n / 1000.0
                ),
                16.0,
                84.0,
                24.0,
                WHITE,
            );
            let fear_col = if m.routing[0] > 0 { ORANGE } else { WHITE };
            draw_text(
                &format!(
                    "morale: fear {:.2} ({} fleeing)  |  enemy fear {:.2} ({} fleeing)",
                    m.mean_fear[0], m.routing[0], m.mean_fear[1], m.routing[1]
                ),
                16.0,
                140.0,
                24.0,
                fear_col,
            );
        }
        let spear = if sim.agents[player].strike_cooldown_s <= 0.0 {
            "spear READY".to_string()
        } else {
            format!("spear {:.1}s", sim.agents[player].strike_cooldown_s.max(0.0))
        };
        draw_text(
            &format!(
                "{spear}   kills {}   armor {:?}",
                sim.agents[player].kills, sim.agents[player].armor
            ),
            16.0,
            112.0,
            24.0,
            WHITE,
        );
        draw_text(
            "WASD move   SHIFT shoulder-in   LMB thrust   mouse look   ESC cursor",
            16.0,
            screen_height() - 16.0,
            20.0,
            LIGHTGRAY,
        );

        next_frame().await;
    }
}
