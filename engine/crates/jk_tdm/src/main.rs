//! jk_tdm — arena shooter on the deterministic 120 Hz core.
//!
//! v4: every weapon is a real multi-part model (barrel, stock, magazine,
//! scope, bow limbs + string, spear blade) — held in the hands, not floating.
//! Bows visibly draw, spears cock overhead javelin-style. The roster is
//! ROBOT COWBOYS: metal heads with glowing visors under proper hats, team
//! bandanas, belts with shining buckles, boots — per-man colors.
//!
//! v5 (Phantom-Forces feel):
//! - FIRST or THIRD person — V toggles; first person gets real HANDS and
//!   a full weapon viewmodel that bobs, kicks, and dips on reload.
//! - DODGE ROLL (Q, or tap crouch while sprinting): a duck-spin somersault,
//!   faster than a sprint, low to the ground, gun locked while tumbling.
//!   Hard landings breakfall into the same roll automatically — parkour.
//! - Legs got KNEES and ANKLES: thigh/shin/foot chains with a real gait,
//!   a full deep crouch, an air tuck, and the roll's tucked ball.
//! - THREE MAPS at the intro: the dust arena, a castle bailey (keep, drum
//!   towers, crenellated walls), and green castle gardens (hedges, ruins,
//!   trees). Recoil is HALF what it was, per the owner's request.
//! - Bow/spear aiming shows a red PREDICTED ARC with a landing marker —
//!   the exact flight the sim will fly.
//!
//! Controls: WASD move · SPACE jump · Q dodge roll · V first/third person ·
//! mouse look · LEFT CLICK aim (zoom) · RIGHT CLICK / T fire · CTRL/C
//! crouch · SHIFT sprint · R reload · TAB scoreboard · ESC menu

mod sim;

use bevy::audio::Volume;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use jk_core::timestep::DT;
use sim::*;
use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Resource)]
struct Game {
    sim: TdmSim,
    accum: f32,
    rebuild: bool,
    last_t: f32,
    /// Edge-triggered inputs latched until a sim step consumes them —
    /// above 120 fps some frames run zero steps, and a raw `just_pressed`
    /// would be lost on exactly those frames.
    pending_jump: bool,
    pending_reload: bool,
    pending_dodge: bool,
    pending_shield: bool,
    pending_slot: Option<u8>,
}

#[derive(Resource)]
struct CamCtl {
    yaw: f32,
    pitch: f32,
    grabbed: bool,
    ads: bool,
    recoil: f32,
    /// V toggles: first-person (hands + viewmodel) vs third-person.
    first_person: bool,
}

/// Everything the intro/loadout screen configures for the next match.
#[derive(Resource, Clone)]
struct Selected {
    map: MapKind,
    difficulty: Difficulty,
    per_team: usize,
    loadout: Loadout,
    /// cosmetic only: hat + tunic colors picked before the match
    hat: usize,
    tunic: usize,
}

impl Default for Selected {
    fn default() -> Self {
        Selected {
            map: MapKind::Arena,
            difficulty: Difficulty::Normal,
            per_team: 5,
            loadout: DEFAULT_LOADOUT,
            hat: 0,
            tunic: 0,
        }
    }
}

/// Player-facing options (the settings page).
#[derive(Resource)]
struct GameSettings {
    /// §8: swap the aim/fire mouse buttons back to convention.
    swap_mouse: bool,
    minimap: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            swap_mouse: false,
            minimap: true,
        }
    }
}

const HAT_CHOICES: [(&str, (f32, f32, f32)); 4] = [
    ("WHITE", (0.92, 0.90, 0.85)),
    ("BLACK", (0.12, 0.11, 0.11)),
    ("BROWN", (0.38, 0.24, 0.12)),
    ("RED", (0.55, 0.15, 0.10)),
];
const TUNIC_CHOICES: [(&str, (f32, f32, f32)); 4] = [
    ("GOLD", (0.95, 0.78, 0.25)),
    ("TEAL", (0.20, 0.75, 0.70)),
    ("VIOLET", (0.60, 0.35, 0.85)),
    ("STEEL", (0.55, 0.58, 0.62)),
];

// ---- world-space visuals -------------------------------------------------

#[derive(Component)]
struct FighterVis {
    index: usize,
}

#[derive(Component)]
struct FighterRig {
    phase: f32,
    /// per side: [thigh (hip pivot), shin (knee pivot), foot (ankle pivot)]
    leg_l: [Entity; 3],
    leg_r: [Entity; 3],
    torso: Entity,
    /// per arm: [upper (shoulder pivot), forearm (elbow pivot), hand (wrist)]
    arm_l: [Entity; 3],
    arm_r: [Entity; 3],
    /// held weapon models, indexed by `weapon_slot`
    weapons: [Entity; N_WEAPONS],
    /// the always-carried shield, shown raised on the left arm
    shield: Entity,
    bow_arrow: Entity,
    armor_rig: Entity,
}

/// First-person viewmodel: hands + weapon parented to the camera.
#[derive(Resource)]
struct VmRig {
    root: Entity,
    weapons: [Entity; N_WEAPONS],
    shield: Entity,
}

/// Extra weapon greebles that only show while aiming — the ADS detail pass.
#[derive(Component)]
struct AdsDetail;

/// The red predicted-arc laser for bow/spear aiming.
#[derive(Resource)]
struct ArcVis {
    dots: Vec<Entity>,
    ring: Entity,
}

/// Model-index for every carriable weapon (v6 roster).
const N_WEAPONS: usize = 10;

fn weapon_slot(kind: GunKind) -> Option<usize> {
    match kind {
        GunKind::Fists => None,
        GunKind::Glock => Some(0),
        GunKind::Deagle => Some(1),
        GunKind::Mp5 => Some(2),
        GunKind::Shotgun => Some(3),
        GunKind::Ak47 => Some(4),
        GunKind::M4 => Some(5),
        GunKind::Awm => Some(6),
        GunKind::M249 => Some(7),
        GunKind::Bow => Some(8),
        GunKind::Spear => Some(9),
    }
}

const ALL_WEAPONS: [GunKind; N_WEAPONS] = [
    GunKind::Glock,
    GunKind::Deagle,
    GunKind::Mp5,
    GunKind::Shotgun,
    GunKind::Ak47,
    GunKind::M4,
    GunKind::Awm,
    GunKind::M249,
    GunKind::Bow,
    GunKind::Spear,
];

fn ammo_kind(kind: GunKind) -> &'static str {
    match kind {
        GunKind::Fists => "",
        GunKind::Glock => "9MM ROUNDS",
        GunKind::Deagle => ".50 AE",
        GunKind::Mp5 => "9MM ROUNDS",
        GunKind::Shotgun => "12 GAUGE",
        GunKind::Ak47 => "7.62 ROUNDS",
        GunKind::M4 => "5.56 ROUNDS",
        GunKind::Awm => ".338 LAPUA",
        GunKind::M249 => "5.56 BELT",
        GunKind::Bow => "ARROWS",
        GunKind::Spear => "WAR SPEARS",
    }
}

#[derive(Component)]
struct CoverVis;

#[derive(Component)]
struct HillVis;

#[derive(Component)]
struct PickupVis {
    index: usize,
    item: Entity,
}

/// The glowing ring over a respawn checkpoint; tinted by its owner.
#[derive(Component)]
struct CheckpointVis {
    index: usize,
}

#[derive(Component)]
struct HealthBarVis {
    index: usize,
    fill: Entity,
    afill: Entity,
}

#[derive(Component)]
struct BarFill;

#[derive(Component)]
struct TracerMarker;

#[derive(Component)]
struct MissileMarker;

#[derive(Component)]
struct DecalMarker;

#[derive(Component)]
struct MainCam;

#[derive(Resource, Default)]
struct TracerPool(Vec<Entity>);

#[derive(Resource, Default)]
struct MissilePool(Vec<Entity>);

#[derive(Resource, Default)]
struct DecalPool(Vec<Entity>);

#[derive(Resource)]
struct FxAssets {
    tracer_mesh: Handle<Mesh>,
    tracer_blue: Handle<StandardMaterial>,
    tracer_red: Handle<StandardMaterial>,
    missile_mesh: Handle<Mesh>,
    arrow_mat: Handle<StandardMaterial>,
    spear_mat: Handle<StandardMaterial>,
    decal_mesh: Handle<Mesh>,
    decal_mat: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct BarAssets {
    green: Handle<StandardMaterial>,
    orange: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
}

/// Shared meshes + materials for building weapon / gear models.
#[derive(Resource, Clone)]
struct ModelKit {
    cube: Handle<Mesh>,
    cyl: Handle<Mesh>, // unit cylinder: radius 0.5, height 1, axis Y
    gunmetal: Handle<StandardMaterial>,
    steel: Handle<StandardMaterial>,
    wood: Handle<StandardMaterial>,
    string: Handle<StandardMaterial>,
    lens: Handle<StandardMaterial>,
    olive: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    white: Handle<StandardMaterial>,
    med_glow: Handle<StandardMaterial>,
    armor_dark: Handle<StandardMaterial>,
    core_glow: Handle<StandardMaterial>,
    /// leather glove — every finger of every hand wears it
    hand: Handle<StandardMaterial>,
}

// ---- audio ---------------------------------------------------------------

#[derive(Resource)]
struct Sfx {
    shot_glock: Handle<AudioSource>,
    shot_deagle: Handle<AudioSource>,
    shot_mp5: Handle<AudioSource>,
    shot_shotgun: Handle<AudioSource>,
    shot_ak: Handle<AudioSource>,
    shot_rifle: Handle<AudioSource>, // M4
    shot_mg: Handle<AudioSource>,   // M249
    shot_sniper: Handle<AudioSource>, // AWM
    bow: Handle<AudioSource>,
    spear: Handle<AudioSource>,
    click: Handle<AudioSource>, // dry fire on an empty magazine
    shield: Handle<AudioSource>, // the plate takes a round
    hit: Handle<AudioSource>,
    headshot: Handle<AudioSource>,
    hurt: Handle<AudioSource>,
    pickup: Handle<AudioSource>,
    reload: Handle<AudioSource>,
    jump: Handle<AudioSource>,
    roll: Handle<AudioSource>,
    kill: Handle<AudioSource>,
    win: Handle<AudioSource>,
}

fn shot_sound<'a>(sfx: &'a Sfx, kind: GunKind) -> &'a Handle<AudioSource> {
    match kind {
        GunKind::Glock => &sfx.shot_glock,
        GunKind::Deagle => &sfx.shot_deagle,
        GunKind::Mp5 => &sfx.shot_mp5,
        GunKind::Shotgun => &sfx.shot_shotgun,
        GunKind::Ak47 => &sfx.shot_ak,
        GunKind::M4 => &sfx.shot_rifle,
        GunKind::Awm => &sfx.shot_sniper,
        GunKind::M249 => &sfx.shot_mg,
        GunKind::Bow => &sfx.bow,
        GunKind::Spear => &sfx.spear,
        GunKind::Fists => &sfx.hit,
    }
}

#[derive(Resource)]
struct SfxState {
    last_t: f32,
    prev_gun: GunKind,
    prev_fire_cd: f32,
    prev_reserve: u32,
    prev_reload: f32,
    prev_vy: f32,
    prev_roll: f32,
    prev_armor: f32,
    prev_health: f32,
    prev_alive: bool,
    prev_over: bool,
    prev_shield: bool,
    click_cd: f32,
    max_missile: u32,
}

impl Default for SfxState {
    fn default() -> Self {
        SfxState {
            last_t: 0.0,
            prev_gun: GunKind::Fists,
            prev_fire_cd: 0.0,
            prev_reserve: 0,
            prev_reload: 0.0,
            prev_vy: 0.0,
            prev_roll: 0.0,
            prev_armor: 0.0,
            prev_health: MAX_HEALTH,
            prev_alive: true,
            prev_over: false,
            prev_shield: false,
            click_cd: 0.0,
            max_missile: 0,
        }
    }
}

// ---- minimap -------------------------------------------------------------

#[derive(Component)]
struct MinimapRoot;

#[derive(Component)]
struct MinimapDot(usize);

#[derive(Component)]
struct MinimapCp(usize);

#[derive(Component)]
struct MinimapHill;

#[derive(Component)]
struct MinimapPlayer;

const MINIMAP_PX: f32 = 170.0;

// ---- AWM scope overlay ---------------------------------------------------

#[derive(Component)]
struct ScopeRoot;

// ---- UI ------------------------------------------------------------------

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct ScoreTimerText;

#[derive(Component)]
struct FeedText;

#[derive(Component)]
struct HitFeedText;

#[derive(Component)]
struct BannerText;

#[derive(Component)]
struct CrosshairText;

#[derive(Component)]
struct PanelInfoText;

#[derive(Component)]
struct PanelAmmoText;

#[derive(Component)]
struct ScoreboardRoot;

#[derive(Component)]
struct ScoreboardText;

/// 0 top, 1 right, 2 bottom, 3 left — damage-direction flash strips.
#[derive(Component)]
struct DmgEdge(u8);

#[derive(Component)]
struct OwnHpFill;

#[derive(Component)]
struct OwnArmorFill;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Intro,
    Playing,
    Paused,
    Settings,
    Manual,
}

#[derive(Component)]
struct IntroRoot;

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct SettingsRoot;

#[derive(Component)]
struct ManualRoot;

#[derive(Component, Clone, Copy)]
enum ModeButton {
    Tdm,
    Koth,
}

#[derive(Component, Clone, Copy)]
struct MapButton(MapKind);

#[derive(Component, Clone, Copy)]
struct DiffButton(Difficulty);

#[derive(Component, Clone, Copy)]
struct SizeButton(usize);

/// Loadout pick: (slot index, weapon).
#[derive(Component, Clone, Copy)]
struct LoadoutButton(usize, GunKind);

/// Cosmetics: (0 = hat, 1 = tunic; choice index).
#[derive(Component, Clone, Copy)]
struct CosmeticButton(usize, usize);

#[derive(Component, Clone, Copy)]
enum MenuButton {
    Resume,
    Restart,
    Settings,
    Manual,
    BackToLoadout,
    Quit,
}

#[derive(Component, Clone, Copy)]
enum SettingsButton {
    SwapMouse,
    Minimap,
    Back,
}

/// Live text labels on the settings page.
#[derive(Component, Clone, Copy)]
struct SettingsLabel(SettingsButtonKind);

#[derive(Clone, Copy, PartialEq)]
enum SettingsButtonKind {
    SwapMouse,
    Minimap,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "John Kingdom Game — Arena".into(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // shared workspace assets dir (engine/assets), whether
                    // launched via cargo or the raw target/release binary
                    file_path: "../../assets".into(),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.58, 0.63, 0.72)))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, input_and_step.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            (
                rebuild_world,
                sync_fighters,
                sync_tracers,
                sync_missiles,
                sync_decals,
                sync_pickups,
                camera_system,
                fp_viewmodel,
                arc_preview,
                sync_health_bars,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                hud_system,
                own_bars,
                scoreboard_system,
                damage_indicator,
                sfx_system,
                scope_overlay,
                ads_detail,
                checkpoint_rings,
                minimap_system,
            ),
        )
        .add_systems(Update, esc_toggle)
        .add_systems(OnEnter(GameState::Playing), grab_cursor)
        .add_systems(OnEnter(GameState::Intro), open_intro)
        .add_systems(OnExit(GameState::Intro), close_intro)
        .add_systems(
            Update,
            (
                intro_buttons,
                intro_map_buttons,
                intro_loadout_buttons,
                intro_cosmetic_buttons,
                intro_diff_buttons,
                intro_size_buttons,
            )
                .run_if(in_state(GameState::Intro)),
        )
        .add_systems(OnEnter(GameState::Paused), open_menu)
        .add_systems(OnExit(GameState::Paused), close_menu)
        .add_systems(Update, menu_buttons.run_if(in_state(GameState::Paused)))
        .add_systems(OnEnter(GameState::Settings), open_settings)
        .add_systems(OnExit(GameState::Settings), close_settings)
        .add_systems(Update, settings_buttons.run_if(in_state(GameState::Settings)))
        .add_systems(OnEnter(GameState::Manual), open_manual)
        .add_systems(OnExit(GameState::Manual), close_manual)
        .run();
}

// ------------------------------------------------------------------ colors

fn fighter_color(team: Team, slot: usize, is_player: bool) -> Color {
    if is_player {
        return Color::srgb(0.95, 0.78, 0.25);
    }
    // per-man shade: same team, different personality
    let k = 0.80 + 0.10 * (slot % 3) as f32;
    match team {
        Team::Blue => Color::srgb(0.25 * k, 0.46 * k, 0.92 * k),
        Team::Red => Color::srgb(0.90 * k, 0.30 * k, 0.24 * k),
    }
}

/// Per-man visor / accent color — the "personality" light on the face.
fn accent_color(slot: usize, is_player: bool) -> Color {
    if is_player {
        return Color::srgb(0.98, 0.85, 0.30);
    }
    const P: [(f32, f32, f32); 5] = [
        (0.95, 0.30, 0.20),
        (0.95, 0.85, 0.20),
        (0.30, 0.90, 0.50),
        (0.90, 0.40, 0.90),
        (0.20, 0.85, 0.95),
    ];
    let (r, g, b) = P[slot % 5];
    Color::srgb(r, g, b)
}

/// Per-man hat color — no two cowboys dress alike; the player rides in white.
fn hat_color(slot: usize, is_player: bool) -> Color {
    if is_player {
        return Color::srgb(0.92, 0.90, 0.85);
    }
    const H: [(f32, f32, f32); 5] = [
        (0.38, 0.24, 0.12), // saddle brown
        (0.12, 0.11, 0.11), // black
        (0.72, 0.60, 0.40), // tan
        (0.30, 0.30, 0.33), // slate
        (0.45, 0.18, 0.12), // oxblood
    ];
    let (r, g, b) = H[slot % 5];
    Color::srgb(r, g, b)
}

