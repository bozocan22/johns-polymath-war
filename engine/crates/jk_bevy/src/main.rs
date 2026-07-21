//! jk_bevy — the real 3D client: the same deterministic WallSim, rendered
//! with Bevy.
//!
//! Soldiers are articulated low-poly humans (head/torso/arms/legs/shield/
//! spear) whose motion is DRIVEN BY THE SIM: legs stride at the body's real
//! velocity, torsos lean into the press by measured chest load, the spear
//! arm thrusts when the man actually strikes, routing men run hunched, and
//! the fallen collapse where the physics dropped them. No canned loops —
//! the animation is a readout of the simulation.
//!
//! Tripo3D/Blender GLB characters (assets/characters/{commander,soldier_a,
//! soldier_b}.glb) replace the procedural men per file when present.
//!
//! Controls: WASD move · mouse look · SHIFT shoulder-in · LMB thrust ·
//! 1-6 orders · SPACE take over nearest man when down · ESC cursor

use bevy::input::mouse::MouseMotion;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use jk_core::timestep::DT;
use jk_wall::{PlayerInput, Side, SquadCommand, WallSim, WallSimConfig};

const WAIST_Y: f32 = 0.98;

#[derive(Resource)]
struct Game {
    sim: WallSim,
    player: usize,
    accum: f32,
}

#[derive(Resource, Default)]
struct CamCtl {
    yaw: f32,
    pitch: f32,
    grabbed: bool,
}

#[derive(Resource)]
struct CharacterScenes {
    commander: Option<Handle<Scene>>,
    soldier_a: Option<Handle<Scene>>,
    soldier_b: Option<Handle<Scene>>,
}

/// Root of one sim agent's visual body.
#[derive(Component)]
struct AgentVis {
    index: usize,
}

/// Procedural-rig state (absent on glTF scene bodies).
#[derive(Component)]
struct Rig {
    phase: f32,
    prev_cooldown: f32,
    strike_t: f32,
    fall_t: f32,
}

/// Pivot entities of the procedural body.
#[derive(Component)]
struct RigParts {
    leg_l: Entity,
    leg_r: Entity,
    torso: Entity,
    arm_l: Entity,
    arm_r: Entity,
}

#[derive(Component)]
struct MainCam;

#[derive(Component)]
struct ProjMarker;

#[derive(Resource, Default)]
struct ProjVis(std::collections::HashMap<u32, Entity>);

#[derive(Resource)]
struct ProjAssets {
    mesh: Handle<Mesh>,
    mat: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Hud;

/// ESC toggles between the battle and the pause menu.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

#[derive(Component)]
struct MenuRoot;

#[derive(Component, Clone, Copy)]
enum MenuButton {
    Resume,
    Restart,
    Quit,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "John Kingdom Game".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.62, 0.68, 0.78)))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (input_system, step_sim)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                sync_agents,
                animate_rigs,
                sync_projectiles,
                camera_system,
                hud_system,
            )
                .chain(),
        )
        .add_systems(Update, esc_toggle)
        .add_systems(OnEnter(GameState::Paused), open_menu)
        .add_systems(OnExit(GameState::Paused), close_menu)
        .add_systems(
            Update,
            menu_buttons.run_if(in_state(GameState::Paused)),
        )
        .run();
}

fn esc_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(match state.get() {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
        });
    }
}