// --------------------------------------------------------------- models ---

/// A HAND with real fingers: palm + four two-jointed fingers + a
/// two-jointed thumb, curled around whatever it grips. `curl` is the
/// grip strength: 1.0 = fist around a rifle grip, 0.55 = the open cradle
/// of a forend, 0.9 = a pistol squeeze. `mirror` flips the chirality so
/// left hands are true left hands (thumb on the correct side).
fn spawn_hand(commands: &mut Commands, kit: &ModelKit, curl: f32, mirror: bool) -> Entity {
    let m = if mirror { -1.0_f32 } else { 1.0 };
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // palm
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_scale(Vec3::new(0.075, 0.028, 0.085)),
        ))
        .set_parent(root);
    // four fingers across the palm's front edge, two joints each
    for (fi, fx) in [-0.027_f32, -0.009, 0.009, 0.027].into_iter().enumerate() {
        let len = if fi == 0 || fi == 3 { 0.030 } else { 0.036 };
        let base = commands
            .spawn((
                Transform::from_xyz(fx * m, 0.0, 0.042)
                    .with_rotation(Quat::from_rotation_x(-1.15 * curl)),
                Visibility::default(),
            ))
            .set_parent(root)
            .id();
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.hand.clone()),
                Transform::from_xyz(0.0, 0.0, len * 0.5).with_scale(Vec3::new(0.015, 0.016, len)),
            ))
            .set_parent(base);
        let tip = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, len)
                    .with_rotation(Quat::from_rotation_x(-1.3 * curl)),
                Visibility::default(),
            ))
            .set_parent(base)
            .id();
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.hand.clone()),
                Transform::from_xyz(0.0, 0.0, len * 0.4)
                    .with_scale(Vec3::new(0.014, 0.015, len * 0.8)),
            ))
            .set_parent(tip);
    }
    // thumb: two joints, wrapping from the side (flipped when mirrored)
    let tbase = commands
        .spawn((
            Transform::from_xyz(-0.040 * m, 0.0, 0.012).with_rotation(
                Quat::from_rotation_y(0.9 * curl * m) * Quat::from_rotation_x(-0.5 * curl),
            ),
            Visibility::default(),
        ))
        .set_parent(root)
        .id();
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_xyz(0.0, 0.0, 0.016).with_scale(Vec3::new(0.017, 0.018, 0.032)),
        ))
        .set_parent(tbase);
    let ttip = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.032)
                .with_rotation(Quat::from_rotation_y(0.9 * curl * m)),
            Visibility::default(),
        ))
        .set_parent(tbase)
        .id();
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform::from_xyz(0.0, 0.0, 0.013).with_scale(Vec3::new(0.015, 0.016, 0.026)),
        ))
        .set_parent(ttip);
    root
}

/// Spawn a detailed weapon model. Root sits at the grip; +Z is the muzzle.
/// (The bow stands along +Y with its string toward -Z.) Every gun comes
/// with fingered HANDS posed to its grip. `with_detail` adds the extra
/// `AdsDetail` greebles that only show while aiming — the §5 detail LOD
/// belongs to YOUR weapons (world + viewmodel); bots skip the parts
/// entirely, which is the LOD working as intended.
fn spawn_weapon_model(
    commands: &mut Commands,
    kit: &ModelKit,
    kind: GunKind,
    with_detail: bool,
) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // (is_cylinder, material, translation, rot_x, scale)
    let mut parts: Vec<(bool, Handle<StandardMaterial>, Vec3, f32, Vec3)> = Vec::new();
    let mut detail: Vec<(bool, Handle<StandardMaterial>, Vec3, f32, Vec3)> = Vec::new();
    // (position, yaw, curl, mirrored-left?) for each gripping hand
    let mut hands: Vec<(Vec3, f32, f32, bool)> = Vec::new();
    match kind {
        GunKind::Fists => {}
        GunKind::Glock => {
            // compact polymer pistol: squared slide, short barrel
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.05, 0.07), 0.0, Vec3::new(0.045, 0.06, 0.22)));
            parts.push((true, kit.steel.clone(), Vec3::new(0.0, 0.05, 0.19), FRAC_PI_2, Vec3::new(0.026, 0.06, 0.026)));
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, -0.05, -0.01), 0.18, Vec3::new(0.042, 0.13, 0.06)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.085, 0.15), 0.0, Vec3::new(0.008, 0.012, 0.01)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.085, -0.03), 0.0, Vec3::new(0.014, 0.012, 0.01)));
            hands.push((Vec3::new(0.0, -0.05, -0.015), 0.0, 0.9, false));
        }
        GunKind::Deagle => {
            // the hand cannon: long heavy slide, gold accents
            parts.push((false, kit.steel.clone(), Vec3::new(0.0, 0.055, 0.10), 0.0, Vec3::new(0.052, 0.075, 0.30)));
            parts.push((true, kit.gunmetal.clone(), Vec3::new(0.0, 0.055, 0.26), FRAC_PI_2, Vec3::new(0.034, 0.07, 0.034)));
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, -0.055, -0.01), 0.20, Vec3::new(0.046, 0.14, 0.065)));
            parts.push((false, kit.gold.clone(), Vec3::new(0.0, 0.02, 0.06), 0.0, Vec3::new(0.054, 0.012, 0.16)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.10, 0.22), 0.0, Vec3::new(0.008, 0.014, 0.01)));
            detail.push((false, kit.gold.clone(), Vec3::new(0.0, 0.10, -0.04), 0.0, Vec3::new(0.016, 0.012, 0.01)));
            hands.push((Vec3::new(0.0, -0.055, -0.015), 0.0, 0.9, false));
        }
        GunKind::Mp5 => {
            // compact SMG: stubby, slab receiver, angled magazine
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.02, 0.04), 0.0, Vec3::new(0.05, 0.09, 0.34)));
            parts.push((true, kit.steel.clone(), Vec3::new(0.0, 0.03, 0.28), FRAC_PI_2, Vec3::new(0.024, 0.18, 0.024)));
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, -0.10, 0.10), 0.45, Vec3::new(0.032, 0.17, 0.06)));
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, -0.07, -0.05), 0.2, Vec3::new(0.04, 0.11, 0.05)));
            parts.push((false, kit.steel.clone(), Vec3::new(0.0, 0.0, -0.18), 0.0, Vec3::new(0.03, 0.07, 0.12)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.085, 0.16), 0.0, Vec3::new(0.01, 0.02, 0.012)));
            detail.push((true, kit.gunmetal.clone(), Vec3::new(0.0, 0.03, 0.37), FRAC_PI_2, Vec3::new(0.03, 0.03, 0.03)));
            hands.push((Vec3::new(0.0, -0.07, -0.055), 0.0, 1.0, false));
            hands.push((Vec3::new(0.0, -0.045, 0.16), 0.15, 0.55, true));
        }
        GunKind::Shotgun => {
            // pump gun: long tube pair, wood pump + stock
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.02, 0.02), 0.0, Vec3::new(0.05, 0.085, 0.30)));
            parts.push((true, kit.steel.clone(), Vec3::new(0.0, 0.045, 0.38), FRAC_PI_2, Vec3::new(0.028, 0.48, 0.028)));
            parts.push((true, kit.gunmetal.clone(), Vec3::new(0.0, -0.005, 0.36), FRAC_PI_2, Vec3::new(0.024, 0.42, 0.024)));
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, -0.015, 0.30), 0.0, Vec3::new(0.052, 0.05, 0.16)));
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, -0.035, -0.20), 0.12, Vec3::new(0.045, 0.10, 0.26)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.09, 0.55), 0.0, Vec3::new(0.008, 0.016, 0.01)));
            detail.push((false, kit.gold.clone(), Vec3::new(0.045, -0.01, 0.05), 0.0, Vec3::new(0.012, 0.03, 0.05)));
            hands.push((Vec3::new(0.0, -0.045, -0.06), 0.0, 1.0, false));
            hands.push((Vec3::new(0.0, -0.05, 0.30), 0.0, 0.55, true));
        }
        GunKind::Ak47 => {
            // the classic: wood furniture, curved magazine, long gas tube
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.02, 0.06), 0.0, Vec3::new(0.05, 0.085, 0.38)));
            parts.push((true, kit.steel.clone(), Vec3::new(0.0, 0.045, 0.44), FRAC_PI_2, Vec3::new(0.026, 0.36, 0.026)));
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, 0.01, 0.32), 0.0, Vec3::new(0.05, 0.055, 0.18)));
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, -0.015, -0.22), 0.10, Vec3::new(0.045, 0.10, 0.26)));
            // the curved mag: two angled segments
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, -0.10, 0.09), 0.35, Vec3::new(0.036, 0.14, 0.075)));
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, -0.185, 0.14), 0.75, Vec3::new(0.034, 0.11, 0.07)));
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, -0.08, -0.05), 0.30, Vec3::new(0.04, 0.10, 0.05)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.085, 0.10), 0.0, Vec3::new(0.014, 0.02, 0.012)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.09, 0.58), 0.0, Vec3::new(0.008, 0.018, 0.01)));
            hands.push((Vec3::new(0.0, -0.08, -0.06), 0.0, 1.0, false));
            hands.push((Vec3::new(0.0, -0.035, 0.32), 0.1, 0.55, true));
        }
        GunKind::M4 => {
            // modern carbine: rails, carry sights, straight mag
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.02, 0.08), 0.0, Vec3::new(0.05, 0.09, 0.42)));
            parts.push((true, kit.steel.clone(), Vec3::new(0.0, 0.03, 0.45), FRAC_PI_2, Vec3::new(0.028, 0.34, 0.028)));
            parts.push((false, kit.olive.clone(), Vec3::new(0.0, -0.01, -0.20), 0.0, Vec3::new(0.045, 0.10, 0.26)));
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, -0.09, 0.06), 0.25, Vec3::new(0.038, 0.16, 0.08)));
            parts.push((false, kit.steel.clone(), Vec3::new(0.0, 0.09, 0.10), 0.0, Vec3::new(0.018, 0.05, 0.10)));
            parts.push((false, kit.olive.clone(), Vec3::new(0.0, -0.08, -0.06), 0.35, Vec3::new(0.04, 0.10, 0.05)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.10, 0.24), 0.0, Vec3::new(0.008, 0.018, 0.01)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.10, -0.02), 0.0, Vec3::new(0.014, 0.016, 0.01)));
            detail.push((false, kit.gunmetal.clone(), Vec3::new(0.045, 0.02, 0.22), 0.0, Vec3::new(0.012, 0.02, 0.14)));
            hands.push((Vec3::new(0.0, -0.08, -0.06), 0.0, 1.0, false));
            hands.push((Vec3::new(0.0, -0.045, 0.24), 0.1, 0.55, true));
        }
        GunKind::Awm => {
            // the AWM: long fluted barrel, big scope, olive stock, bipod
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.01, 0.05), 0.0, Vec3::new(0.045, 0.08, 0.46)));
            parts.push((true, kit.steel.clone(), Vec3::new(0.0, 0.03, 0.55), FRAC_PI_2, Vec3::new(0.024, 0.55, 0.024)));
            parts.push((true, kit.gunmetal.clone(), Vec3::new(0.0, 0.10, 0.08), FRAC_PI_2, Vec3::new(0.055, 0.20, 0.055)));
            parts.push((true, kit.lens.clone(), Vec3::new(0.0, 0.10, 0.185), FRAC_PI_2, Vec3::new(0.045, 0.012, 0.045)));
            parts.push((false, kit.olive.clone(), Vec3::new(0.0, -0.02, -0.22), 0.0, Vec3::new(0.045, 0.11, 0.30)));
            parts.push((false, kit.steel.clone(), Vec3::new(0.045, -0.08, 0.42), 0.0, Vec3::new(0.012, 0.14, 0.012)));
            parts.push((false, kit.steel.clone(), Vec3::new(-0.045, -0.08, 0.42), 0.0, Vec3::new(0.012, 0.14, 0.012)));
            detail.push((true, kit.steel.clone(), Vec3::new(0.0, 0.03, 0.70), FRAC_PI_2, Vec3::new(0.030, 0.05, 0.030)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.05, 0.02, -0.04), -0.5, Vec3::new(0.014, 0.06, 0.014)));
            detail.push((true, kit.gunmetal.clone(), Vec3::new(0.0, 0.10, -0.02), FRAC_PI_2, Vec3::new(0.058, 0.02, 0.058)));
            hands.push((Vec3::new(0.0, -0.075, -0.10), 0.0, 1.0, false));
            hands.push((Vec3::new(0.0, -0.05, 0.26), 0.1, 0.55, true));
        }
        GunKind::M249 => {
            // belt-fed support gun: box mag, top feed tray, thick barrel
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.02, 0.05), 0.0, Vec3::new(0.075, 0.12, 0.50)));
            parts.push((true, kit.steel.clone(), Vec3::new(0.0, 0.04, 0.50), FRAC_PI_2, Vec3::new(0.045, 0.40, 0.045)));
            parts.push((true, kit.gunmetal.clone(), Vec3::new(0.0, 0.04, 0.72), FRAC_PI_2, Vec3::new(0.065, 0.09, 0.065)));
            parts.push((false, kit.olive.clone(), Vec3::new(0.0, -0.13, 0.02), 0.0, Vec3::new(0.09, 0.16, 0.13)));
            parts.push((false, kit.steel.clone(), Vec3::new(0.0, 0.12, 0.08), 0.0, Vec3::new(0.02, 0.06, 0.16)));
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.0, -0.22), 0.0, Vec3::new(0.05, 0.09, 0.18)));
            detail.push((false, kit.gold.clone(), Vec3::new(0.05, -0.02, 0.10), 0.0, Vec3::new(0.014, 0.05, 0.10)));
            detail.push((false, kit.steel.clone(), Vec3::new(0.0, 0.10, 0.30), 0.0, Vec3::new(0.01, 0.02, 0.012)));
            hands.push((Vec3::new(0.0, -0.07, -0.08), 0.0, 1.0, false));
            hands.push((Vec3::new(0.0, -0.10, 0.22), 0.1, 0.55, true));
        }
        GunKind::Bow => {
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, 0.24, 0.03), -0.30, Vec3::new(0.028, 0.46, 0.045)));
            parts.push((false, kit.wood.clone(), Vec3::new(0.0, -0.24, 0.03), 0.30, Vec3::new(0.028, 0.46, 0.045)));
            parts.push((false, kit.gunmetal.clone(), Vec3::new(0.0, 0.0, 0.01), 0.0, Vec3::new(0.035, 0.15, 0.055)));
            parts.push((false, kit.string.clone(), Vec3::new(0.0, 0.0, -0.09), 0.0, Vec3::new(0.008, 0.82, 0.008)));
            detail.push((false, kit.gold.clone(), Vec3::new(0.0, 0.10, 0.02), 0.0, Vec3::new(0.038, 0.02, 0.058)));
            detail.push((false, kit.gold.clone(), Vec3::new(0.0, -0.10, 0.02), 0.0, Vec3::new(0.038, 0.02, 0.058)));
            hands.push((Vec3::new(0.0, 0.0, 0.03), 0.0, 0.8, true));
        }
        GunKind::Spear => {
            parts.push((true, kit.wood.clone(), Vec3::new(0.0, 0.0, 0.35), FRAC_PI_2, Vec3::new(0.032, 1.85, 0.032)));
            parts.push((false, kit.steel.clone(), Vec3::new(0.0, 0.0, 1.32), 0.0, Vec3::new(0.055, 0.018, 0.22)));
            parts.push((true, kit.gunmetal.clone(), Vec3::new(0.0, 0.0, 1.18), FRAC_PI_2, Vec3::new(0.042, 0.09, 0.042)));
            parts.push((true, kit.gunmetal.clone(), Vec3::new(0.0, 0.0, -0.56), FRAC_PI_2, Vec3::new(0.038, 0.07, 0.038)));
            detail.push((false, kit.gold.clone(), Vec3::new(0.0, 0.0, 1.10), 0.0, Vec3::new(0.045, 0.045, 0.02)));
            hands.push((Vec3::new(0.0, -0.02, 0.0), 0.0, 1.0, false));
        }
    }
    for (is_cyl, mat, tr, rx, sc) in parts {
        let mesh = if is_cyl { kit.cyl.clone() } else { kit.cube.clone() };
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform {
                    translation: tr,
                    rotation: Quat::from_rotation_x(rx),
                    scale: sc,
                },
            ))
            .set_parent(root);
    }
    // aiming-only greebles (§5: weapon detail steps up on ADS)
    if !with_detail {
        detail.clear();
    }
    for (is_cyl, mat, tr, rx, sc) in detail {
        let mesh = if is_cyl { kit.cyl.clone() } else { kit.cube.clone() };
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform {
                    translation: tr,
                    rotation: Quat::from_rotation_x(rx),
                    scale: sc,
                },
                Visibility::Hidden,
                AdsDetail,
            ))
            .set_parent(root);
    }
    // the gripping hands, fingers curled to this weapon
    for (pos, ry, curl, mirror) in hands {
        let hand = spawn_hand(commands, kit, curl, mirror);
        commands
            .entity(hand)
            .insert(Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(ry)))
            .set_parent(root);
    }
    root
}