fn open_menu(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    cam.grabbed = false;
    if let Ok(mut w) = windows.get_single_mut() {
        w.cursor_options.grab_mode = CursorGrabMode::None;
        w.cursor_options.visible = true;
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.75)),
            MenuRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("JOHN KINGDOM GAME"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
                Node {
                    margin: UiRect::bottom(Val::Px(18.0)),
                    ..default()
                },
            ));
            for (label, which) in [
                ("Resume        (ESC)", MenuButton::Resume),
                ("Restart Battle", MenuButton::Restart),
                ("Quit", MenuButton::Quit),
            ] {
                p.spawn((
                    Button,
                    which,
                    Node {
                        width: Val::Px(280.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

fn close_menu(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    menu: Query<Entity, With<MenuRoot>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    cam.grabbed = true;
    if let Ok(mut w) = windows.get_single_mut() {
        w.cursor_options.grab_mode = CursorGrabMode::Locked;
        w.cursor_options.visible = false;
    }
    for e in &menu {
        commands.entity(e).despawn_recursive();
    }
}

fn menu_buttons(
    mut q: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut game: ResMut<Game>,
    mut rigs: Query<&mut Rig>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut restarts: Local<u64>,
) {
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.24, 0.29, 0.40)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
            Interaction::Pressed => match which {
                MenuButton::Resume => next.set(GameState::Playing),
                MenuButton::Restart => {
                    // A fresh battle, fresh seed; the 80 visual bodies map
                    // 1:1 onto the new sim's 80 agents.
                    *restarts += 1;
                    let mut sim = WallSim::new(WallSimConfig {
                        seed: 0xC0FFEE + *restarts,
                        ..Default::default()
                    });
                    let player = sim.take_player(Side::A, 0, 4).expect("player slot");
                    *game = Game {
                        sim,
                        player,
                        accum: 0.0,
                    };
                    for mut rig in &mut rigs {
                        *rig = Rig {
                            phase: 0.0,
                            prev_cooldown: 0.0,
                            strike_t: 0.0,
                            fall_t: 0.0,
                        };
                    }
                    next.set(GameState::Playing);
                }
                MenuButton::Quit => {
                    exit.send(AppExit::Success);
                }
            },
        }
    }
}

fn character_scene(asset_server: &AssetServer, name: &str) -> Option<Handle<Scene>> {
    let path = format!("characters/{name}.glb");
    if std::path::Path::new("assets").join(&path).exists() {
        Some(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path)))
    } else {
        None
    }
}

struct BodyKit {
    skin: Handle<StandardMaterial>,
    tunic: Handle<StandardMaterial>,
    legs: Handle<StandardMaterial>,
    shield: Handle<StandardMaterial>,
    wood: Handle<StandardMaterial>,
    iron: Handle<StandardMaterial>,
    head: Handle<Mesh>,
    torso: Handle<Mesh>,
    arm: Handle<Mesh>,
    leg: Handle<Mesh>,
    shield_mesh: Handle<Mesh>,
    spear: Handle<Mesh>,
    spear_tip: Handle<Mesh>,
    helmet: Handle<Mesh>,
}