/// The always-carried tower shield: rounded plate, boss, sight slit,
/// gold trim — held on the left arm when raised (E).
fn spawn_shield_model(commands: &mut Commands, kit: &ModelKit) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // three gently angled slats fake a curved plate
    for (x, ry) in [(-0.16_f32, 0.28_f32), (0.0, 0.0), (0.16, -0.28)] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.armor_dark.clone()),
                Transform::from_xyz(x, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(ry))
                    .with_scale(Vec3::new(0.20, 0.72, 0.045)),
            ))
            .set_parent(root);
    }
    // boss
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.steel.clone()),
            Transform::from_xyz(0.0, 0.05, 0.04)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                .with_scale(Vec3::new(0.18, 0.05, 0.18)),
        ))
        .set_parent(root);
    // sight slit (glows faintly — the robot looks THROUGH the plate)
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.lens.clone()),
            Transform::from_xyz(0.0, 0.26, 0.028).with_scale(Vec3::new(0.16, 0.02, 0.01)),
        ))
        .set_parent(root);
    // gold trim top + bottom
    for y in [0.37_f32, -0.37] {
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.gold.clone()),
                Transform::from_xyz(0.0, y, 0.0).with_scale(Vec3::new(0.46, 0.035, 0.05)),
            ))
            .set_parent(root);
    }
    // handle bar + a gripping fist behind the plate
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.steel.clone()),
            Transform::from_xyz(0.0, 0.0, -0.06)
                .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
                .with_scale(Vec3::new(0.03, 0.16, 0.03)),
        ))
        .set_parent(root);
    let fist = spawn_hand(commands, kit, 1.0, true);
    commands
        .entity(fist)
        .insert(Transform::from_xyz(0.0, -0.02, -0.08))
        .set_parent(root);
    root
}

/// Spawn the wearable robot-armor rig (chest plate, power pack, pauldrons).
fn spawn_armor_rig(commands: &mut Commands, kit: &ModelKit) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    for (mat, tr, sc) in [
        (kit.armor_dark.clone(), Vec3::new(0.0, 0.32, 0.07), Vec3::new(0.50, 0.44, 0.28)),
        (kit.armor_dark.clone(), Vec3::new(0.0, 0.35, -0.20), Vec3::new(0.30, 0.36, 0.14)),
        (kit.core_glow.clone(), Vec3::new(0.0, 0.35, -0.28), Vec3::new(0.14, 0.14, 0.05)),
        (kit.armor_dark.clone(), Vec3::new(0.33, 0.58, 0.0), Vec3::new(0.17, 0.14, 0.20)),
        (kit.armor_dark.clone(), Vec3::new(-0.33, 0.58, 0.0), Vec3::new(0.17, 0.14, 0.20)),
    ] {
        commands
            .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                translation: tr,
                rotation: Quat::IDENTITY,
                scale: sc,
            }))
            .set_parent(root);
    }
    root
}

/// Spawn the floating model shown over a pickup pad.
fn spawn_pickup_model(commands: &mut Commands, kit: &ModelKit, kind: PickupKind) -> Entity {
    match kind {
        PickupKind::Health => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 1.0, 0.0), Visibility::default()))
                .id();
            for (mat, tr, sc) in [
                (kit.white.clone(), Vec3::ZERO, Vec3::new(0.30, 0.24, 0.30)),
                (kit.med_glow.clone(), Vec3::new(0.0, 0.0, 0.16), Vec3::new(0.20, 0.06, 0.02)),
                (kit.med_glow.clone(), Vec3::new(0.0, 0.0, 0.16), Vec3::new(0.06, 0.20, 0.02)),
                (kit.med_glow.clone(), Vec3::new(0.0, 0.13, 0.0), Vec3::new(0.20, 0.02, 0.06)),
            ] {
                commands
                    .spawn((Mesh3d(kit.cube.clone()), MeshMaterial3d(mat), Transform {
                        translation: tr,
                        rotation: Quat::IDENTITY,
                        scale: sc,
                    }))
                    .set_parent(root);
            }
            root
        }
        PickupKind::Ammo => {
            let root = commands
                .spawn((Transform::from_xyz(0.0, 1.0, 0.0), Visibility::default()))
                .id();
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(kit.olive.clone()),
                    Transform::from_scale(Vec3::new(0.34, 0.22, 0.26)),
                ))
                .set_parent(root);
            for x in [-0.09_f32, 0.0, 0.09] {
                commands
                    .spawn((
                        Mesh3d(kit.cyl.clone()),
                        MeshMaterial3d(kit.gold.clone()),
                        Transform::from_xyz(x, 0.17, 0.0).with_scale(Vec3::new(0.05, 0.14, 0.05)),
                    ))
                    .set_parent(root);
            }
            root
        }
        PickupKind::RobotArmor => {
            let e = spawn_armor_rig(commands, kit);
            commands.entity(e).insert(
                Transform::from_xyz(0.0, 0.75, 0.0).with_scale(Vec3::splat(0.9)),
            );
            e
        }
    }
}

// ------------------------------------------- match-scoped world builders --

/// Build every fighter's rig. v6 art direction: ROUNDED and cute — big
/// head, short limbs, inflated capsule body (Baymax construction, Smurf
/// proportions) — over a REAL skeleton: hips/knees/ankles in the legs,
/// shoulder/elbow/wrist in the arms, fingered hands on every weapon.
fn spawn_fighter_rigs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    kit: &ModelKit,
    sim: &TdmSim,
    sel: &Selected,
) {
    let metal = |c: Color, m: f32, r: f32| StandardMaterial {
        base_color: c,
        metallic: m,
        perceptual_roughness: r,
        ..default()
    };
    // rounded primitives only — no hard edges on a body
    let mesh_head = meshes.add(Sphere::new(0.27));
    let mesh_torso = meshes.add(Capsule3d::new(0.30, 0.22));
    let mesh_thigh = meshes.add(Capsule3d::new(0.080, 0.16));
    let mesh_shin = meshes.add(Capsule3d::new(0.068, 0.14));
    let mesh_upper = meshes.add(Capsule3d::new(0.065, 0.12));
    let mesh_fore = meshes.add(Capsule3d::new(0.055, 0.11));
    let mesh_glove = meshes.add(Sphere::new(0.06));
    let mesh_boot = meshes.add(Capsule3d::new(0.075, 0.10));
    let robot_head = materials.add(metal(Color::srgb(0.72, 0.73, 0.76), 0.9, 0.35));
    let legs_mat = materials.add(metal(Color::srgb(0.28, 0.24, 0.20), 0.1, 0.9));
    let boot_mat = materials.add(metal(Color::srgb(0.20, 0.13, 0.08), 0.0, 0.9));
    let belt_mat = materials.add(metal(Color::srgb(0.25, 0.16, 0.09), 0.0, 0.85));

    for (i, f) in sim.fighters.iter().enumerate() {
        let is_player = i == sim.player;
        let slot = i % 5;
        // cosmetics: the player wears what the loadout screen picked
        let tunic_color = if is_player {
            let (_, (r, g, b)) = TUNIC_CHOICES[sel.tunic % TUNIC_CHOICES.len()];
            Color::srgb(r, g, b)
        } else {
            fighter_color(f.team, slot, false)
        };
        let hat_c = if is_player {
            let (_, (r, g, b)) = HAT_CHOICES[sel.hat % HAT_CHOICES.len()];
            Color::srgb(r, g, b)
        } else {
            hat_color(slot, false)
        };
        let tunic = materials.add(StandardMaterial {
            base_color: tunic_color,
            perceptual_roughness: 0.9,
            ..default()
        });
        let bandana = materials.add(StandardMaterial {
            base_color: match f.team {
                Team::Blue => Color::srgb(0.20, 0.40, 0.95),
                Team::Red => Color::srgb(0.95, 0.22, 0.18),
            },
            ..default()
        });
        let hat = materials.add(metal(hat_c, 0.05, 0.85));
        let visor = materials.add(StandardMaterial {
            base_color: accent_color(slot, is_player),
            emissive: {
                let c = accent_color(slot, is_player).to_srgba();
                LinearRgba::new(c.red * 2.2, c.green * 2.2, c.blue * 2.2, 1.0)
            },
            unlit: true,
            ..default()
        });
        let root = commands
            .spawn((
                Transform::from_xyz(f.pos[0], f.pos[1], f.pos[2]),
                Visibility::default(),
                FighterVis { index: i },
            ))
            .id();
        // STUBBY legs: thigh → knee → shin → ankle → boot
        let mut legs = [[Entity::PLACEHOLDER; 3]; 2];
        for (li, lx) in [(-0.13_f32), 0.13].into_iter().enumerate() {
            let thigh = commands
                .spawn((Transform::from_xyz(lx, 0.62, 0.0), Visibility::default()))
                .set_parent(root)
                .id();
            commands
                .spawn((
                    Mesh3d(mesh_thigh.clone()),
                    MeshMaterial3d(legs_mat.clone()),
                    Transform::from_xyz(0.0, -0.15, 0.0),
                ))
                .set_parent(thigh);
            let shin = commands
                .spawn((Transform::from_xyz(0.0, -0.30, 0.0), Visibility::default()))
                .set_parent(thigh)
                .id();
            commands
                .spawn((
                    Mesh3d(mesh_shin.clone()),
                    MeshMaterial3d(legs_mat.clone()),
                    Transform::from_xyz(0.0, -0.13, 0.0),
                ))
                .set_parent(shin);
            let foot = commands
                .spawn((Transform::from_xyz(0.0, -0.27, 0.0), Visibility::default()))
                .set_parent(shin)
                .id();
            commands
                .spawn((
                    Mesh3d(mesh_boot.clone()),
                    MeshMaterial3d(boot_mat.clone()),
                    Transform::from_xyz(0.0, -0.02, 0.05)
                        .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                ))
                .set_parent(foot);
            legs[li] = [thigh, shin, foot];
        }
        let [leg_l, leg_r] = legs;
        // round torso + cowboy dressing
        let torso = commands
            .spawn((Transform::from_xyz(0.0, 0.62, 0.0), Visibility::default()))
            .set_parent(root)
            .id();
        commands
            .spawn((
                Mesh3d(mesh_torso.clone()),
                MeshMaterial3d(tunic),
                Transform::from_xyz(0.0, 0.38, 0.0),
            ))
            .set_parent(torso);
        // belt + buckle
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(belt_mat.clone()),
                Transform::from_xyz(0.0, 0.12, 0.0).with_scale(Vec3::new(0.52, 0.07, 0.44)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.gold.clone()),
                Transform::from_xyz(0.0, 0.12, 0.225).with_scale(Vec3::new(0.10, 0.06, 0.02)),
            ))
            .set_parent(torso);
        // bandana at the neck
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(bandana),
                Transform::from_xyz(0.0, 0.60, 0.06)
                    .with_rotation(Quat::from_rotation_x(0.15))
                    .with_scale(Vec3::new(0.32, 0.12, 0.28)),
            ))
            .set_parent(torso);
        // the BIG round head + glowing visor
        commands
            .spawn((
                Mesh3d(mesh_head.clone()),
                MeshMaterial3d(robot_head.clone()),
                Transform::from_xyz(0.0, 0.82, 0.02),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(visor),
                Transform::from_xyz(0.0, 0.85, 0.30).with_scale(Vec3::new(0.28, 0.06, 0.05)),
            ))
            .set_parent(torso);
        // hat: brim, crown, band — plus the little antenna
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(hat.clone()),
                Transform::from_xyz(0.0, 1.02, 0.0).with_scale(Vec3::new(0.72, 0.028, 0.72)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(hat.clone()),
                Transform::from_xyz(0.0, 1.11, 0.0).with_scale(Vec3::new(0.36, 0.18, 0.36)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.gunmetal.clone()),
                Transform::from_xyz(0.0, 1.045, 0.0).with_scale(Vec3::new(0.365, 0.04, 0.365)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.steel.clone()),
                Transform::from_xyz(0.13, 1.22, 0.0).with_scale(Vec3::new(0.015, 0.13, 0.015)),
            ))
            .set_parent(torso);
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.core_glow.clone()),
                Transform::from_xyz(0.13, 1.30, 0.0).with_scale(Vec3::splat(0.035)),
            ))
            .set_parent(torso);
        // JOINTED arms: shoulder → elbow → wrist, short and round
        let mut arms = [[Entity::PLACEHOLDER; 3]; 2];
        for (ai, ax) in [(-0.36_f32), 0.36].into_iter().enumerate() {
            let upper = commands
                .spawn((Transform::from_xyz(ax, 0.62, 0.02), Visibility::default()))
                .set_parent(torso)
                .id();
            commands
                .spawn((
                    Mesh3d(mesh_upper.clone()),
                    MeshMaterial3d(legs_mat.clone()),
                    Transform::from_xyz(0.0, -0.10, 0.0),
                ))
                .set_parent(upper);
            let fore = commands
                .spawn((Transform::from_xyz(0.0, -0.21, 0.0), Visibility::default()))
                .set_parent(upper)
                .id();
            commands
                .spawn((
                    Mesh3d(mesh_fore.clone()),
                    MeshMaterial3d(legs_mat.clone()),
                    Transform::from_xyz(0.0, -0.09, 0.0),
                ))
                .set_parent(fore);
            let hand = commands
                .spawn((Transform::from_xyz(0.0, -0.19, 0.0), Visibility::default()))
                .set_parent(fore)
                .id();
            commands
                .spawn((
                    Mesh3d(mesh_glove.clone()),
                    MeshMaterial3d(kit.hand.clone()),
                    Transform::from_xyz(0.0, -0.02, 0.0),
                ))
                .set_parent(hand);
            arms[ai] = [upper, fore, hand];
        }
        let [arm_l, arm_r] = arms;
        // held weapons hang off the WRIST: one model per roster kind.
        // (local: the arm chain runs −Y, so ×90° X maps the muzzle out
        // along the arm — when the shoulder aims, the gun aims)
        let mut weapons = [Entity::PLACEHOLDER; N_WEAPONS];
        for (wi, wk) in ALL_WEAPONS.into_iter().enumerate() {
            let model = spawn_weapon_model(commands, kit, wk, is_player);
            let parent = if wk == GunKind::Bow { arm_l[2] } else { arm_r[2] };
            commands
                .entity(model)
                .insert((
                    Transform::from_xyz(0.0, -0.05, 0.02)
                        .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    Visibility::Hidden,
                ))
                .set_parent(parent);
            weapons[wi] = model;
        }
        // the always-carried shield, on the left forearm
        let shield = spawn_shield_model(commands, kit);
        commands
            .entity(shield)
            .insert((
                Transform::from_xyz(0.0, -0.12, 0.09)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                Visibility::Hidden,
            ))
            .set_parent(arm_l[1]);
        // the nocked arrow, shown while a bow is drawn
        let bow_arrow = commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.wood.clone()),
                Transform::from_xyz(0.0, -0.12, 0.04)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                    .with_scale(Vec3::new(0.015, 0.015, 0.62)),
                Visibility::Hidden,
            ))
            .set_parent(arm_l[2])
            .id();
        // wearable robot armor
        let armor_rig = spawn_armor_rig(commands, kit);
        commands
            .entity(armor_rig)
            .insert((Transform::IDENTITY, Visibility::Hidden))
            .set_parent(torso);
        commands.entity(root).insert(FighterRig {
            phase: 0.0,
            leg_l,
            leg_r,
            torso,
            arm_l,
            arm_r,
            weapons,
            shield,
            bow_arrow,
            armor_rig,
        });
    }
}

fn spawn_health_bars(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    bars: &BarAssets,
    n: usize,
) {
    let bar_back_mesh = meshes.add(Cuboid::new(0.74, 0.11, 0.01));
    let bar_fill_mesh = meshes.add(Cuboid::new(1.0, 0.08, 0.014));
    let bar_armor_mesh = meshes.add(Cuboid::new(1.0, 0.045, 0.016));
    let unlit = |c: Color| StandardMaterial {
        base_color: c,
        unlit: true,
        ..default()
    };
    let back_mat = materials.add(unlit(Color::srgba(0.05, 0.05, 0.07, 0.9)));
    let armor_mat = materials.add(unlit(Color::srgb(0.35, 0.8, 0.95)));
    for i in 0..n {
        let root = commands
            .spawn((Transform::IDENTITY, Visibility::default()))
            .id();
        commands
            .spawn((
                Mesh3d(bar_back_mesh.clone()),
                MeshMaterial3d(back_mat.clone()),
                Transform::IDENTITY,
            ))
            .set_parent(root);
        let fill = commands
            .spawn((
                Mesh3d(bar_fill_mesh.clone()),
                MeshMaterial3d(bars.green.clone()),
                Transform::from_xyz(0.0, 0.0, 0.01),
                BarFill,
            ))
            .set_parent(root)
            .id();
        let afill = commands
            .spawn((
                Mesh3d(bar_armor_mesh.clone()),
                MeshMaterial3d(armor_mat.clone()),
                Transform::from_xyz(0.0, 0.09, 0.01),
                Visibility::Hidden,
                BarFill,
            ))
            .set_parent(root)
            .id();
        commands
            .entity(root)
            .insert(HealthBarVis { index: i, fill, afill });
    }
}