/// Build one articulated low-poly man under `root`. Pivots sit at the hips,
/// waist, and shoulders so the animation bends where humans bend.
fn spawn_procedural_man(commands: &mut Commands, root: Entity, kit: &BodyKit) -> RigParts {
    // legs hang from hip pivots
    let leg_l = commands
        .spawn((Transform::from_xyz(-0.10, WAIST_Y, 0.0), Visibility::default()))
        .set_parent(root)
        .id();
    commands
        .spawn((
            Mesh3d(kit.leg.clone()),
            MeshMaterial3d(kit.legs.clone()),
            Transform::from_xyz(0.0, -0.49, 0.0),
        ))
        .set_parent(leg_l);
    let leg_r = commands
        .spawn((Transform::from_xyz(0.10, WAIST_Y, 0.0), Visibility::default()))
        .set_parent(root)
        .id();
    commands
        .spawn((
            Mesh3d(kit.leg.clone()),
            MeshMaterial3d(kit.legs.clone()),
            Transform::from_xyz(0.0, -0.49, 0.0),
        ))
        .set_parent(leg_r);

    // torso pivot at the waist carries chest, head, and both arms
    let torso = commands
        .spawn((Transform::from_xyz(0.0, WAIST_Y, 0.0), Visibility::default()))
        .set_parent(root)
        .id();
    commands
        .spawn((
            Mesh3d(kit.torso.clone()),
            MeshMaterial3d(kit.tunic.clone()),
            Transform::from_xyz(0.0, 0.28, 0.0),
        ))
        .set_parent(torso);
    commands
        .spawn((
            Mesh3d(kit.head.clone()),
            MeshMaterial3d(kit.skin.clone()),
            Transform::from_xyz(0.0, 0.70, 0.0),
        ))
        .set_parent(torso);
    commands
        .spawn((
            Mesh3d(kit.helmet.clone()),
            MeshMaterial3d(kit.iron.clone()),
            Transform::from_xyz(0.0, 0.78, 0.0),
        ))
        .set_parent(torso);

    // left arm carries the shield
    let arm_l = commands
        .spawn((Transform::from_xyz(-0.25, 0.52, 0.0), Visibility::default()))
        .set_parent(torso)
        .id();
    commands
        .spawn((
            Mesh3d(kit.arm.clone()),
            MeshMaterial3d(kit.skin.clone()),
            Transform::from_xyz(0.0, -0.28, 0.0),
        ))
        .set_parent(arm_l);
    commands
        .spawn((
            Mesh3d(kit.shield_mesh.clone()),
            MeshMaterial3d(kit.shield.clone()),
            Transform::from_xyz(0.04, -0.30, 0.20),
        ))
        .set_parent(arm_l);

    // right arm carries the spear
    let arm_r = commands
        .spawn((Transform::from_xyz(0.25, 0.52, 0.0), Visibility::default()))
        .set_parent(torso)
        .id();
    commands
        .spawn((
            Mesh3d(kit.arm.clone()),
            MeshMaterial3d(kit.skin.clone()),
            Transform::from_xyz(0.0, -0.28, 0.0),
        ))
        .set_parent(arm_r);
    let spear = commands
        .spawn((
            Mesh3d(kit.spear.clone()),
            MeshMaterial3d(kit.wood.clone()),
            Transform::from_xyz(0.06, -0.30, 0.08),
        ))
        .set_parent(arm_r)
        .id();
    commands
        .spawn((
            Mesh3d(kit.spear_tip.clone()),
            MeshMaterial3d(kit.iron.clone()),
            Transform::from_xyz(0.0, 1.15, 0.0),
        ))
        .set_parent(spear);

    RigParts {
        leg_l,
        leg_r,
        torso,
        arm_l,
        arm_r,
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut sim = WallSim::new(WallSimConfig::default());
    let player = sim.take_player(Side::A, 0, 4).expect("player slot");

    let scenes = CharacterScenes {
        commander: character_scene(&asset_server, "commander"),
        soldier_a: character_scene(&asset_server, "soldier_a"),
        soldier_b: character_scene(&asset_server, "soldier_b"),
    };

    // ---- world ----------------------------------------------------------
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(240.0, 240.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.33, 0.38, 0.26),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.6, 0.0)),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.78, 0.82, 0.92),
        brightness: 400.0,
    });

    // ---- shared body parts ----------------------------------------------
    let mesh_head = meshes.add(Sphere::new(0.11));
    let mesh_helmet = meshes.add(Sphere::new(0.125));
    let mesh_torso = meshes.add(Cuboid::new(0.36, 0.56, 0.22));
    let mesh_arm = meshes.add(Cuboid::new(0.09, 0.58, 0.11));
    let mesh_leg = meshes.add(Cuboid::new(0.11, 0.94, 0.13));
    let mesh_shield = meshes.add(Cylinder::new(0.45, 0.05));
    let mesh_spear = meshes.add(Cylinder::new(0.02, 2.2));
    let mesh_tip = meshes.add(Cuboid::new(0.05, 0.16, 0.02));

    let skin = materials.add(StandardMaterial {
        base_color: Color::srgb(0.80, 0.65, 0.53),
        ..default()
    });
    let legs_a = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.27, 0.30),
        ..default()
    });
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.43, 0.32, 0.21),
        ..default()
    });
    let iron = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.57, 0.62),
        metallic: 0.8,
        perceptual_roughness: 0.4,
        ..default()
    });
    let tunic_a = materials.add(StandardMaterial {
        base_color: Color::srgb(0.33, 0.49, 0.84),
        ..default()
    });
    let tunic_b = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.29, 0.25),
        ..default()
    });
    let tunic_player = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.75, 0.27),
        ..default()
    });
    let shield_a = materials.add(StandardMaterial {
        base_color: Color::srgb(0.59, 0.69, 0.90),
        ..default()
    });
    let shield_b = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.59, 0.53),
        ..default()
    });

    // ---- men ------------------------------------------------------------
    for (i, a) in sim.agents.iter().enumerate() {
        let is_player = i == player;
        let scene = if is_player {
            scenes.commander.clone().or_else(|| scenes.soldier_a.clone())
        } else {
            match a.side {
                Side::A => scenes.soldier_a.clone(),
                Side::B => scenes.soldier_b.clone(),
            }
        };
        let fwd_yaw = if a.side == Side::A {
            0.0
        } else {
            std::f32::consts::PI
        };
        let root = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(fwd_yaw)),
                Visibility::default(),
                AgentVis { index: i },
            ))
            .id();
        match scene {
            Some(scene) => {
                commands
                    .spawn((SceneRoot(scene), Transform::IDENTITY))
                    .set_parent(root);
            }
            None => {
                let kit = BodyKit {
                    skin: skin.clone(),
                    tunic: if is_player {
                        tunic_player.clone()
                    } else if a.side == Side::A {
                        tunic_a.clone()
                    } else {
                        tunic_b.clone()
                    },
                    legs: legs_a.clone(),
                    shield: if a.side == Side::A {
                        shield_a.clone()
                    } else {
                        shield_b.clone()
                    },
                    wood: wood.clone(),
                    iron: iron.clone(),
                    head: mesh_head.clone(),
                    torso: mesh_torso.clone(),
                    arm: mesh_arm.clone(),
                    leg: mesh_leg.clone(),
                    shield_mesh: mesh_shield.clone(),
                    spear: mesh_spear.clone(),
                    spear_tip: mesh_tip.clone(),
                    helmet: mesh_helmet.clone(),
                };
                let parts = spawn_procedural_man(&mut commands, root, &kit);
                commands.entity(root).insert((
                    parts,
                    Rig {
                        phase: 0.0,
                        prev_cooldown: 0.0,
                        strike_t: 0.0,
                        fall_t: 0.0,
                    },
                ));
            }
        }
    }

    // ---- projectiles -----------------------------------------------------
    commands.insert_resource(ProjAssets {
        mesh: meshes.add(Cylinder::new(0.018, 2.0)),
        mat: wood.clone(),
    });
    commands.insert_resource(ProjVis::default());

    // ---- camera + HUD ---------------------------------------------------
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 65_f32.to_radians(),
            ..default()
        }),
        DistanceFog {
            color: Color::srgb(0.62, 0.68, 0.78),
            falloff: FogFalloff::Linear {
                start: 35.0,
                end: 110.0,
            },
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCam,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(14.0),
            top: Val::Px(10.0),
            ..default()
        },
        Hud,
    ));

    commands.insert_resource(scenes);
    commands.insert_resource(Game {
        sim,
        player,
        accum: 0.0,
    });
    commands.insert_resource(CamCtl {
        yaw: 0.0,
        pitch: 0.55,
        grabbed: true,
    });
}

fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut cam: ResMut<CamCtl>,
    mut game: ResMut<Game>,
) {
    if cam.grabbed {
        for ev in motion.read() {
            cam.yaw -= ev.delta.x * 0.0028;
            cam.pitch = (cam.pitch - ev.delta.y * 0.002).clamp(0.15, 1.2);
        }
    } else {
        motion.clear();
    }

    for (key, cmd) in [
        (KeyCode::Digit1, SquadCommand::Advance),
        (KeyCode::Digit2, SquadCommand::Hold),
        (KeyCode::Digit3, SquadCommand::Brace),
        (KeyCode::Digit4, SquadCommand::Charge),
        (KeyCode::Digit5, SquadCommand::Rotate),
        (KeyCode::Digit6, SquadCommand::Withdraw),
    ] {
        if keys.just_pressed(key) {
            game.sim.set_command(Side::A, cmd);
        }
    }
    if keys.just_pressed(KeyCode::KeyF) {
        game.sim.volley(Side::A); // the wall casts together
    }

    let mut mv = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        mv.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        mv.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        mv.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        mv.x += 1.0;
    }
    let mv = mv.normalize_or_zero() * jk_core::constants::PLAYER_MOVE_SPEED_M_S;
    let (s, c) = (cam.yaw.sin(), cam.yaw.cos());
    let world = Vec2::new(mv.x * c + mv.y * s, -mv.x * s + mv.y * c);
    let push = if keys.pressed(KeyCode::ShiftLeft) { 1.0 } else { 0.3 };
    // Aim follows the camera: direction from yaw, distance from pitch
    // (look up to throw far, down to drop it short).
    let aim_dist = (6.0 + (1.2 - cam.pitch) * 24.0).clamp(8.0, 28.0);
    game.sim.set_player_input(PlayerInput {
        move_x: world.x,
        move_z: world.y,
        push,
        strike: buttons.just_pressed(MouseButton::Left),
        throw: buttons.just_pressed(MouseButton::Right),
        aim_x: s,
        aim_z: c,
        aim_dist,
    });

    if game.sim.agents[game.player].downed && keys.just_pressed(KeyCode::Space) {
        let (px, pz) = game.sim.position(&game.sim.agents[game.player]);
        let next = game
            .sim
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.side == Side::A && !a.downed)
            .min_by(|(_, a), (_, b)| {
                let da = {
                    let (x, z) = game.sim.position(a);
                    (x - px).powi(2) + (z - pz).powi(2)
                };
                let db = {
                    let (x, z) = game.sim.position(b);
                    (x - px).powi(2) + (z - pz).powi(2)
                };
                da.partial_cmp(&db).unwrap()
            })
            .map(|(i, _)| i);
        if let Some(i) = next {
            game.sim.player = Some(i);
            game.player = i;
        }
    }
}