fn spawn_pickup_pads(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    kit: &ModelKit,
    sim: &TdmSim,
) {
    let pad_mesh = meshes.add(Cylinder::new(0.95, 0.08));
    let pad_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.28, 0.35),
        emissive: LinearRgba::new(0.4, 0.9, 1.4, 1.0),
        ..default()
    });
    for (pi, p) in sim.pickups.iter().enumerate() {
        let root = commands
            .spawn((
                Transform::from_xyz(p.pos[0], p.pos[1], p.pos[2]),
                Visibility::default(),
            ))
            .id();
        commands
            .spawn((
                Mesh3d(pad_mesh.clone()),
                MeshMaterial3d(pad_mat.clone()),
                Transform::from_xyz(0.0, 0.04, 0.0),
            ))
            .set_parent(root);
        let item = spawn_pickup_model(commands, kit, p.kind);
        commands.entity(item).set_parent(root);
        commands.entity(root).insert(PickupVis { index: pi, item });
    }
    // checkpoint rings: a glowing circle per forward-spawn point
    let ring_mesh = meshes.add(Cylinder::new(CHECKPOINT_RADIUS, 0.04));
    for (ci, cp) in sim.checkpoints.iter().enumerate() {
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.85, 0.85, 0.9, 0.30),
            emissive: LinearRgba::new(0.7, 0.7, 0.9, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.spawn((
            Mesh3d(ring_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(cp.pos[0], cp.pos[1] + 0.05, cp.pos[2]),
            CheckpointVis { index: ci },
        ));
    }
}

// ------------------------------------------------------------------- setup

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let game = Game {
        sim: TdmSim::new(MatchConfig::default()),
        accum: 0.0,
        rebuild: true, // first rebuild_world pass spawns the cover
        last_t: 0.0,
        pending_jump: false,
        pending_reload: false,
        pending_dodge: false,
        pending_shield: false,
        pending_slot: None,
    };
    commands.insert_resource(Selected::default());
    commands.insert_resource(GameSettings::default());

    // ---- sounds ---------------------------------------------------------
    commands.insert_resource(Sfx {
        shot_glock: asset_server.load("audio/shot_glock.wav"),
        shot_deagle: asset_server.load("audio/shot_deagle.wav"),
        shot_mp5: asset_server.load("audio/shot_mp5.wav"),
        shot_shotgun: asset_server.load("audio/shot_shotgun.wav"),
        shot_ak: asset_server.load("audio/shot_ak.wav"),
        shot_rifle: asset_server.load("audio/shot_rifle.wav"),
        shot_mg: asset_server.load("audio/shot_mg.wav"),
        shot_sniper: asset_server.load("audio/shot_sniper.wav"),
        bow: asset_server.load("audio/bow.wav"),
        spear: asset_server.load("audio/spear.wav"),
        click: asset_server.load("audio/click.wav"),
        shield: asset_server.load("audio/shield.wav"),
        hit: asset_server.load("audio/hit.wav"),
        headshot: asset_server.load("audio/headshot.wav"),
        hurt: asset_server.load("audio/hurt.wav"),
        pickup: asset_server.load("audio/pickup.wav"),
        reload: asset_server.load("audio/reload.wav"),
        jump: asset_server.load("audio/jump.wav"),
        roll: asset_server.load("audio/roll.wav"),
        kill: asset_server.load("audio/kill.wav"),
        win: asset_server.load("audio/win.wav"),
    });
    commands.insert_resource(SfxState::default());

    // ---- lights (ground + walls are per-map: see rebuild_world) --------
    commands.spawn((
        DirectionalLight {
            illuminance: 13_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.5, 0.0)),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.85, 0.82, 0.78),
        brightness: 380.0,
    });

    // ---- shared model kit ----------------------------------------------
    let metal = |c: Color, m: f32, r: f32| StandardMaterial {
        base_color: c,
        metallic: m,
        perceptual_roughness: r,
        ..default()
    };
    let kit = ModelKit {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        cyl: meshes.add(Cylinder::new(0.5, 1.0)),
        gunmetal: materials.add(metal(Color::srgb(0.16, 0.17, 0.19), 0.8, 0.45)),
        steel: materials.add(metal(Color::srgb(0.62, 0.64, 0.68), 0.95, 0.30)),
        wood: materials.add(metal(Color::srgb(0.42, 0.28, 0.15), 0.0, 0.85)),
        string: materials.add(metal(Color::srgb(0.85, 0.82, 0.70), 0.0, 0.9)),
        lens: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 1.0),
            emissive: LinearRgba::new(0.4, 1.6, 2.4, 1.0),
            unlit: true,
            ..default()
        }),
        olive: materials.add(metal(Color::srgb(0.32, 0.35, 0.22), 0.2, 0.8)),
        gold: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.80, 0.30),
            metallic: 1.0,
            perceptual_roughness: 0.25,
            emissive: LinearRgba::new(0.6, 0.45, 0.1, 1.0),
            ..default()
        }),
        white: materials.add(metal(Color::srgb(0.92, 0.92, 0.90), 0.1, 0.6)),
        med_glow: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.95, 0.4),
            emissive: LinearRgba::new(0.2, 1.8, 0.4, 1.0),
            unlit: true,
            ..default()
        }),
        armor_dark: materials.add(metal(Color::srgb(0.14, 0.15, 0.18), 0.9, 0.35)),
        core_glow: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.25, 0.15),
            emissive: LinearRgba::new(2.2, 0.35, 0.15, 1.0),
            unlit: true,
            ..default()
        }),
        hand: materials.add(metal(Color::srgb(0.48, 0.36, 0.24), 0.05, 0.9)),
    };

    // Fighter rigs, health bars, and pickup pads are match-scoped now
    // (team size and map can change) — they're built by rebuild_world.
    let unlit = |c: Color| StandardMaterial {
        base_color: c,
        unlit: true,
        ..default()
    };
    let green = materials.add(unlit(Color::srgb(0.25, 0.9, 0.35)));
    let orange = materials.add(unlit(Color::srgb(0.95, 0.65, 0.15)));
    let red = materials.add(unlit(Color::srgb(0.92, 0.18, 0.15)));
    commands.insert_resource(BarAssets { green, orange, red });
    commands.insert_resource(kit.clone());

    // ---- shot / impact FX pools ----------------------------------------
    commands.insert_resource(FxAssets {
        tracer_mesh: meshes.add(Cuboid::new(0.02, 0.02, 1.0)),
        tracer_blue: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.9, 1.0),
            emissive: LinearRgba::new(2.0, 2.5, 4.0, 1.0),
            unlit: true,
            ..default()
        }),
        tracer_red: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.7),
            emissive: LinearRgba::new(4.0, 2.2, 1.2, 1.0),
            unlit: true,
            ..default()
        }),
        missile_mesh: meshes.add(Cuboid::new(0.05, 0.05, 1.0)),
        arrow_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.62, 0.44, 0.22),
            ..default()
        }),
        spear_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.31, 0.16),
            metallic: 0.4,
            ..default()
        }),
        decal_mesh: meshes.add(Cuboid::new(0.15, 0.15, 0.02)),
        decal_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.07, 0.06, 0.05),
            unlit: true,
            ..default()
        }),
    });
    commands.insert_resource(TracerPool::default());
    commands.insert_resource(MissilePool::default());
    commands.insert_resource(DecalPool::default());

    // ---- camera ---------------------------------------------------------
    let cam = commands
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 62_f32.to_radians(),
                ..default()
            }),
            DistanceFog {
                color: Color::srgb(0.58, 0.63, 0.72),
                falloff: FogFalloff::Linear {
                    start: 45.0,
                    end: 130.0,
                },
                ..default()
            },
            Transform::from_xyz(0.0, 3.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
            MainCam,
        ))
        .id();

    // ---- first-person viewmodel: hands + weapon on the camera ----------
    // (the camera looks down its -Z, so the models yaw 180°: muzzle out)
    let vm_root = commands
        .spawn((Transform::IDENTITY, Visibility::Hidden))
        .set_parent(cam)
        .id();
    let vm_arm_mat = materials.add(metal(Color::srgb(0.28, 0.24, 0.20), 0.1, 0.9));
    let mut vm_weapons = [Entity::PLACEHOLDER; N_WEAPONS];
    for (wi, wk) in ALL_WEAPONS.into_iter().enumerate() {
        let model = spawn_weapon_model(&mut commands, &kit, wk, true);
        let (tr, extra_rx) = match wk {
            GunKind::Bow => (Vec3::new(-0.16, -0.20, -0.42), 0.0),
            GunKind::Spear => (Vec3::new(0.24, -0.14, -0.30), -0.12),
            GunKind::Glock | GunKind::Deagle => (Vec3::new(0.16, -0.20, -0.40), 0.0),
            _ => (Vec3::new(0.20, -0.22, -0.42), 0.0),
        };
        commands
            .entity(model)
            .insert((
                Transform {
                    translation: tr,
                    rotation: Quat::from_rotation_y(PI) * Quat::from_rotation_x(extra_rx),
                    scale: Vec3::splat(0.9),
                },
                Visibility::Hidden,
            ))
            .set_parent(vm_root);
        // robot forearms running from the screen edges to the grips (the
        // weapon models already carry their own fingered hands)
        let fore_z = if wk == GunKind::Bow { -0.02 } else { 0.24 };
        for (fp, frx) in [
            (Vec3::new(0.09, -0.20, -0.16), -0.7_f32),
            (Vec3::new(-0.11, -0.22, fore_z - 0.14), -0.6),
        ] {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(vm_arm_mat.clone()),
                    Transform {
                        translation: fp,
                        rotation: Quat::from_rotation_x(frx),
                        scale: Vec3::new(0.075, 0.30, 0.075),
                    },
                ))
                .set_parent(model);
        }
        vm_weapons[wi] = model;
    }
    // the raised shield fills the view when it's up
    let vm_shield = spawn_shield_model(&mut commands, &kit);
    commands
        .entity(vm_shield)
        .insert((
            Transform::from_xyz(-0.10, -0.14, -0.55).with_scale(Vec3::splat(1.1)),
            Visibility::Hidden,
        ))
        .set_parent(vm_root);
    commands.insert_resource(VmRig {
        root: vm_root,
        weapons: vm_weapons,
        shield: vm_shield,
    });

    // ---- projectile arc preview: red laser dots + landing ring ---------
    let dot_mesh = meshes.add(Sphere::new(0.05));
    let laser_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.2, 0.15),
        emissive: LinearRgba::new(3.0, 0.4, 0.3, 1.0),
        unlit: true,
        ..default()
    });
    let mut dots = Vec::new();
    for _ in 0..28 {
        dots.push(
            commands
                .spawn((
                    Mesh3d(dot_mesh.clone()),
                    MeshMaterial3d(laser_mat.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    let ring = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.45, 0.03))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.2, 0.15, 0.55),
                emissive: LinearRgba::new(2.2, 0.3, 0.2, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::IDENTITY,
            Visibility::Hidden,
        ))
        .id();
    commands.insert_resource(ArcVis { dots, ring });

    // ---- HUD ------------------------------------------------------------
    commands.spawn((
        Text::new("+"),
        TextFont {
            font_size: 26.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(49.6),
            top: Val::Percent(48.6),
            ..default()
        },
        CrosshairText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(14.0),
            top: Val::Px(10.0),
            ..default()
        },
        HudText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(38.0),
            top: Val::Px(8.0),
            ..default()
        },
        ScoreTimerText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(14.0),
            top: Val::Px(10.0),
            ..default()
        },
        FeedText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(40.0),
            top: Val::Percent(58.0),
            ..default()
        },
        HitFeedText,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 34.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.85, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(38.0),
            ..default()
        },
        BannerText,
    ));
    // damage-direction strips
    for (idx, node) in [
        (
            0u8,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                ..default()
            },
        ),
        (
            1,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(24.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ),
        (
            2,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                ..default()
            },
        ),
        (
            3,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(24.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ),
    ] {
        commands.spawn((
            node,
            BackgroundColor(Color::srgba(0.85, 0.08, 0.08, 0.0)),
            DmgEdge(idx),
        ));
    }
    // your status panel — weapon, ammo, HP/armor — bottom right
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                bottom: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.72)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.85, 0.4)),
                PanelInfoText,
            ));
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 34.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PanelAmmoText,
            ));
            p.spawn((
                Node {
                    width: Val::Px(240.0),
                    height: Val::Px(14.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
            ))
            .with_children(|b| {
                b.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.9, 0.35)),
                    OwnHpFill,
                ));
            });
            p.spawn((
                Node {
                    width: Val::Px(240.0),
                    height: Val::Px(7.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.6)),
            ))
            .with_children(|b| {
                b.spawn((
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.35, 0.8, 0.95)),
                    OwnArmorFill,
                ));
            });
        });
    // scoreboard (TAB)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(24.0),
                top: Val::Percent(16.0),
                width: Val::Percent(52.0),
                padding: UiRect::all(Val::Px(18.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.88)),
            Visibility::Hidden,
            ScoreboardRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ScoreboardText,
            ));
        });

    // ---- AWM scope overlay: full-screen glass, not a zoom -------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            ScopeRoot,
        ))
        .with_children(|p| {
            // black curtains left/right of the lens
            for side in [true, false] {
                let mut n = Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    width: Val::Percent(32.0),
                    height: Val::Percent(100.0),
                    ..default()
                };
                if side {
                    n.left = Val::Px(0.0);
                } else {
                    n.right = Val::Px(0.0);
                }
                p.spawn((n, BackgroundColor(Color::BLACK)));
            }
            // thin curtains top/bottom
            for side in [true, false] {
                let mut n = Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(7.0),
                    ..default()
                };
                if side {
                    n.top = Val::Px(0.0);
                } else {
                    n.bottom = Val::Px(0.0);
                }
                p.spawn((n, BackgroundColor(Color::BLACK)));
            }
            // the lens ring
            p.spawn((
                Node {
                    width: Val::Vh(88.0),
                    height: Val::Vh(88.0),
                    border: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
            ));
            // crosshair lines
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(10.0),
                    width: Val::Px(1.5),
                    height: Val::Percent(80.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            ));
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(20.0),
                    top: Val::Percent(50.0),
                    width: Val::Percent(60.0),
                    height: Val::Px(1.5),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            ));
        });

    // ---- minimap (M / settings to toggle) ------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                bottom: Val::Px(16.0),
                width: Val::Px(MINIMAP_PX),
                height: Val::Px(MINIMAP_PX),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.05, 0.72)),
            MinimapRoot,
        ))
        .with_children(|p| {
            // teammates (max 8) — blue squares
            for i in 0..8 {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(7.0),
                        height: Val::Px(7.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.6, 1.0)),
                    Visibility::Hidden,
                    MinimapDot(i),
                ));
            }
            // objectives: checkpoints + the hill
            for i in 0..2 {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(11.0),
                        height: Val::Px(11.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BorderRadius::MAX,
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    MinimapCp(i),
                ));
            }
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(9.0),
                    height: Val::Px(9.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.8, 0.4, 1.0, 0.9)),
                BorderRadius::MAX,
                MinimapHill,
            ));
            // YOU: gold, with a facing needle
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(9.0),
                    height: Val::Px(9.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.8, 0.25)),
                BorderRadius::MAX,
                MinimapPlayer,
            ))
            .with_children(|b| {
                b.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(3.2),
                        top: Val::Px(-6.0),
                        width: Val::Px(2.5),
                        height: Val::Px(7.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.95, 0.8, 0.25)),
                ));
            });
        });

    commands.insert_resource(game);
    commands.insert_resource(CamCtl {
        yaw: 0.0,
        pitch: 0.08,
        grabbed: false,
        ads: false,
        recoil: 0.0,
        first_person: false,
    });
}