fn step_sim(time: Res<Time>, mut game: ResMut<Game>) {
    game.accum += time.delta_secs().min(0.25);
    let mut steps = 0;
    while game.accum >= DT && steps < 8 {
        game.sim.step();
        game.accum -= DT;
        steps += 1;
    }
}

/// Roots follow the physics bodies; facing follows side and state. The
/// PLAYER faces his movement when moving, the camera when standing — the
/// body answers the stick, which is most of what "good steering" feels like.
fn sync_agents(
    game: Res<Game>,
    cam: Res<CamCtl>,
    mut q: Query<(&AgentVis, &mut Transform)>,
) {
    for (vis, mut tf) in &mut q {
        let a = &game.sim.agents[vis.index];
        let (x, z) = game.sim.position(a);
        let is_player = game.sim.player == Some(vis.index);
        let base_yaw = if a.side == Side::A {
            0.0
        } else {
            std::f32::consts::PI
        };
        if a.downed {
            tf.translation = Vec3::new(x, 0.0, z);
            continue; // the fall itself is animated in animate_rigs
        }
        tf.translation = Vec3::new(x, 0.0, z);
        let yaw = if is_player {
            let (vx, vz) = game.sim.velocity(a);
            if vx * vx + vz * vz > 0.16 {
                vx.atan2(vz) // face where you're going
            } else {
                cam.yaw // face where you're looking
            }
        } else if a.routing {
            base_yaw + std::f32::consts::PI
        } else {
            base_yaw
        };
        // turn smoothly, don't snap
        let target = Quat::from_rotation_y(yaw);
        tf.rotation = tf.rotation.slerp(target, 0.25);
    }
}

/// The animation is a readout of the sim: stride from real velocity, lean
/// from real chest load, thrust from real strikes, panic from real fear.
fn animate_rigs(
    time: Res<Time>,
    game: Res<Game>,
    mut roots: Query<(&AgentVis, &RigParts, &mut Rig, &mut Transform)>,
    mut parts: Query<&mut Transform, Without<AgentVis>>,
) {
    let dt = time.delta_secs();
    for (vis, rig_parts, mut rig, mut root_tf) in &mut roots {
        let a = &game.sim.agents[vis.index];

        // ---- the fall --------------------------------------------------
        if a.downed {
            rig.fall_t = (rig.fall_t + dt * 2.2).min(1.0);
            let t = rig.fall_t * rig.fall_t * (3.0 - 2.0 * rig.fall_t); // smoothstep
            let side_roll = if vis.index % 2 == 0 { 1.0 } else { -1.0 };
            root_tf.rotation = root_tf.rotation.lerp(
                Quat::from_rotation_y(root_tf.rotation.to_euler(EulerRot::YXZ).0)
                    * Quat::from_rotation_z(side_roll * std::f32::consts::FRAC_PI_2),
                t,
            );
            root_tf.translation.y = -WAIST_Y * 0.72 * t + 0.0;
            continue;
        }
        rig.fall_t = 0.0;
        root_tf.translation.y = 0.0;

        let (vx, vz) = game.sim.velocity(a);
        let speed = (vx * vx + vz * vz).sqrt();

        // ---- stride ----------------------------------------------------
        // step frequency tracks real speed (stride ~0.75 m)
        if speed > 0.06 {
            rig.phase += speed * 8.5 * dt;
        } else {
            // settle legs to neutral
            rig.phase += (-(rig.phase.sin()) * 0.15).clamp(-0.2, 0.2);
        }
        let amp = (speed / 1.2).clamp(0.0, 1.0) * 0.55
            + if a.routing { 0.25 } else { 0.0 };
        let swing = rig.phase.sin() * amp;

        if let Ok(mut t) = parts.get_mut(rig_parts.leg_l) {
            t.rotation = Quat::from_rotation_x(swing);
        }
        if let Ok(mut t) = parts.get_mut(rig_parts.leg_r) {
            t.rotation = Quat::from_rotation_x(-swing);
        }

        // ---- torso lean: into the press, or hunched flight -------------
        let press_lean = (a.compression_n / 4000.0).clamp(0.0, 1.0) * 0.35;
        let push_lean = (a.applied_push_n / 1200.0).clamp(0.0, 1.0) * 0.18;
        let lean = if a.routing {
            0.38
        } else {
            (press_lean + push_lean).min(0.45)
        };
        if let Ok(mut t) = parts.get_mut(rig_parts.torso) {
            t.rotation = Quat::from_rotation_x(lean);
        }

        // ---- arms ------------------------------------------------------
        // strike detection: cooldown jumped upward = a strike happened
        if a.strike_cooldown_s > rig.prev_cooldown + 0.5 {
            rig.strike_t = 0.28;
        }
        rig.prev_cooldown = a.strike_cooldown_s;
        rig.strike_t = (rig.strike_t - dt).max(0.0);

        // left arm: shield up, braced against the load
        let shield_raise = -1.15 - press_lean * 0.4;
        if let Ok(mut t) = parts.get_mut(rig_parts.arm_l) {
            t.rotation = Quat::from_rotation_x(shield_raise + swing * 0.08);
        }
        // right arm: counter-swing while marching; snaps forward on strike
        let arm_r_rot = if rig.strike_t > 0.0 {
            let k = rig.strike_t / 0.28; // 1 → 0
            let thrust = (1.0 - (2.0 * k - 1.0).abs()) * 1.25; // out and back
            -0.35 - thrust
        } else if a.routing {
            -0.9 + rig.phase.cos() * amp * 0.8 // pumping arms
        } else {
            -0.35 + rig.phase.cos() * amp * 0.4
        };
        if let Ok(mut t) = parts.get_mut(rig_parts.arm_r) {
            t.rotation = Quat::from_rotation_x(arm_r_rot);
        }
    }
}