/// The whole battlefield look is map-owned: ground, border walls, cover
/// (styled by what each block is made of), sky/fog tint, and decorations.
/// Rebuilt on map change and on rematch (the crate layout is reseeded).
fn rebuild_world(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    kit: Res<ModelKit>,
    bars: Res<BarAssets>,
    sel: Res<Selected>,
    old: Query<
        Entity,
        Or<(
            With<CoverVis>,
            With<HillVis>,
            With<FighterVis>,
            With<HealthBarVis>,
            With<PickupVis>,
            With<CheckpointVis>,
        )>,
    >,
    mut clear: ResMut<ClearColor>,
    mut fog_q: Query<&mut DistanceFog, With<MainCam>>,
) {
    if !game.rebuild {
        return;
    }
    game.rebuild = false;
    for e in &old {
        commands.entity(e).despawn_recursive();
    }
    // everything match-scoped comes back for the CURRENT sim: fighters
    // (count follows team size), their bars, pads, checkpoint rings
    spawn_fighter_rigs(
        &mut commands,
        &mut meshes,
        &mut materials,
        &kit,
        &game.sim,
        &sel,
    );
    spawn_health_bars(
        &mut commands,
        &mut meshes,
        &mut materials,
        &bars,
        game.sim.fighters.len(),
    );
    spawn_pickup_pads(&mut commands, &mut meshes, &mut materials, &kit, &game.sim);
    let map = game.sim.map;
    // sky / ground / border palettes — the castle maps go green
    let (sky, ground_c, border_c) = match map {
        MapKind::Arena => (
            Color::srgb(0.58, 0.63, 0.72),
            Color::srgb(0.45, 0.40, 0.33),
            Color::srgb(0.55, 0.52, 0.47),
        ),
        MapKind::Bailey => (
            Color::srgb(0.52, 0.66, 0.60),
            Color::srgb(0.30, 0.42, 0.24),
            Color::srgb(0.52, 0.52, 0.50),
        ),
        MapKind::Gardens => (
            Color::srgb(0.55, 0.71, 0.55),
            Color::srgb(0.27, 0.48, 0.24),
            Color::srgb(0.58, 0.56, 0.50),
        ),
    };
    clear.0 = sky;
    if let Ok(mut fog) = fog_q.get_single_mut() {
        fog.color = sky;
    }
    // ground + border walls, sized to THIS map
    let half = game.sim.half;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(half * 2.2, half * 2.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: ground_c,
            perceptual_roughness: 1.0,
            ..default()
        })),
        CoverVis,
    ));
    let border_mat = materials.add(StandardMaterial {
        base_color: border_c,
        perceptual_roughness: 0.85,
        ..default()
    });
    for (pos, size) in [
        (
            Vec3::new(0.0, 2.0, half + 0.25),
            Vec3::new(half * 2.0 + 1.0, 4.0, 0.5),
        ),
        (
            Vec3::new(0.0, 2.0, -half - 0.25),
            Vec3::new(half * 2.0 + 1.0, 4.0, 0.5),
        ),
        (
            Vec3::new(half + 0.25, 2.0, 0.0),
            Vec3::new(0.5, 4.0, half * 2.0 + 1.0),
        ),
        (
            Vec3::new(-half - 0.25, 2.0, 0.0),
            Vec3::new(0.5, 4.0, half * 2.0 + 1.0),
        ),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(border_mat.clone()),
            Transform::from_translation(pos),
            CoverVis,
        ));
    }
    // castle dressing on the border: crenellated battlements + corner drums
    if map != MapKind::Arena {
        let merlon = meshes.add(Cuboid::new(0.9, 0.7, 0.6));
        let mut n = -half + 1.0;
        while n < half {
            for (p, s) in [
                (Vec3::new(n, 4.3, half + 0.25), false),
                (Vec3::new(n, 4.3, -half - 0.25), false),
                (Vec3::new(half + 0.25, 4.3, n), true),
                (Vec3::new(-half - 0.25, 4.3, n), true),
            ] {
                commands.spawn((
                    Mesh3d(merlon.clone()),
                    MeshMaterial3d(border_mat.clone()),
                    Transform::from_translation(p).with_rotation(if s {
                        Quat::from_rotation_y(FRAC_PI_2)
                    } else {
                        Quat::IDENTITY
                    }),
                    CoverVis,
                ));
            }
            n += 2.2;
        }
        let drum = meshes.add(Cylinder::new(2.4, 7.0));
        for sx in [-1.0_f32, 1.0] {
            for sz in [-1.0_f32, 1.0] {
                commands.spawn((
                    Mesh3d(drum.clone()),
                    MeshMaterial3d(border_mat.clone()),
                    Transform::from_xyz(sx * (half + 0.6), 3.5, sz * (half + 0.6)),
                    CoverVis,
                ));
            }
        }
    }
    // cover blocks, styled by what they're made of
    let crate_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.50, 0.40, 0.28),
        perceptual_roughness: 0.9,
        ..default()
    });
    let stone_mat = materials.add(StandardMaterial {
        base_color: match map {
            MapKind::Arena => Color::srgb(0.52, 0.48, 0.42),
            _ => Color::srgb(0.56, 0.56, 0.54),
        },
        perceptual_roughness: 0.95,
        ..default()
    });
    let hedge_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.38, 0.14),
        perceptual_roughness: 1.0,
        ..default()
    });
    let trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.24, 0.13),
        perceptual_roughness: 1.0,
        ..default()
    });
    let leaf_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.45, 0.16),
        perceptual_roughness: 1.0,
        ..default()
    });
    let leaf_mesh = meshes.add(Sphere::new(1.0));
    for (c, k) in game.sim.cover.iter().zip(game.sim.cover_kind.iter()) {
        let size = Vec3::new(
            c.max[0] - c.min[0],
            c.max[1] - c.min[1],
            c.max[2] - c.min[2],
        );
        let center = Vec3::new(
            (c.min[0] + c.max[0]) * 0.5,
            (c.min[1] + c.max[1]) * 0.5,
            (c.min[2] + c.max[2]) * 0.5,
        );
        let mat = match k {
            CoverKind::Crate => crate_mat.clone(),
            CoverKind::Stone => stone_mat.clone(),
            CoverKind::Hedge => hedge_mat.clone(),
            CoverKind::Tree => trunk_mat.clone(),
        };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(mat),
            Transform::from_translation(center),
            CoverVis,
        ));
        if *k == CoverKind::Tree {
            // crown the trunk with foliage
            for (dy, r) in [(0.4_f32, 1.5_f32), (1.4, 1.0)] {
                commands.spawn((
                    Mesh3d(leaf_mesh.clone()),
                    MeshMaterial3d(leaf_mat.clone()),
                    Transform::from_xyz(center.x, c.max[1] + dy, center.z)
                        .with_scale(Vec3::splat(r)),
                    CoverVis,
                ));
            }
        }
    }
    if game.sim.mode == Mode::Koth {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(HILL_RADIUS, 0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.75, 0.35, 0.95, 0.35),
                emissive: LinearRgba::new(1.2, 0.4, 1.8, 1.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(game.sim.hill[0], game.sim.hill[1] + 0.06, game.sim.hill[2]),
            HillVis,
        ));
    }
}

// ------------------------------------------------------------------- input

fn input_and_step(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    settings: Res<GameSettings>,
    mut motion: EventReader<MouseMotion>,
    mut cam: ResMut<CamCtl>,
    mut game: ResMut<Game>,
    cam_q: Query<&Transform, With<MainCam>>,
) {
    if cam.grabbed {
        for ev in motion.read() {
            cam.yaw -= ev.delta.x * 0.0026;
            cam.pitch = (cam.pitch + ev.delta.y * 0.0022).clamp(-0.7, 0.8);
        }
    } else {
        motion.clear();
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
    let mv = mv.normalize_or_zero();
    let (s, c) = (cam.yaw.sin(), cam.yaw.cos());
    // note the strafe sign: per playtest, A = screen-left, D = screen-right
    let world = Vec2::new(-mv.x * c + mv.y * s, mv.x * s + mv.y * c);

    // Aim: from the MUZZLE (crouch-lowered, lean-shifted — the same
    // origin the sim fires from) toward what the crosshair sees. Using a
    // standing eye here made crouched/leaned shots land off-crosshair.
    let aim = if let Ok(cam_tf) = cam_q.get_single() {
        let fwd = cam_tf.forward().as_vec3();
        let far = cam_tf.translation + fwd * 90.0;
        let eye = Vec3::from_array(game.sim.muzzle_origin(game.sim.player));
        (far - eye).normalize_or_zero()
    } else {
        Vec3::Z
    };

    // v6 mapping: LEFT = aim/zoom, RIGHT or T = fire (swap in settings),
    // SPACE jump, Q roll (or tap crouch at a sprint), E shield,
    // O or V first/third person, Z/X lean, 1-3 weapon slots, M minimap.
    // Edge inputs latch until a sim step runs.
    let (aim_btn, fire_btn) = if settings.swap_mouse {
        (MouseButton::Right, MouseButton::Left)
    } else {
        (MouseButton::Left, MouseButton::Right)
    };
    let ads = buttons.pressed(aim_btn);
    cam.ads = ads;
    if keys.just_pressed(KeyCode::KeyV) || keys.just_pressed(KeyCode::KeyO) {
        cam.first_person = !cam.first_person;
    }
    if keys.just_pressed(KeyCode::Space) {
        game.pending_jump = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.pending_reload = true;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        game.pending_shield = true;
    }
    for (key, s) in [
        (KeyCode::Digit1, 0u8),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
    ] {
        if keys.just_pressed(key) {
            game.pending_slot = Some(s);
        }
    }
    // lean: hold Z (left) / X (right) — Phantom-Forces peeking
    let lean = (keys.pressed(KeyCode::KeyX) as i32 - keys.pressed(KeyCode::KeyZ) as i32) as f32;
    let sprinting = keys.pressed(KeyCode::ShiftLeft) && !ads;
    if keys.just_pressed(KeyCode::KeyQ)
        || (sprinting
            && (keys.just_pressed(KeyCode::ControlLeft) || keys.just_pressed(KeyCode::KeyC)))
    {
        game.pending_dodge = true;
    }
    let mut cmd = PlayerCmd {
        move_x: world.x,
        move_z: world.y,
        sprint: sprinting,
        yaw: cam.yaw,
        aim: [aim.x, aim.y, aim.z],
        shoot: buttons.pressed(fire_btn) || keys.pressed(KeyCode::KeyT),
        reload: game.pending_reload,
        ads,
        crouch: keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::KeyC),
        jump: game.pending_jump,
        dodge: game.pending_dodge,
        slot: game.pending_slot,
        shield: game.pending_shield,
        lean,
    };

    let prev_fire_cd = game.sim.fighters[game.sim.player].fire_cd;
    game.accum += time.delta_secs().min(0.25);
    let mut steps = 0;
    while game.accum >= DT && steps < 8 {
        game.sim.step(cmd);
        // the shield TOGGLE must fire exactly once per press — repeating
        // it across a multi-step frame flips it right back (jump/reload/
        // dodge/slot are idempotent across steps; the toggle is not)
        cmd.shield = false;
        game.accum -= DT;
        steps += 1;
    }
    if steps > 0 {
        game.pending_jump = false;
        game.pending_reload = false;
        game.pending_dodge = false;
        game.pending_shield = false;
        game.pending_slot = None;
    }

    // recoil: the camera kicks up when your gun goes off — sustained fire
    // walks the muzzle (the gun is deliberately not stable). A shot is
    // detected by fire_cd jumping up (ammo deltas miss the shot fired on
    // the same tick a reload completes). Leaning braces the shoulder:
    // the CAMERA kick honors the same ×0.8 the sim's bloom gets.
    let p = &game.sim.fighters[game.sim.player];
    if p.alive() && p.fire_cd > prev_fire_cd {
        let spec = gun(p.gun);
        let brace = if p.lean.abs() > 0.1 {
            LEAN_RECOIL_MULT
        } else {
            1.0
        };
        cam.pitch =
            (cam.pitch - (spec.kick * 6.0 + p.bloom * 1.5) * brace).clamp(-0.7, 0.8);
        cam.recoil = (cam.recoil + 0.6).min(1.0);
    }

    // rematch detection: the sim reseeds itself → rebuild cover visuals
    if game.sim.t < game.last_t {
        game.rebuild = true;
    }
    game.last_t = game.sim.t;
}

// ----------------------------------------------------------------- visuals