/// Spears in flight arc with their velocity; spent ones stand in the field.
fn sync_projectiles(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<ProjAssets>,
    mut vis: ResMut<ProjVis>,
    mut q: Query<&mut Transform, With<ProjMarker>>,
) {
    let mut live: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for pr in &game.sim.projectiles {
        live.insert(pr.id);
        let dir = Vec3::new(pr.vel[0], pr.vel[1], pr.vel[2]).normalize_or_zero();
        let rot = if dir.length_squared() > 0.01 {
            Quat::from_rotation_arc(Vec3::Y, dir)
        } else {
            Quat::IDENTITY
        };
        let pos = Vec3::new(pr.pos[0], pr.pos[1].max(0.0) + 0.0, pr.pos[2]);
        match vis.0.get(&pr.id) {
            Some(&e) => {
                if let Ok(mut tf) = q.get_mut(e) {
                    tf.translation = pos;
                    tf.rotation = rot;
                }
            }
            None => {
                let e = commands
                    .spawn((
                        Mesh3d(assets.mesh.clone()),
                        MeshMaterial3d(assets.mat.clone()),
                        Transform::from_translation(pos).with_rotation(rot),
                        ProjMarker,
                    ))
                    .id();
                vis.0.insert(pr.id, e);
            }
        }
    }
    vis.0.retain(|id, e| {
        if live.contains(id) {
            true
        } else {
            commands.entity(*e).despawn_recursive();
            false
        }
    });
}

fn camera_system(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    mut q: Query<(&mut Transform, &mut Projection), With<MainCam>>,
) {
    let Ok((mut tf, mut proj)) = q.get_single_mut() else {
        return;
    };
    let p = &game.sim.agents[game.player];
    let (px, pz) = game.sim.position(p);
    let dt = time.delta_secs();

    let target = Vec3::new(px, 0.0, pz);
    let dist = 3.4 + cam_ctl.pitch * 1.8;
    let height = 1.6 + cam_ctl.pitch * 2.6;
    let desired = Vec3::new(
        target.x - cam_ctl.yaw.sin() * dist,
        target.y + height,
        target.z - cam_ctl.yaw.cos() * dist,
    );
    let stiffness = (5.0 * dt).min(1.0);
    let delta = (desired - tf.translation) * stiffness;
    tf.translation += delta;

    let look = Vec3::new(target.x, 1.2, target.z);
    tf.look_at(look, Vec3::Y);

    if let Projection::Perspective(persp) = &mut *proj {
        let (vx, vz) = game.sim.velocity(p);
        let speed = (vx * vx + vz * vz).sqrt();
        let base = 65_f32.to_radians();
        let kick = (speed / 3.2).clamp(0.0, 1.0) * 13_f32.to_radians();
        let target_fov = base + kick;
        persp.fov += (target_fov - persp.fov) * (6.0 * dt).min(1.0);
    }
}

fn hud_system(game: Res<Game>, mut q: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = q.get_single_mut() else {
        return;
    };
    let p = &game.sim.agents[game.player];
    let m = game.sim.telemetry.steps.last();
    let mut s = format!(
        "ORDER: {:?}   [1 Advance 2 Hold 3 Brace 4 Charge 5 Rotate 6 Withdraw]\n",
        game.sim.side_command[0]
    );
    s += &format!(
        "you: stamina {:>3.0}%   chest {:>4.0} N   {:?}   kills {}   javelins {} [RMB throw, F volley]{}\n",
        p.stamina.output_fraction() * 100.0,
        p.compression_n,
        p.armor,
        p.kills,
        p.javelins,
        if p.downed {
            "   DOWN — SPACE: next man"
        } else {
            ""
        }
    );
    if let Some(m) = m {
        s += &format!(
            "wall: {} vs {}   cohesion {:.2}   press {:>4.1} kN\n",
            m.active[0],
            m.active[1],
            m.cohesion[0].mean_omega,
            m.interface_force_n / 1000.0
        );
        s += &format!(
            "morale: fear {:.2} ({} fleeing)  |  enemy {:.2} ({} fleeing)",
            m.mean_fear[0], m.routing[0], m.mean_fear[1], m.routing[1]
        );
    }
    **text = s;
}