fn sync_fighters(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    mut roots: Query<(&FighterVis, &mut FighterRig, &mut Transform, &mut Visibility)>,
    mut parts: Query<(&mut Transform, &mut Visibility), Without<FighterVis>>,
) {
    let dt = time.delta_secs();
    for (vis, mut rig, mut tf, mut root_vis) in &mut roots {
        // on the deploy frame the OLD rigs still exist while the sim may
        // already be smaller (8v8 → 5v5): index safely, never panic
        let Some(f) = game.sim.fighters.get(vis.index) else {
            *root_vis = Visibility::Hidden;
            continue;
        };
        let is_player = vis.index == game.sim.player;
        // first person — or looking through the AWM's glass, which parks
        // the camera at the eye in EITHER mode: your own body leaves the
        // frame (hands take over)
        let self_view = cam_ctl.first_person
            || (cam_ctl.ads && gun(f.gun).scoped && !f.shield_up);
        *root_vis = if is_player && self_view && f.alive() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if !f.alive() {
            // dead: lie flat until respawn — the chibi body is ~0.3 wide,
            // so only a small sink keeps the corpse visibly ON the ground
            tf.translation = Vec3::new(f.pos[0], f.pos[1] - 0.10, f.pos[2]);
            tf.rotation = Quat::from_rotation_y(f.yaw)
                * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            continue;
        }
        let rolling = f.roll_t > 0.0;
        if rolling {
            // the somersault: a full forward tumble about the ball's center,
            // facing the roll direction
            let prog = (1.0 - f.roll_t / ROLL_S).clamp(0.0, 1.0);
            let roll_yaw = f.roll_dir[0].atan2(f.roll_dir[1]);
            // pivot at the BALL's center (0.6 up) with the body curled
            // tight around it — the head must orbit, not plow the floor
            tf.translation = Vec3::new(f.pos[0], f.pos[1] + 0.6, f.pos[2]);
            tf.rotation = Quat::from_rotation_y(roll_yaw)
                * Quat::from_rotation_x(prog * std::f32::consts::TAU);
        } else {
            tf.translation = Vec3::new(f.pos[0], f.pos[1], f.pos[2]);
            // lean tilts the whole body sideways off the hips (positive
            // lean = peek screen-right = roll clockwise)
            let lean_roll = Quat::from_rotation_z(f.lean * 0.24);
            tf.rotation = tf
                .rotation
                .slerp(Quat::from_rotation_y(f.yaw) * lean_roll, 0.35);
        }
        // spawn-protection shimmer: bob slightly
        if f.protect_t > 0.0 {
            tf.translation.y += (game.sim.t * 14.0).sin() * 0.02;
        }
        let speed = (f.vel[0] * f.vel[0] + f.vel[1] * f.vel[1]).sqrt();
        if speed > 0.1 && !rolling {
            rig.phase += speed * 2.4 * dt * 4.0;
        }
        let amp = (speed / SPRINT_SPEED).clamp(0.0, 1.0) * 0.6;
        let swing = rig.phase.sin() * amp;
        let airborne = !f.grounded;
        // just-fired jerk, driven by the sim's own cooldown
        let spec = gun(f.gun);
        let jerk = if f.armed() && spec.fire_period > 0.0 {
            ((f.fire_cd / spec.fire_period).clamp(0.0, 1.0)).powi(3)
        } else {
            0.0
        };
        // hips + torso by state (chibi frame): ball, FULL squat, tuck,
        // stand. Standing hip 0.68 puts the boot soles ON the ground
        // (thigh 0.30 + shin 0.27 + ankle/boot ≈ 0.10).
        let (hip_y, torso_pitch_base) = if rolling {
            (0.25, 1.0) // curled tight around the tumble pivot
        } else if f.crouch {
            // deep pitch keeps the VISIBLE head inside the crouch hitbox
            (0.34, 0.55)
        } else {
            (
                0.68 + if speed > 0.5 {
                    (rig.phase * 2.0).sin().abs() * 0.02 * amp
                } else {
                    0.0
                },
                if airborne { 0.10 } else { 0.06 },
            )
        };
        // legs: thigh (hip), shin (knee), foot (ankle) per side.
        // Sign convention: +X rotation swings the limb BACKWARD, so knees
        // fold positive and a raised thigh is negative.
        for (leg, off) in [(rig.leg_l, 0.0_f32), (rig.leg_r, PI)] {
            let (thigh, shin, foot);
            if rolling {
                // tucked ball
                thigh = -1.55;
                shin = 2.30;
                foot = -0.55;
            } else if f.crouch && !airborne {
                // FULL crouch: a real deep squat
                thigh = -1.15;
                shin = 2.05;
                foot = -(2.05 - 1.15) * 0.6;
            } else if airborne {
                // jump tuck
                thigh = -0.85;
                shin = 1.50;
                foot = -0.40;
            } else {
                // the gait: hips swing, the knee bends hardest as the leg
                // recovers forward, the ankle keeps the sole near-level
                // with a little toe-off flick
                let sw = (rig.phase + off).sin() * amp;
                let lift = (rig.phase + off + 0.9).sin().max(0.0) * amp;
                thigh = sw * 0.9;
                shin = 0.10 + lift * 1.25;
                foot = -(thigh + shin) * 0.55 + (rig.phase + off - 1.2).sin() * 0.20 * amp;
            }
            if let Ok((mut t, _)) = parts.get_mut(leg[0]) {
                t.translation.y = hip_y;
                t.rotation = Quat::from_rotation_x(thigh);
            }
            if let Ok((mut t, _)) = parts.get_mut(leg[1]) {
                t.rotation = Quat::from_rotation_x(shin);
            }
            if let Ok((mut t, _)) = parts.get_mut(leg[2]) {
                t.rotation = Quat::from_rotation_x(foot);
            }
        }
        if let Ok((mut t, _)) = parts.get_mut(rig.torso) {
            // crouching also drops the torso a notch below the hips so
            // the big head ducks under the (shootable) crouch height
            t.translation.y = hip_y - if f.crouch && !rolling { 0.12 } else { 0.0 };
            t.rotation = Quat::from_rotation_x(torso_pitch_base + amp * 0.12 - jerk * 0.06);
        }
        // ---- arms + the held weapon --------------------------------------
        // Jointed chains now: shoulder (free) → elbow (hinge, no
        // inversion) → wrist. The pose solver keeps the WEAPON's pitch on
        // the aim line while the elbow carries a visible bend: shoulder
        // gets (aim + bend), elbow gets (−bend), so the chain sum tracks
        // the crosshair — the arm-IK contract of §1.
        let aim_pitch = if is_player { cam_ctl.pitch * 0.9 } else { 0.05 };
        let slot = weapon_slot(f.gun);
        for (wi, we) in rig.weapons.iter().enumerate() {
            if let Ok((_, mut v)) = parts.get_mut(*we) {
                // the bow shares the left hand with the shield — a raised
                // shield stows it
                let show = slot == Some(wi)
                    && !(f.shield_up && ALL_WEAPONS[wi] == GunKind::Bow);
                *v = if show {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
        // shoulder aim base: arm forward = −π/2 in this rig's local frame.
        // The arms live UNDER the torso, so whatever the torso pitches
        // (crouch lean, run lean, fire jerk) must be compensated here or
        // the weapon drifts off the crosshair by exactly that much.
        let torso_pitch = torso_pitch_base + amp * 0.12 - jerk * 0.06;
        let fwd = -FRAC_PI_2 + aim_pitch - torso_pitch;
        let mut arrow_vis = Visibility::Hidden;
        // (shoulder quat, elbow flex ≥ 0 shown as −flex, hand pitch)
        let mut left = (
            Quat::from_rotation_x(swing * 0.5),
            0.15,
            0.0_f32,
        );
        let mut right = (
            Quat::from_rotation_x(-swing * 0.5),
            0.15,
            0.0_f32,
        );
        if rolling {
            // arms wrap the ball
            left = (Quat::from_rotation_x(-0.9), 1.2, 0.0);
            right = (Quat::from_rotation_x(-0.9), 1.2, 0.0);
        } else if f.shield_up {
            // shield forward on the left arm; gun hand drops to the hip
            left = (Quat::from_rotation_x(-1.35), 0.35, 0.0);
            right = (Quat::from_rotation_x(-0.5), 0.3, 0.0);
        } else {
            match f.gun {
                GunKind::Fists => {}
                GunKind::Bow => {
                    // left arm punches the bow out; right arm draws, the
                    // elbow folding as the string comes back
                    let draw = if is_player {
                        if cam_ctl.ads {
                            1.0
                        } else {
                            0.25
                        }
                    } else {
                        0.6
                    };
                    left = (Quat::from_rotation_x(fwd + 0.18), 0.18, 0.0);
                    right = (
                        Quat::from_rotation_y(-0.35) * Quat::from_rotation_x(fwd + 0.45),
                        0.45 + draw * 0.75,
                        0.0,
                    );
                    if f.ammo > 0 && f.reload_t <= 0.0 {
                        arrow_vis = Visibility::Inherited;
                    }
                }
                GunKind::Spear => {
                    // cocked overhead javelin-style while aiming; whips
                    // forward on release; carry blend when lowered
                    let cocked = if is_player { cam_ctl.ads } else { true };
                    let a = if cocked {
                        -2.0 + jerk * 1.7 + aim_pitch * 0.4
                    } else {
                        0.35 + jerk * (-1.0)
                    };
                    let elbow = 0.3;
                    // wrist counter-rotates so the spear rides ~level on
                    // the aim line whatever the arm is doing. The muzzle
                    // points ALONG the arm, i.e. chain + 90° — that 90°
                    // must come out of the wrist too.
                    let hand = aim_pitch - (fwd + a - elbow) - FRAC_PI_2;
                    right = (Quat::from_rotation_x(fwd + a), elbow, hand);
                    left = (Quat::from_rotation_x(0.1 + swing * 0.3), 0.2, 0.0);
                }
                _ => {
                    // two-handed gun: right hand at the trigger with a
                    // solid elbow bend, left crossed to the foregrip;
                    // both kick with the shot
                    right = (
                        Quat::from_rotation_x(fwd + 0.45 + jerk * 0.30),
                        0.45 + jerk * 0.2,
                        0.0,
                    );
                    left = (
                        Quat::from_rotation_y(0.5) * Quat::from_rotation_x(fwd + 0.6 + jerk * 0.12),
                        0.6,
                        0.0,
                    );
                }
            }
        }
        for (arm, (sh, elbow, hand)) in [(rig.arm_l, left), (rig.arm_r, right)] {
            if let Ok((mut t, _)) = parts.get_mut(arm[0]) {
                t.rotation = sh;
            }
            if let Ok((mut t, _)) = parts.get_mut(arm[1]) {
                // hinge constraint: the elbow only folds forward
                t.rotation = Quat::from_rotation_x(-elbow.max(0.0));
            }
            if let Ok((mut t, _)) = parts.get_mut(arm[2]) {
                t.rotation = Quat::from_rotation_x(hand);
            }
        }
        // the shield plate itself
        if let Ok((_, mut v)) = parts.get_mut(rig.shield) {
            *v = if f.shield_up {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if let Ok((_, mut v)) = parts.get_mut(rig.bow_arrow) {
            *v = arrow_vis;
        }
        // robot armor shell
        if let Ok((_, mut v)) = parts.get_mut(rig.armor_rig) {
            *v = if f.armor > 0.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn sync_tracers(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    mut pool: ResMut<TracerPool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<TracerMarker>>,
) {
    while pool.0.len() < game.sim.tracers.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.tracer_mesh.clone()),
                MeshMaterial3d(assets.tracer_blue.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                TracerMarker,
            ))
            .id();
        pool.0.push(e);
    }
    for (idx, e) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis)) = q.get_mut(*e) else {
            continue;
        };
        match game.sim.tracers.get(idx) {
            Some(tr) => {
                let a = Vec3::from_array(tr.from);
                let b = Vec3::from_array(tr.to);
                let mid = (a + b) * 0.5;
                let len = (b - a).length().max(0.05);
                let dir = (b - a) / len;
                *tf = Transform::from_translation(mid)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(1.0, 1.0, len));
                *vis = Visibility::Visible;
                let mat = match tr.team {
                    Team::Blue => assets.tracer_blue.clone(),
                    Team::Red => assets.tracer_red.clone(),
                };
                commands.entity(*e).insert(MeshMaterial3d(mat));
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn sync_missiles(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    mut pool: ResMut<MissilePool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<MissileMarker>>,
) {
    while pool.0.len() < game.sim.missiles.len() {
        let e = commands
            .spawn((
                Mesh3d(assets.missile_mesh.clone()),
                MeshMaterial3d(assets.arrow_mat.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                MissileMarker,
            ))
            .id();
        pool.0.push(e);
    }
    for (idx, e) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis)) = q.get_mut(*e) else {
            continue;
        };
        match game.sim.missiles.get(idx) {
            Some(m) => {
                let dir = Vec3::from_array(m.vel).normalize_or(Vec3::Z);
                let (len, thick) = if m.is_spear { (1.9, 1.6) } else { (0.8, 1.0) };
                *tf = Transform::from_translation(Vec3::from_array(m.pos))
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(thick, thick, len));
                *vis = Visibility::Visible;
                let mat = if m.is_spear {
                    assets.spear_mat.clone()
                } else {
                    assets.arrow_mat.clone()
                };
                commands.entity(*e).insert(MeshMaterial3d(mat));
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn sync_decals(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<FxAssets>,
    mut pool: ResMut<DecalPool>,
    mut q: Query<(&mut Transform, &mut Visibility), With<DecalMarker>>,
) {
    const MAX_DECALS: usize = 400;
    let n = game.sim.impacts.len().min(MAX_DECALS);
    while pool.0.len() < n {
        let e = commands
            .spawn((
                Mesh3d(assets.decal_mesh.clone()),
                MeshMaterial3d(assets.decal_mat.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                DecalMarker,
            ))
            .id();
        pool.0.push(e);
    }
    let start = game.sim.impacts.len() - n;
    for (idx, e) in pool.0.iter().enumerate() {
        let Ok((mut tf, mut vis)) = q.get_mut(*e) else {
            continue;
        };
        match game.sim.impacts.get(start + idx) {
            Some((im, _)) => {
                let nrm = Vec3::from_array(im.normal).normalize_or(Vec3::Y);
                *tf = Transform::from_translation(Vec3::from_array(im.at) + nrm * 0.02)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, nrm));
                *vis = Visibility::Visible;
            }
            None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn sync_pickups(
    game: Res<Game>,
    roots: Query<&PickupVis>,
    mut items: Query<(&mut Transform, &mut Visibility)>,
) {
    for pv in &roots {
        let Some(p) = game.sim.pickups.get(pv.index) else {
            continue;
        };
        let Ok((mut tf, mut vis)) = items.get_mut(pv.item) else {
            continue;
        };
        if p.respawn_t > 0.0 {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Inherited;
            let t = game.sim.t + pv.index as f32 * 0.7;
            let base_y = if matches!(p.kind, PickupKind::RobotArmor) {
                0.75
            } else {
                1.0
            };
            tf.translation.y = base_y + (t * 2.0).sin() * 0.12;
            tf.rotation = Quat::from_rotation_y(t * 1.5);
        }
    }
}

fn sync_health_bars(
    game: Res<Game>,
    cam: Res<CamCtl>,
    bars: Res<BarAssets>,
    mut roots: Query<(&HealthBarVis, &mut Transform, &mut Visibility), Without<BarFill>>,
    mut fills: Query<
        (&mut Transform, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>),
        (With<BarFill>, Without<HealthBarVis>),
    >,
) {
    for (hb, mut tf, mut vis) in &mut roots {
        // same deploy-frame safety as the rigs: stale bars must not panic
        let Some(f) = game.sim.fighters.get(hb.index) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let self_view =
            cam.first_person || (cam.ads && gun(f.gun).scoped && !f.shield_up);
        if !f.alive() || (hb.index == game.sim.player && self_view) {
            // dead men carry no bar; in first person (or scoped glass)
            // neither do YOU — the HUD panel already shows your numbers
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;
        // above the hat brim
        tf.translation = Vec3::new(f.pos[0], f.pos[1] + f.height() + 0.72, f.pos[2]);
        // billboard: quad faces the camera
        tf.rotation = Quat::from_rotation_y(cam.yaw + PI);
        let frac = (f.health / MAX_HEALTH).clamp(0.0, 1.0);
        if let Ok((mut ft, _, mut mat)) = fills.get_mut(hb.fill) {
            ft.scale = Vec3::new(0.7 * frac.max(0.01), 1.0, 1.0);
            ft.translation.x = -0.35 * (1.0 - frac);
            let handle = if frac > 0.55 {
                bars.green.clone()
            } else if frac > 0.28 {
                bars.orange.clone()
            } else {
                bars.red.clone()
            };
            *mat = MeshMaterial3d(handle);
        }
        if let Ok((mut at, mut av, _)) = fills.get_mut(hb.afill) {
            let afrac = (f.armor / ROBOT_ARMOR_HP).clamp(0.0, 1.0);
            *av = if afrac > 0.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            at.scale = Vec3::new(0.7 * afrac.max(0.01), 1.0, 1.0);
            at.translation.x = -0.35 * (1.0 - afrac);
        }
    }
}

fn camera_system(
    time: Res<Time>,
    game: Res<Game>,
    mut cam_ctl: ResMut<CamCtl>,
    mut q: Query<(&mut Transform, &mut Projection), With<MainCam>>,
) {
    cam_ctl.recoil = (cam_ctl.recoil - time.delta_secs() * 5.0).max(0.0);
    let Ok((mut tf, mut proj)) = q.get_single_mut() else {
        return;
    };
    let p = &game.sim.fighters[game.sim.player];
    let (sy, cy) = (cam_ctl.yaw.sin(), cam_ctl.yaw.cos());
    let (sp, cp) = (cam_ctl.pitch.sin(), cam_ctl.pitch.cos());
    let fwd = Vec3::new(sy * cp, -sp, cy * cp);
    let right = Vec3::new(cy, 0.0, -sy);
    // an ADS'd AWM ALWAYS looks through the glass: even in third person
    // the scoped view snaps to the eye (a scope over a shoulder-cam
    // makes no sense)
    let scoped_in = cam_ctl.ads && p.alive() && gun(p.gun).scoped && !p.shield_up;
    // `right` above is the lean-LEFT direction under this game's yaw
    // convention; the true screen-right is its negation
    let screen_right = -right;
    if (cam_ctl.first_person || scoped_in) && p.alive() {
        // first person: the camera IS the eye — crouching, rolling, and
        // LEANING physically move it
        let eye_h = (p.height() - 0.16).max(0.55);
        let eye = Vec3::new(p.pos[0], p.pos[1] + eye_h, p.pos[2])
            + screen_right * (p.lean * LEAN_SHIFT);
        let k = (34.0 * time.delta_secs()).min(1.0);
        tf.translation = tf.translation.lerp(eye, k);
        let look = tf.translation + fwd;
        tf.look_at(look, Vec3::Y);
        // the head tilts with the lean
        tf.rotate_local_z(p.lean * 0.10);
    } else {
        // third person, over-the-right-shoulder; ADS pulls in tight
        let crouch_drop = if p.roll_t > 0.0 {
            0.75
        } else if p.crouch {
            0.62
        } else {
            0.0
        };
        let anchor = Vec3::new(p.pos[0], p.pos[1] + 1.6 - crouch_drop, p.pos[2])
            + screen_right * (p.lean * LEAN_SHIFT * 0.8);
        let dist = if cam_ctl.ads { 1.7 } else { 2.6 };
        let lift = if cam_ctl.ads { 0.22 } else { 0.35 };
        let desired = anchor - fwd * dist + right * 0.5 + Vec3::Y * lift;
        let k = (20.0 * time.delta_secs()).min(1.0);
        tf.translation = tf.translation.lerp(desired, k);
        let look = anchor + fwd * 12.0;
        tf.look_at(look, Vec3::Y);
    }
    if let Projection::Perspective(persp) = &mut *proj {
        // each gun has its own zoom: the sniper's is a real scope —
        // but not from behind a raised shield
        let target = if cam_ctl.ads && p.armed() && !p.shield_up {
            gun(p.gun).zoom_deg
        } else {
            62.0
        }
        .to_radians();
        persp.fov += (target - persp.fov) * (16.0 * time.delta_secs()).min(1.0);
    }
}

/// First-person hands + weapon: only in FP, swapped to the carried gun,
/// bobbing with the run, kicking with each shot, dipping through reloads.
fn fp_viewmodel(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    vm: Res<VmRig>,
    mut q: Query<(&mut Transform, &mut Visibility), Without<MainCam>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let show = cam_ctl.first_person && p.alive() && p.roll_t <= 0.0;
    if let Ok((_, mut v)) = q.get_mut(vm.root) {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !show {
        return;
    }
    let slot = weapon_slot(p.gun);
    let scoped = cam_ctl.ads && gun(p.gun).scoped;
    for (wi, we) in vm.weapons.iter().enumerate() {
        if let Ok((_, mut v)) = q.get_mut(*we) {
            // the raised shield replaces the gun view; a scoped AWM view
            // is all glass (the overlay), no viewmodel in the way
            *v = if slot == Some(wi) && !p.shield_up && !scoped {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    if let Ok((_, mut v)) = q.get_mut(vm.shield) {
        *v = if p.shield_up {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let spec = gun(p.gun);
    let jerk = if p.armed() && spec.fire_period > 0.0 {
        ((p.fire_cd / spec.fire_period).clamp(0.0, 1.0)).powi(3)
    } else {
        0.0
    };
    let speed = (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt();
    let t = time.elapsed_secs();
    let (bx, by) = if speed > 0.5 {
        let a = (speed / SPRINT_SPEED).clamp(0.0, 1.0);
        ((t * 7.0).sin() * 0.014 * a, (t * 14.0).sin().abs() * 0.011 * a)
    } else {
        (0.0, (t * 2.2).sin() * 0.004) // idle breathing
    };
    let reloading = p.reload_t > 0.0;
    if let Ok((mut tf, _)) = q.get_mut(vm.root) {
        tf.translation = Vec3::new(
            bx,
            by - if reloading { 0.14 } else { 0.0 },
            jerk * 0.07, // camera -Z is forward, so +Z kicks the gun back
        );
        tf.rotation = Quat::from_rotation_x(
            jerk * 0.20 + if reloading { 0.45 } else { 0.0 },
        );
    }
}

/// The red aiming laser for bow/spear: sample the sim's own `predict_arc`
/// so the dots trace EXACTLY the flight the projectile will take, and drop
/// a landing ring where it ends.
fn arc_preview(
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    arc: Res<ArcVis>,
    cam_q: Query<&Transform, With<MainCam>>,
    mut q: Query<(&mut Transform, &mut Visibility), Without<MainCam>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let spec = gun(p.gun);
    let show = cam_ctl.ads && p.alive() && spec.projectile.is_some() && p.roll_t <= 0.0;
    if !show {
        for e in arc.dots.iter().chain(std::iter::once(&arc.ring)) {
            if let Ok((_, mut v)) = q.get_mut(*e) {
                *v = Visibility::Hidden;
            }
        }
        return;
    }
    let Ok(cam_tf) = cam_q.get_single() else {
        return;
    };
    // the same launch the sim will use: from the muzzle (crouch + lean
    // included), toward the crosshair
    let fwd = cam_tf.forward().as_vec3();
    let far = cam_tf.translation + fwd * 90.0;
    let eye = Vec3::from_array(game.sim.muzzle_origin(game.sim.player));
    let d = (far - eye).normalize_or_zero();
    let (v0, _) = spec.projectile.unwrap();
    let (pts, impact, normal) = game.sim.predict_arc(
        [eye.x, eye.y, eye.z],
        [d.x, d.y, d.z],
        v0,
        p.gun == GunKind::Spear,
        5.0,
    );
    for (i, e) in arc.dots.iter().enumerate() {
        let Ok((mut t, mut v)) = q.get_mut(*e) else {
            continue;
        };
        // short flights use one dot per sample; long ones spread evenly
        let idx = if pts.len() > arc.dots.len() {
            i * (pts.len() - 1) / (arc.dots.len() - 1).max(1)
        } else {
            i
        };
        match pts.get(idx) {
            Some(pt) => {
                t.translation = Vec3::from_array(*pt);
                *v = Visibility::Visible;
            }
            None => {
                *v = Visibility::Hidden;
            }
        }
    }
    if let Ok((mut t, mut v)) = q.get_mut(arc.ring) {
        let n = Vec3::from_array(normal).normalize_or(Vec3::Y);
        t.translation = Vec3::from_array(impact) + n * 0.05;
        t.rotation = Quat::from_rotation_arc(Vec3::Y, n);
        *v = Visibility::Visible;
    }
}

/// The AWM's full-screen glass (§4): curtains + lens ring + fine cross,
/// shown only while scoped in. The FOV drop rides the normal ADS path.
fn scope_overlay(
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    mut q: Query<&mut Visibility, With<ScopeRoot>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let show =
        cam_ctl.ads && p.alive() && gun(p.gun).scoped && p.roll_t <= 0.0 && !p.shield_up;
    for mut v in &mut q {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// §5 detail LOD: the extra greebles on every weapon model appear only
/// while aiming (they inherit their weapon's visibility otherwise).
fn ads_detail(cam_ctl: Res<CamCtl>, mut q: Query<&mut Visibility, With<AdsDetail>>) {
    for mut v in &mut q {
        *v = if cam_ctl.ads {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// World-space checkpoint rings tint to their owner.
fn checkpoint_rings(
    game: Res<Game>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q: Query<(&CheckpointVis, &MeshMaterial3d<StandardMaterial>)>,
) {
    for (cv, mat) in &q {
        let Some(cp) = game.sim.checkpoints.get(cv.index) else {
            continue;
        };
        if let Some(m) = materials.get_mut(&mat.0) {
            let (r, g, b) = match cp.owner {
                Some(Team::Blue) => (0.25, 0.5, 1.0),
                Some(Team::Red) => (1.0, 0.3, 0.25),
                None => (0.85, 0.85, 0.9),
            };
            m.base_color = Color::srgba(r, g, b, 0.30);
            m.emissive = LinearRgba::new(r * 1.2, g * 1.2, b * 1.2, 1.0);
        }
    }
}

/// The minimap (§12): teammates, objectives, and your facing — M or the
/// settings page toggles it.
#[allow(clippy::type_complexity)]
fn minimap_system(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    state: Res<State<GameState>>,
    mut settings: ResMut<GameSettings>,
    mut qs: ParamSet<(
        Query<&mut Visibility, With<MinimapRoot>>,
        Query<(&MinimapDot, &mut Node, &mut Visibility)>,
        Query<(&MinimapCp, &mut Node, &mut BorderColor, &mut Visibility)>,
        Query<(&mut Node, &mut Visibility), With<MinimapHill>>,
        Query<(&mut Node, &mut Transform), With<MinimapPlayer>>,
    )>,
) {
    // the M hotkey only means "minimap" during actual play — not while
    // typing nothing in particular on a menu screen
    if keys.just_pressed(KeyCode::KeyM) && *state.get() == GameState::Playing {
        settings.minimap = !settings.minimap;
    }
    let in_match = matches!(state.get(), GameState::Playing | GameState::Paused);
    let show = settings.minimap && in_match;
    for mut v in &mut qs.p0() {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !show {
        return;
    }
    let simr = &game.sim;
    let half = simr.half;
    let px = MINIMAP_PX - 10.0;
    let to_map = |x: f32, z: f32| {
        (
            (x + half) / (half * 2.0) * px,
            (1.0 - (z + half) / (half * 2.0)) * px,
        )
    };
    // teammates (the player has their own gold marker)
    let p_team = simr.fighters[simr.player].team;
    let mates: Vec<&Fighter> = simr
        .fighters
        .iter()
        .enumerate()
        .filter(|(i, f)| *i != simr.player && f.team == p_team && f.alive())
        .map(|(_, f)| f)
        .collect();
    for (dot, mut node, mut v) in &mut qs.p1() {
        match mates.get(dot.0) {
            Some(f) => {
                let (u, w) = to_map(f.pos[0], f.pos[2]);
                node.left = Val::Px(u);
                node.top = Val::Px(w);
                *v = Visibility::Inherited;
            }
            None => *v = Visibility::Hidden,
        }
    }
    // objectives: forward-spawn rings, colored by owner
    for (cp, mut node, mut border, mut v) in &mut qs.p2() {
        match simr.checkpoints.get(cp.0) {
            Some(c) => {
                let (u, w) = to_map(c.pos[0], c.pos[2]);
                node.left = Val::Px(u - 2.0);
                node.top = Val::Px(w - 2.0);
                *border = BorderColor(match c.owner {
                    Some(Team::Blue) => Color::srgb(0.3, 0.6, 1.0),
                    Some(Team::Red) => Color::srgb(1.0, 0.35, 0.3),
                    None => Color::WHITE,
                });
                *v = Visibility::Inherited;
            }
            None => *v = Visibility::Hidden,
        }
    }
    // the hill (KOTH only)
    for (mut node, mut v) in &mut qs.p3() {
        if simr.mode == Mode::Koth {
            let (u, w) = to_map(simr.hill[0], simr.hill[2]);
            node.left = Val::Px(u);
            node.top = Val::Px(w);
            *v = Visibility::Inherited;
        } else {
            *v = Visibility::Hidden;
        }
    }
    // YOU + facing needle
    let me = &simr.fighters[simr.player];
    for (mut node, mut tfm) in &mut qs.p4() {
        let (u, w) = to_map(me.pos[0], me.pos[2]);
        node.left = Val::Px(u);
        node.top = Val::Px(w);
        tfm.rotation = Quat::from_rotation_z(me.yaw);
    }
}

// ------------------------------------------------------------------- sound

fn play(commands: &mut Commands, h: &Handle<AudioSource>, vol: f32) {
    commands.spawn((
        AudioPlayer::new(h.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::new(vol)),
    ));
}

fn sfx_system(
    mut commands: Commands,
    game: Res<Game>,
    sfx: Res<Sfx>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    settings: Res<GameSettings>,
    mut st: ResMut<SfxState>,
) {
    let simr = &game.sim;
    // rematch / restart: sim clock went backwards — reset the tracker
    if simr.t < st.last_t {
        *st = SfxState::default();
    }
    // An event is fresh if it was created since the last sfx run. Its age
    // and `elapsed` advance in exact DT lockstep, so the epsilon must be
    // SUBTRACTED — adding it replays every event a second time.
    let elapsed = (simr.t - st.last_t).max(0.0);
    let fresh = |age: f32| age < elapsed - 1e-4;
    let p = &simr.fighters[simr.player];
    let p_team = p.team;

    // your own shot — fire_cd jumps up on the tick you fire (an ammo delta
    // misses the shot fired the same tick a reload completes)
    if p.alive() && p.gun == st.prev_gun && p.fire_cd > st.prev_fire_cd {
        play(&mut commands, shot_sound(&sfx, p.gun), 0.8);
    }
    // dry fire: trigger down on an empty magazine → the click (whatever
    // the reserve says — an empty chamber clicks until you reload)
    st.click_cd = (st.click_cd - elapsed).max(0.0);
    let fire_held = if settings.swap_mouse {
        buttons.pressed(MouseButton::Left)
    } else {
        buttons.pressed(MouseButton::Right)
    } || keys.pressed(KeyCode::KeyT);
    if fire_held
        && p.alive()
        && p.armed()
        && p.ammo == 0
        && p.reload_t <= 0.0
        && !p.shield_up
        && st.click_cd <= 0.0
    {
        play(&mut commands, &sfx.click, 0.5);
        st.click_cd = 0.35;
    }
    // shield raised / lowered: the plate clanks onto the arm (gated
    // across the respawn reset so rebirth doesn't phantom-clank)
    if st.prev_alive && p.alive() && p.shield_up != st.prev_shield {
        play(&mut commands, &sfx.shield, 0.35);
    }
    // everyone else's gunfire (hitscan tracers), quieter with distance
    let mut bot_shots = 0;
    for tr in &simr.tracers {
        if !fresh(0.06 - tr.ttl) {
            continue;
        }
        let d2 = (tr.from[0] - p.pos[0]).powi(2)
            + (tr.from[1] - (p.pos[1] + EYE_REL)).powi(2)
            + (tr.from[2] - p.pos[2]).powi(2);
        if tr.team == p_team && d2 < 1.0 {
            continue; // that's your own muzzle (already played above)
        }
        if bot_shots < 2 {
            let vol = (1.0 - (d2.sqrt() / 60.0)).clamp(0.05, 0.3);
            play(&mut commands, &sfx.shot_rifle, vol);
            bot_shots += 1;
        }
    }
    // new arrows / spears in the air
    for m in &simr.missiles {
        if m.id >= st.max_missile {
            if m.shooter != simr.player {
                play(
                    &mut commands,
                    if m.is_spear { &sfx.spear } else { &sfx.bow },
                    0.25,
                );
            }
            st.max_missile = m.id + 1;
        }
    }
    // hits: confirms for you, pain for you — a struck SHIELD clanks
    // instead on both ends
    for (ev, ttl) in &simr.hits {
        if !fresh(2.2 - ttl) {
            continue;
        }
        if ev.shooter == simr.player {
            play(
                &mut commands,
                if ev.shielded {
                    &sfx.shield
                } else if ev.zone == HitZone::Head {
                    &sfx.headshot
                } else {
                    &sfx.hit
                },
                0.5,
            );
        } else if ev.victim == simr.player {
            play(
                &mut commands,
                if ev.shielded { &sfx.shield } else { &sfx.hurt },
                0.6,
            );
        }
    }
    // kills
    for (ev, ttl) in &simr.kill_feed {
        if !fresh(5.0 - ttl) {
            continue;
        }
        if ev.killer == simr.player {
            play(&mut commands, &sfx.kill, 0.55);
        }
    }
    // reload start — magazine weapons only: the bow/spear auto-nock is
    // part of the shot, not a reload foley
    if p.reload_t > 0.0
        && st.prev_reload <= 0.0
        && p.alive()
        && gun(p.gun).projectile.is_none()
    {
        play(&mut commands, &sfx.reload, 0.5);
    }
    // jump: vertical velocity kicked upward
    if p.vy > 3.0 && st.prev_vy <= 3.0 {
        play(&mut commands, &sfx.jump, 0.35);
    }
    // dodge roll / breakfall: the tumble whoosh
    if p.roll_t > 0.0 && st.prev_roll <= 0.0 {
        play(&mut commands, &sfx.roll, 0.5);
    }
    // slot swap = draw foley; ammo top-up / armor / health = pickup chime
    // (all gated to a live player so respawn resets stay silent)
    if st.prev_alive && p.alive() && p.gun != st.prev_gun {
        play(&mut commands, &sfx.reload, 0.35);
    } else if st.prev_alive && p.alive() && p.gun == st.prev_gun && p.reserve > st.prev_reserve {
        play(&mut commands, &sfx.pickup, 0.6);
    }
    if p.armor > st.prev_armor + 1.0 {
        play(&mut commands, &sfx.pickup, 0.6);
    }
    if st.prev_alive && p.alive() && p.health > st.prev_health + 2.0 {
        play(&mut commands, &sfx.pickup, 0.6);
    }
    // round end: fanfare only if YOUR team took it; a low sting for a loss
    let over = simr.round_over_t.is_some();
    if over && !st.prev_over {
        if simr.winner == Some(p_team) {
            play(&mut commands, &sfx.win, 0.7);
        } else {
            play(&mut commands, &sfx.kill, 0.45);
        }
    }

    st.last_t = simr.t;
    st.prev_gun = p.gun;
    st.prev_fire_cd = p.fire_cd;
    st.prev_reserve = p.reserve;
    st.prev_reload = p.reload_t;
    st.prev_vy = p.vy;
    st.prev_roll = p.roll_t;
    st.prev_armor = p.armor;
    st.prev_health = p.health;
    st.prev_alive = p.alive();
    st.prev_over = over;
    st.prev_shield = p.shield_up;
}

// --------------------------------------------------------------------- HUD

fn fmt_clock(t: f32) -> String {
    let t = t.max(0.0) as u32;
    format!("{}:{:02}", t / 60, t % 60)
}

fn hud_system(
    game: Res<Game>,
    settings: Res<GameSettings>,
    mut texts: ParamSet<(
        Query<&mut Text, With<HudText>>,
        Query<&mut Text, With<ScoreTimerText>>,
        Query<&mut Text, With<FeedText>>,
        Query<&mut Text, With<HitFeedText>>,
        Query<&mut Text, With<BannerText>>,
        Query<&mut Text, With<PanelInfoText>>,
        Query<&mut Text, With<PanelAmmoText>>,
    )>,
    mut cross: Query<&mut TextColor, With<CrosshairText>>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];

    if let Ok(mut t) = texts.p0().get_single_mut() {
        let mut s = format!(
            "K/D {}/{}   hits {}{}\n",
            p.kills,
            p.deaths,
            p.hits_dealt,
            if p.roll_t > 0.0 {
                "   [ROLLING]"
            } else if p.shield_up {
                "   [SHIELD UP]"
            } else if p.crouch {
                "   [crouched]"
            } else {
                ""
            }
        );
        let mouse = if settings.swap_mouse {
            "RMB aim  LMB/T fire"
        } else {
            "LMB aim  RMB/T fire"
        };
        s += &format!(
            "WASD move  SPACE jump  Q roll  E shield  O 1st/3rd  Z/X lean  1-3 weapons  {mouse}  CTRL crouch  M map  ESC menu"
        );
        **t = s;
    }

    if let Ok(mut t) = texts.p1().get_single_mut() {
        let clock = fmt_clock(simr.match_t);
        let head = if simr.overtime {
            format!("OVERTIME {clock} — sudden death")
        } else {
            clock
        };
        let score = match simr.mode {
            Mode::Tdm => format!(
                "BLUE {:>2} — {:<2} RED   (first to {})",
                simr.score[0] as u32, simr.score[1] as u32, TDM_TARGET
            ),
            Mode::Koth => format!(
                "HILL   BLUE {:>3.0}s — {:<3.0}s RED   (hold {:.0}s)",
                simr.score[0], simr.score[1], KOTH_TARGET_S
            ),
        };
        **t = format!("{head}\n{score}");
    }

    if let Ok(mut t) = texts.p2().get_single_mut() {
        let mut s = String::new();
        for (ev, _) in simr.kill_feed.iter().rev().take(6) {
            s += &format!(
                "{} killed {}{}\n",
                simr.fighters[ev.killer].name,
                simr.fighters[ev.victim].name,
                if ev.headshot { "  (HEADSHOT)" } else { "" }
            );
        }
        **t = s;
    }

    if let Ok(mut t) = texts.p3().get_single_mut() {
        let mut s = String::new();
        for (ev, _) in simr.hits.iter().rev().take(4) {
            if ev.shooter == simr.player {
                s += &format!(
                    "You hit {} — {} ({:.0}){}\n",
                    simr.fighters[ev.victim].name,
                    ev.zone.name(),
                    ev.damage,
                    if ev.fatal { "  KILL" } else { "" }
                );
            } else if ev.victim == simr.player {
                s += &format!(
                    "{} hit you — {}\n",
                    simr.fighters[ev.shooter].name,
                    ev.zone.name()
                );
            }
        }
        **t = s;
    }

    if let Ok(mut t) = texts.p4().get_single_mut() {
        **t = match simr.winner {
            Some(Team::Blue) => "BLUE WINS — rematch shortly".to_string(),
            Some(Team::Red) => "RED WINS — rematch shortly".to_string(),
            None => String::new(),
        };
    }

    // status panel: loadout slots, weapon, ammo kind, HP/armor numbers
    if let Ok(mut t) = texts.p5().get_single_mut() {
        **t = if !p.alive() {
            format!("DOWN — respawn in {:.1}s", p.respawn_t.max(0.0))
        } else {
            let slots = (0..3)
                .map(|s| {
                    let tag = if s == p.active { ">" } else { " " };
                    format!("{}{} {}", tag, s + 1, gun(p.inventory[s]).name)
                })
                .collect::<Vec<_>>()
                .join("  ");
            format!(
                "{}\n{}{}\n{}   HP {:.0}{}",
                slots,
                gun(p.gun).name.to_uppercase(),
                if p.shield_up { "  [SHIELD]" } else { "" },
                ammo_kind(p.gun),
                p.health.max(0.0),
                if p.armor > 0.0 {
                    format!("  ARMOR {:.0}", p.armor)
                } else {
                    String::new()
                }
            )
        };
    }
    if let Ok(mut t) = texts.p6().get_single_mut() {
        **t = if !p.alive() {
            String::new()
        } else if p.shield_up {
            "SHIELD".to_string()
        } else if p.reload_t > 0.0 {
            "RELOADING".to_string()
        } else {
            format!("{} / {}", p.ammo, p.reserve)
        };
    }

    // crosshair flash: white → gold on a fresh headshot, red on a fresh hit
    if let Ok(mut tc) = cross.get_single_mut() {
        let fresh = simr
            .hits
            .iter()
            .rev()
            .find(|(ev, ttl)| ev.shooter == simr.player && *ttl > 2.0);
        *tc = TextColor(match fresh {
            Some((ev, _)) if ev.zone == HitZone::Head => Color::srgb(1.0, 0.85, 0.2),
            Some(_) => Color::srgb(1.0, 0.3, 0.25),
            None => Color::srgba(1.0, 1.0, 1.0, 0.9),
        });
    }
}

fn own_bars(
    game: Res<Game>,
    mut hp: Query<
        (&mut Node, &mut BackgroundColor),
        (With<OwnHpFill>, Without<OwnArmorFill>),
    >,
    mut ar: Query<&mut Node, (With<OwnArmorFill>, Without<OwnHpFill>)>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let frac = (p.health / MAX_HEALTH).clamp(0.0, 1.0);
    if let Ok((mut node, mut bg)) = hp.get_single_mut() {
        node.width = Val::Percent(frac * 100.0);
        *bg = BackgroundColor(if frac > 0.55 {
            Color::srgb(0.25, 0.9, 0.35)
        } else if frac > 0.28 {
            Color::srgb(0.95, 0.65, 0.15)
        } else {
            Color::srgb(0.92, 0.18, 0.15)
        });
    }
    if let Ok(mut node) = ar.get_single_mut() {
        node.width = Val::Percent((p.armor / ROBOT_ARMOR_HP).clamp(0.0, 1.0) * 100.0);
    }
}

fn scoreboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    mut root: Query<&mut Visibility, With<ScoreboardRoot>>,
    mut text: Query<&mut Text, With<ScoreboardText>>,
) {
    let show = keys.pressed(KeyCode::Tab) || game.sim.round_over_t.is_some();
    if let Ok(mut v) = root.get_single_mut() {
        *v = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !show {
        return;
    }
    if let Ok(mut t) = text.get_single_mut() {
        let mut s = String::new();
        for (team, label) in [(Team::Blue, "BLUE"), (Team::Red, "RED")] {
            s += &format!(
                "{label}  —  {} pts\n{:<12}{:>4}{:>4}{:>6}   {}\n",
                match game.sim.mode {
                    Mode::Tdm => format!("{:.0}", game.sim.score[TdmSim::team_idx(team)]),
                    Mode::Koth => format!("{:.0}s", game.sim.score[TdmSim::team_idx(team)]),
                },
                "NAME",
                "K",
                "D",
                "HITS",
                "WEAPON"
            );
            let mut rows: Vec<&Fighter> = game
                .sim
                .fighters
                .iter()
                .filter(|f| f.team == team)
                .collect();
            rows.sort_by(|a, b| b.kills.cmp(&a.kills).then(a.deaths.cmp(&b.deaths)));
            for f in rows {
                s += &format!(
                    "{:<12}{:>4}{:>4}{:>6}   {}\n",
                    f.name,
                    f.kills,
                    f.deaths,
                    f.hits_dealt,
                    gun(f.gun).name
                );
            }
            s += "\n";
        }
        **t = s;
    }
}

fn wrap_angle(a: f32) -> f32 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}

/// When you're hit, the screen edge facing the shooter flashes red — you
/// always know which way the bullet came from.
fn damage_indicator(
    game: Res<Game>,
    cam: Res<CamCtl>,
    mut edges: Query<(&DmgEdge, &mut BackgroundColor)>,
) {
    let pi = game.sim.player;
    let ppos = game.sim.fighters[pi].pos;
    let mut inten = [0.0_f32; 4]; // top(front) right bottom(behind) left
    for (ev, ttl) in &game.sim.hits {
        if ev.victim != pi {
            continue;
        }
        let w = (ttl / 2.2).clamp(0.0, 1.0);
        let dx = ev.from[0] - ppos[0];
        let dz = ev.from[2] - ppos[2];
        if dx * dx + dz * dz < 1e-4 {
            continue;
        }
        let rel = wrap_angle(dx.atan2(dz) - cam.yaw);
        let (c, s) = (rel.cos(), rel.sin());
        // screen-right is (−cos yaw, sin yaw) in this game's convention
        // (the playtest-verified A/D mapping), so a shooter at rel −π/2
        // sits screen-RIGHT: sin < 0 → the right strip, not the left
        let idx = if c > 0.5 {
            0
        } else if c < -0.5 {
            2
        } else if s < 0.0 {
            1
        } else {
            3
        };
        inten[idx] = inten[idx].max(w);
    }
    for (e, mut bg) in &mut edges {
        *bg = BackgroundColor(Color::srgba(
            0.85,
            0.08,
            0.08,
            inten[e.0 as usize] * 0.55,
        ));
    }
}

// ---------------------------------------------------------------- menus ---

fn esc_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next.set(GameState::Paused),
            GameState::Paused => next.set(GameState::Playing),
            GameState::Settings | GameState::Manual => next.set(GameState::Paused),
            GameState::Intro => {}
        }
    }
}

/// One row of pick-buttons on the loadout screen.
fn pick_row<C: Component + Copy>(
    p: &mut ChildBuilder,
    label: &str,
    items: &[(&str, C)],
    w: f32,
) {
    p.spawn((Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(8.0),
        align_items: AlignItems::Center,
        ..default()
    },))
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.7, 0.9)),
                Node {
                    width: Val::Px(110.0),
                    ..default()
                },
            ));
            for (name, comp) in items {
                row.spawn((
                    Button,
                    *comp,
                    Node {
                        width: Val::Px(w),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.14, 0.17, 0.22)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(name.to_string()),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

/// The intro is the LOADOUT SCREEN now (§6, §13): weapons per slot,
/// cosmetics, battlefield, difficulty, team size — then deploy.
fn open_intro(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
    cam.ads = false;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.94)),
            IntroRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("JOHN KINGDOM — ARENA"),
                TextFont { font_size: 40.0, ..default() },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
            ));
            p.spawn((
                Text::new("build your LOADOUT — the shield always rides in its own slot (E raises it)\nfull controls: ESC menu > RULES & MANUAL"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(Color::srgb(0.7, 0.78, 0.95)),
            ));
            let prim: Vec<(&str, LoadoutButton)> = PRIMARIES
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(0, *g)))
                .collect();
            pick_row(p, "PRIMARY", &prim, 128.0);
            let sec: Vec<(&str, LoadoutButton)> = SECONDARIES
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(1, *g)))
                .collect();
            pick_row(p, "SECONDARY", &sec, 128.0);
            let spc: Vec<(&str, LoadoutButton)> = SPECIALS
                .iter()
                .map(|g| (gun(*g).name, LoadoutButton(2, *g)))
                .collect();
            pick_row(p, "SPECIAL", &spc, 128.0);
            let hats: Vec<(&str, CosmeticButton)> = HAT_CHOICES
                .iter()
                .enumerate()
                .map(|(i, (n, _))| (*n, CosmeticButton(0, i)))
                .collect();
            pick_row(p, "HAT", &hats, 90.0);
            let tunics: Vec<(&str, CosmeticButton)> = TUNIC_CHOICES
                .iter()
                .enumerate()
                .map(|(i, (n, _))| (*n, CosmeticButton(1, i)))
                .collect();
            pick_row(p, "OUTFIT", &tunics, 90.0);
            let maps: Vec<(&str, MapButton)> =
                MapKind::ALL.iter().map(|m| (m.name(), MapButton(*m))).collect();
            pick_row(p, "BATTLEFIELD", &maps, 160.0);
            let diffs: Vec<(&str, DiffButton)> = Difficulty::ALL
                .iter()
                .map(|d| (d.name(), DiffButton(*d)))
                .collect();
            pick_row(p, "DIFFICULTY", &diffs, 100.0);
            let sizes: Vec<(&str, SizeButton)> =
                vec![("5 v 5", SizeButton(5)), ("8 v 8", SizeButton(8))];
            pick_row(p, "BATTLE SIZE", &sizes, 100.0);
            p.spawn((
                Text::new("\nPICK A MODE TO DEPLOY"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.4)),
            ));
            for (label, which) in [
                ("TEAM DEATHMATCH — first to 30", ModeButton::Tdm),
                ("KING OF THE HILL — hold the center 90 s", ModeButton::Koth),
            ] {
                p.spawn((
                    Button,
                    which,
                    Node {
                        width: Val::Px(420.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

fn intro_buttons(
    mut q: Query<
        (&Interaction, &ModeButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    sel: Res<Selected>,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<GameState>>,
) {
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.24, 0.29, 0.40)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
            Interaction::Pressed => {
                let mode = match which {
                    ModeButton::Tdm => Mode::Tdm,
                    ModeButton::Koth => Mode::Koth,
                };
                game.sim = TdmSim::new(MatchConfig {
                    seed: 0x7EA9,
                    per_team: sel.per_team,
                    mode,
                    map: sel.map,
                    difficulty: sel.difficulty,
                    loadout: sel.loadout,
                });
                game.accum = 0.0;
                game.last_t = 0.0;
                game.rebuild = true;
                next.set(GameState::Playing);
            }
        }
    }
}

/// Shared select-highlight painter for all the pick-rows.
fn paint(bg: &mut BackgroundColor, selected: bool, hovered: bool) {
    *bg = BackgroundColor(if selected {
        Color::srgb(0.20, 0.45, 0.24)
    } else if hovered {
        Color::srgb(0.22, 0.30, 0.24)
    } else {
        Color::srgb(0.14, 0.17, 0.22)
    });
}

fn intro_map_buttons(
    mut q: Query<(&Interaction, &MapButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, mb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.map = mb.0;
        }
    }
    for (i, mb, mut bg) in &mut q {
        paint(&mut bg, sel.map == mb.0, *i == Interaction::Hovered);
    }
}

fn intro_loadout_buttons(
    mut q: Query<(&Interaction, &LoadoutButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, lb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.loadout[lb.0] = lb.1;
        }
    }
    for (i, lb, mut bg) in &mut q {
        paint(&mut bg, sel.loadout[lb.0] == lb.1, *i == Interaction::Hovered);
    }
}

fn intro_cosmetic_buttons(
    mut q: Query<(&Interaction, &CosmeticButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, cb, _) in &mut q {
        if *i == Interaction::Pressed {
            if cb.0 == 0 {
                sel.hat = cb.1;
            } else {
                sel.tunic = cb.1;
            }
        }
    }
    for (i, cb, mut bg) in &mut q {
        let selected = if cb.0 == 0 {
            sel.hat == cb.1
        } else {
            sel.tunic == cb.1
        };
        paint(&mut bg, selected, *i == Interaction::Hovered);
    }
}

fn intro_diff_buttons(
    mut q: Query<(&Interaction, &DiffButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, db, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.difficulty = db.0;
        }
    }
    for (i, db, mut bg) in &mut q {
        paint(&mut bg, sel.difficulty == db.0, *i == Interaction::Hovered);
    }
}

fn intro_size_buttons(
    mut q: Query<(&Interaction, &SizeButton, &mut BackgroundColor), With<Button>>,
    mut sel: ResMut<Selected>,
) {
    for (i, sb, _) in &mut q {
        if *i == Interaction::Pressed {
            sel.per_team = sb.0;
        }
    }
    for (i, sb, mut bg) in &mut q {
        paint(&mut bg, sel.per_team == sb.0, *i == Interaction::Hovered);
    }
}

/// The ONLY place the cursor gets captured: entering actual play.
/// Menus and screens (Intro/Paused/Settings/Manual) all need a live,
/// visible cursor — grabbing on OnExit(Paused) soft-locked the loadout
/// and settings screens whenever they were entered from the pause menu.
fn grab_cursor(mut cam: ResMut<CamCtl>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    cam.grabbed = true;
    if let Ok(mut w) = windows.get_single_mut() {
        w.cursor_options.grab_mode = CursorGrabMode::Locked;
        w.cursor_options.visible = false;
    }
}

fn release_cursor(cam: &mut CamCtl, windows: &mut Query<&mut Window, With<PrimaryWindow>>) {
    cam.grabbed = false;
    if let Ok(mut w) = windows.get_single_mut() {
        w.cursor_options.grab_mode = CursorGrabMode::None;
        w.cursor_options.visible = true;
    }
}

fn close_intro(mut commands: Commands, intro: Query<Entity, With<IntroRoot>>) {
    for e in &intro {
        commands.entity(e).despawn_recursive();
    }
}

fn open_menu(
    mut commands: Commands,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
    cam.ads = false; // no stale scope glass / zoom over the menu
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
                Text::new("PAUSED"),
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
                ("Restart Match", MenuButton::Restart),
                ("Change Mode / Loadout", MenuButton::BackToLoadout),
                ("Settings", MenuButton::Settings),
                ("Rules & Manual", MenuButton::Manual),
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

fn close_menu(mut commands: Commands, menu: Query<Entity, With<MenuRoot>>) {
    // no cursor grab here: Paused can exit to Intro/Settings/Manual,
    // which all need the mouse — OnEnter(Playing) does the grabbing
    for e in &menu {
        commands.entity(e).despawn_recursive();
    }
}

// ---- settings page (§14) -------------------------------------------------

fn open_settings(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
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
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.92)),
            SettingsRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("SETTINGS"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
            ));
            p.spawn((
                Text::new("mode / map / difficulty / team size / loadout changes:\nESC menu > Change Mode / Loadout, then redeploy"),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.78, 0.95)),
            ));
            for (which, kind, label) in [
                (
                    SettingsButton::SwapMouse,
                    Some(SettingsButtonKind::SwapMouse),
                    settings_label_text(SettingsButtonKind::SwapMouse, &settings),
                ),
                (
                    SettingsButton::Minimap,
                    Some(SettingsButtonKind::Minimap),
                    settings_label_text(SettingsButtonKind::Minimap, &settings),
                ),
                (SettingsButton::Back, None, "Back (ESC)".to_string()),
            ] {
                p.spawn((
                    Button,
                    which,
                    Node {
                        width: Val::Px(420.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
                ))
                .with_children(|b| {
                    let mut e = b.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 19.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    if let Some(k) = kind {
                        e.insert(SettingsLabel(k));
                    }
                });
            }
        });
}

fn settings_label_text(kind: SettingsButtonKind, s: &GameSettings) -> String {
    match kind {
        SettingsButtonKind::SwapMouse => {
            if s.swap_mouse {
                "Mouse: LEFT fire / RIGHT aim  (conventional)".to_string()
            } else {
                "Mouse: LEFT aim / RIGHT fire  (this game's default)".to_string()
            }
        }
        SettingsButtonKind::Minimap => {
            format!("Minimap: {}  (M in game)", if s.minimap { "ON" } else { "OFF" })
        }
    }
}

fn close_settings(mut commands: Commands, q: Query<Entity, With<SettingsRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn settings_buttons(
    mut q: Query<
        (&Interaction, &SettingsButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<GameSettings>,
    mut labels: Query<(&SettingsLabel, &mut Text)>,
    mut next: ResMut<NextState<GameState>>,
) {
    let mut dirty = false;
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.24, 0.29, 0.40)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
            Interaction::Pressed => match which {
                SettingsButton::SwapMouse => {
                    settings.swap_mouse = !settings.swap_mouse;
                    dirty = true;
                }
                SettingsButton::Minimap => {
                    settings.minimap = !settings.minimap;
                    dirty = true;
                }
                SettingsButton::Back => next.set(GameState::Paused),
            },
        }
    }
    if dirty {
        for (l, mut t) in &mut labels {
            **t = settings_label_text(l.0, &settings);
        }
    }
}

// ---- rules & manual (§14): generated from the live weapon table ----------

fn open_manual(
    mut commands: Commands,
    settings: Res<GameSettings>,
    mut cam: ResMut<CamCtl>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    release_cursor(&mut cam, &mut windows);
    let (aim_b, fire_b) = if settings.swap_mouse {
        ("RIGHT CLICK", "LEFT CLICK")
    } else {
        ("LEFT CLICK", "RIGHT CLICK")
    };
    let mut manual = format!(
        "CONTROLS\n\
         WASD move · SPACE jump · Q dodge roll (hard landings roll automatically)\n\
         E shield up/down · O or V first/third person · Z / X lean · M minimap\n\
         {aim_b} aim (bow/spear draw the flight arc; AWM opens the scope)\n\
         {fire_b} or T fire · 1/2/3 weapon slots · CTRL/C full crouch\n\
         SHIFT sprint (tap crouch at a sprint to roll) · R reload · TAB scores\n\n\
         THE SHIELD\n\
         Always carried. Blocks the FRONT ARC only (±60°): standing cuts\n\
         damage 65%, crouched 95%. Sides and rear ignore it — FLANK.\n\
         Shield up = no shooting, slow walk.\n\n\
         DAMAGE MODEL\n\
         100 HP. Zones: head ×4, torso ×1, arms/legs ×0.75.\n\
         Baseline M4A1: 2 headshots / 8 body shots.\n\
         AWM: head instant; torso, arms, legs = 2 shots.\n\n\
         CHECKPOINTS ('check back')\n\
         Stand in a white ring uncontested to flip it; your team then\n\
         respawns AT the ring. Contested rings freeze.\n\n\
         MODES\n\
         TDM first to 30 · KOTH hold the center 90 s · 5-min clock,\n\
         80 s sudden-death overtime.\n\n\
         WEAPONS (torso dmg / shots to kill body / head / mag)\n",
    );
    for g in ALL_WEAPONS {
        let s = gun(g);
        let body_stk = if s.projectile.is_some() {
            ((MAX_HEALTH / s.damage).ceil()) as u32
        } else {
            (MAX_HEALTH / (s.damage * s.pellets.max(1) as f32)).ceil() as u32
        };
        let head_stk = if s.projectile.is_some() {
            body_stk
        } else {
            (MAX_HEALTH / (s.damage * HEAD_MULT * s.pellets.max(1) as f32)).ceil() as u32
        };
        let class = match s.class {
            GunClass::Primary => "PRIMARY  ",
            GunClass::Secondary => "SECONDARY",
            GunClass::Special => "SPECIAL  ",
        };
        manual += &format!(
            "{:<14} {class} {:>5.1}{}  body x{}  head x{}{}  mag {}\n",
            s.name,
            s.damage,
            if s.pellets > 1 {
                format!(" x{} pellets", s.pellets)
            } else {
                String::new()
            },
            body_stk,
            head_stk,
            // honesty clause: pellet numbers assume the WHOLE spread lands
            if s.pellets > 1 { " (full spread)" } else { "" },
            s.mag
        );
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
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.94)),
            ManualRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("RULES & MANUAL          (ESC to go back)"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.75, 0.27)),
            ));
            p.spawn((
                Text::new(manual),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn close_manual(mut commands: Commands, q: Query<Entity, With<ManualRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn menu_buttons(
    mut q: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, which, mut bg) in &mut q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.24, 0.29, 0.40)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.16, 0.19, 0.26)),
            Interaction::Pressed => match which {
                MenuButton::Resume => next.set(GameState::Playing),
                MenuButton::Restart => {
                    // same match again — mode, map, difficulty, size kept
                    game.sim = TdmSim::new(game.sim.cfg);
                    game.accum = 0.0;
                    game.last_t = 0.0;
                    game.rebuild = true;
                    next.set(GameState::Playing);
                }
                MenuButton::BackToLoadout => next.set(GameState::Intro),
                MenuButton::Settings => next.set(GameState::Settings),
                MenuButton::Manual => next.set(GameState::Manual),
                MenuButton::Quit => {
                    exit.send(AppExit::Success);
                }
            },
        }
    }
}
