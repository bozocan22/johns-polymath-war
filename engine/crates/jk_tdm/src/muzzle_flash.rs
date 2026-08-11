//! THE MUZZLE FLASH — the flare that leaves the barrel when a gun fires.
//!
//! Until this module the crate had none. Every other "fresh shot" client
//! effect existed (brass, audio, camera kick, scope flinch) and the one
//! the eye actually looks at did not: a rifle on full auto was a silent
//! grey stick with cases falling out of it.
//!
//! ## It has no clock of its own
//!
//! A shot is detected exactly the way `spawn_casings` detects one — the
//! fighter's SHOT CLOCK (`crate::shot_clock`) jumping UP between frames.
//! That helper already answers "which weapon actually fired" for a pilot
//! whose hull mount runs on `gatling_cd` while his carried rifle runs on
//! `fire_cd`, so this file never has to know a chassis exists. One rule,
//! one implementation, five consumers.
//!
//! ## Everything here is COSMETIC
//!
//! What is written: a handful of short-lived child entities under
//! `FighterRig::weapon_root` (and, in first person, under the viewmodel's
//! own weapon model). Nothing is raycast against them, no sim field is
//! read except `gun`, `alive`, `in_mech`, `mech_weapon` and the shot
//! clock, and nothing at all is written back. Delta time is real and the
//! decay is frame-rate dependent by construction — allowed for a cosmetic
//! layer, which is why this is not in `sim.rs`.
//!
//! Per-shot variety (the flare's roll about the barrel) is hashed from
//! the fighter's INDEX and the sim clock, never drawn from the sim's RNG
//! — pulling from that stream would desynchronise replay.
//!
//! ## Where the flare is placed
//!
//! `weapon_parts` builds every gun in weapon-root-local space with +Z as
//! the muzzle direction, so the tip is a constant per gun and the flare
//! is a CHILD of the weapon root at that constant. It therefore inherits
//! the gun's aim, the sprint low-ready, the reload cant and the recoil
//! for free, and cannot drift from the model the way a world-space
//! position recomputed from `yaw` would.
//!
//! `sim.muzzle_origin()` is deliberately NOT used: that is the EYE (the
//! hit test casts from there), and drawing a flash at it puts the flare
//! in the middle of the player's face.
//!
//! ## Wiring
//!
//! Two lines in `main.rs`:
//! ```ignore
//! mod muzzle_flash;
//! // ...
//! .add_plugins(muzzle_flash::MuzzleFlashPlugin)
//! ```

use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::sim::{GunKind, MechWeapon};
use crate::{CamCtl, FighterRig, FighterVis, Game, GameState, VmRig, ALL_WEAPONS, VIEWMODEL_LAYER};

/// How long one flare lives, seconds.
///
/// A muzzle flash is an INSTANT — the real thing is around a
/// millisecond. Anything long enough to be comfortable to photograph is
/// long enough to read as a lingering glow bolted to the barrel, which
/// is the failure mode this number is guarding. 50 ms is about three
/// frames at 60 fps: unmistakably a flash, and short enough that the
/// AK's 100 ms cycle still shows gaps between rounds rather than one
/// continuous flame.
const FLASH_TTL_S: f32 = 0.05;

/// Ceiling on live flares, so a sixteen-fighter firefight cannot spawn
/// an unbounded number in one frame. Same shape as `CASING_CAP`.
const FLASH_CAP: usize = 48;

/// How far the flare reaches down-range, as a multiple of its width.
const FLASH_LENGTH_RATIO: f32 = 2.3;

/// The first-person copy is drawn this much larger than the world one.
///
/// Not a fudge and not a second table: the viewmodel is a DIFFERENT
/// camera with a fixed narrow FOV, and its gun is a 0.9-scale model
/// carried 30 cm from the lens and heavily foreshortened. At true size
/// the AK's flare came back about twelve pixels across in the capture -
/// present, correct, and beneath notice. The world flare is untouched,
/// so third person still measures the real thing.
const FP_FLARE_SCALE: f32 = 1.7;

/// One live flare. `life` is what `ttl` started at, so the decay curve
/// does not have to know the constant.
#[derive(Component)]
struct MuzzleFlash {
    ttl: f32,
    life: f32,
}

/// The two meshes and two materials every flare shares. Built once, in
/// this module, so `ModelKit` — a 60-field struct four other files
/// depend on — does not have to grow for a cosmetic effect.
#[derive(Resource)]
struct FlashAssets {
    /// unit quad in the XY plane, normal +Z
    blade: Handle<Mesh>,
    /// unit sphere
    core: Handle<Mesh>,
    /// the near-white centre
    hot: Handle<StandardMaterial>,
    /// the orange petals around it
    halo: Handle<StandardMaterial>,
}

pub struct MuzzleFlashPlugin;

impl Plugin for MuzzleFlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_flash_assets).add_systems(
            Update,
            (spawn_muzzle_flashes, update_muzzle_flashes)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_flash_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Unlit and double-sided. There is no bloom in this renderer (the
    // cameras are LDR), so a flash cannot be made to glow by pushing
    // `emissive` past 1.0 — it has to BE bright, and it has to be
    // visible from whichever side the blade happens to present.
    let flare = |c: Color| StandardMaterial {
        base_color: c,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        ..default()
    };
    commands.insert_resource(FlashAssets {
        blade: meshes.add(Rectangle::new(1.0, 1.0)),
        core: meshes.add(Sphere::new(0.5)),
        // A hot thing is white in the middle and coloured at the edge.
        // One material would have to choose, and choosing gives you
        // either a pale smear or an orange lozenge.
        hot: materials.add(flare(Color::srgba(1.0, 0.97, 0.85, 0.95))),
        halo: materials.add(flare(Color::srgba(1.0, 0.62, 0.16, 0.70))),
    });
}

/// Where a carried gun's bore ends, in weapon-root-local metres, and how
/// wide its flare should be.
///
/// Every tip below is READ OFF `weapon_parts` rather than estimated:
/// `push_muzzle(parts, y, z, w)` lays a device body at `z` (0.07 long)
/// and a bore recess centred at `z + 0.028` that is 0.03 long, so the
/// bore's front face is at `z + 0.043`. The three guns that never call
/// `push_muzzle` are measured from their own foremost part instead.
///
/// Width is a per-gun number and not a `GunClass` tier on purpose: the
/// classes do not separate what matters here. `M249` is a Primary beside
/// the MP5 and the AWM is a Special beside the BOW, so mapping tiers
/// onto classes would give the biggest-bore weapon in the game the same
/// flare as a submachine gun.
///
/// `None` = nothing that flashes. Fists, and the two tackle weapons,
/// which have no propellant at all.
fn muzzle_tip_local(kind: GunKind) -> Option<(Vec3, f32)> {
    let (tip, size) = match kind {
        GunKind::Fists | GunKind::Bow | GunKind::Spear => return None,
        // no `push_muzzle`: the slide runs to z 0.18 and the black bore
        // cylinder at (0, 0.052, 0.185) is 0.05 long, ending at 0.21
        GunKind::Glock => (Vec3::new(0.0, 0.052, 0.21), 0.055),
        // push_muzzle(0.055, 0.27, ..) -> 0.27 + 0.043
        GunKind::Deagle => (Vec3::new(0.0, 0.055, 0.313), 0.078),
        // push_muzzle(0.03, 0.385, ..)
        GunKind::Mp5 => (Vec3::new(0.0, 0.03, 0.428), 0.062),
        // no `push_muzzle`: the barrel cylinder is 0.48 long centred on
        // z 0.38, so its mouth is at 0.62
        GunKind::Shotgun => (Vec3::new(0.0, 0.045, 0.62), 0.105),
        // push_muzzle(0.045, 0.635, ..)
        GunKind::Ak47 => (Vec3::new(0.0, 0.045, 0.678), 0.085),
        // push_muzzle(0.03, 0.635, ..)
        GunKind::M4 => (Vec3::new(0.0, 0.03, 0.678), 0.080),
        // push_muzzle(0.03, 0.85, ..) puts the bore face at 0.893, but
        // the AWM also carries a slotted brake whose last cut is at
        // z 0.925 - so the flare starts past the brake, not inside it.
        GunKind::Awm => (Vec3::new(0.0, 0.03, 0.94), 0.115),
        // push_muzzle(0.04, 0.73, ..)
        GunKind::M249 => (Vec3::new(0.0, 0.04, 0.773), 0.115),
        // no `push_muzzle`: the six barrels are 0.56 long centred on
        // z 0.26, so the cluster's face is at 0.54. On the bore axis
        // (y 0) because the cluster is centred on the spine.
        GunKind::Minigun => (Vec3::new(0.0, 0.0, 0.55), 0.10),
    };
    Some((tip, size))
}

/// Is this hull mount a GUN — something that burns powder behind a
/// projectile and would flash if it were drawn?
///
/// Spelled as a match for the reason `spawn_casings` spells its own that
/// way: a sixth mount has to state its answer rather than inherit one.
/// The repair beam is the load-bearing case — it drives `gatling_cd`, so
/// it clears the shot-clock test every 0.16 s exactly like a gatling
/// does, and that is how the casing system ended up throwing brass out
/// of a healing emitter before somebody wrote this rule down.
fn mount_is_a_gun(mount: MechWeapon) -> bool {
    match mount {
        MechWeapon::Gatling | MechWeapon::Autocannon => true,
        MechWeapon::Rockets | MechWeapon::Plasma | MechWeapon::Repair => false,
    }
}

/// Does this fighter's CURRENTLY FIRING weapon throw a flash today?
///
/// **Hull mounts are excluded wholesale, and the two halves of that are
/// different.** The rocket pod, the plasma cannon and the repair beam
/// must NEVER flash — see `mount_is_a_gun`. The gatling and the
/// autocannon should and do not yet: every placement in this file is a
/// constant in `weapon_root`-local space, the carried arsenal is stowed
/// while piloting, and a mount's barrel tips live in the chassis rigs'
/// coordinate systems, which this file cannot see. A flare in the wrong
/// place reads as a bug; a missing one reads as unfinished. Deferred
/// deliberately rather than guessed.
fn flashes(gun: GunKind, in_mech: bool) -> bool {
    !in_mech && muzzle_tip_local(gun).is_some()
}

/// How big the flare is at age `t` of its `life`, as a fraction of full.
///
/// Split out as a pure function because a curve only reachable through a
/// Bevy schedule is a curve nobody can test — this crate has already
/// paid for that once with a camera bug that lived inside a system for
/// months.
///
/// It starts at FULL and collapses. The opposite (grow then shrink) was
/// the first instinct and it is wrong: at three frames of life a ramp-in
/// means the first frame — the one the eye lands on — is a dot.
fn flash_scale(remaining: f32, life: f32) -> f32 {
    let k = (remaining / life.max(1e-6)).clamp(0.0, 1.0);
    0.25 + 0.75 * k.sqrt()
}

/// Build one flare under `parent`: two crossed blades along the bore, a
/// disc across it, and a hot core.
///
/// Crossed blades rather than a camera-facing billboard. A billboard
/// needs the camera transform, and there are TWO cameras here (world and
/// viewmodel) looking at the same effect through different layers; a
/// cross is orientation-free, costs four triangles, and is what the
/// effect has looked like in every game that ever shipped one.
fn build_flare(
    commands: &mut Commands,
    fx: &FlashAssets,
    parent: Entity,
    tip: Vec3,
    size: f32,
    roll: f32,
    layer: Option<usize>,
    light: bool,
) {
    let len = size * FLASH_LENGTH_RATIO;
    let root = commands
        .spawn((
            Transform::from_translation(tip).with_rotation(Quat::from_rotation_z(roll)),
            Visibility::Inherited,
            MuzzleFlash { ttl: FLASH_TTL_S, life: FLASH_TTL_S },
        ))
        .id();
    commands.entity(root).set_parent(parent);
    let mut kids: Vec<Entity> = Vec::with_capacity(5);
    // the two blades that CONTAIN the bore axis - these are what read
    // from the side, which is where a third-person camera sits
    for i in 0..2 {
        // rotation_x(90 deg) sends the quad's local +Y to +Z (down the
        // bore) and its normal to -Y; the second blade is the same one
        // rolled a quarter turn about the bore.
        let rot = Quat::from_rotation_z(i as f32 * std::f32::consts::FRAC_PI_2)
            * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        kids.push(
            commands
                .spawn((
                    Mesh3d(fx.blade.clone()),
                    MeshMaterial3d(fx.halo.clone()),
                    Transform {
                        translation: Vec3::new(0.0, 0.0, len * 0.5),
                        rotation: rot,
                        scale: Vec3::new(size * 1.4, len, 1.0),
                    },
                ))
                .id(),
        );
    }
    // A small disc ACROSS the bore, to fill the middle of the star.
    //
    // It was `size * 1.7` on the first capture and that was the one
    // thing wrong with the frame: from directly behind the gun - which
    // is exactly where the shooter's own camera is - a square quad
    // facing down-range presents its flat back face and the flash read
    // as an orange CARD stuck to the barrel. Small enough to be a
    // highlight inside the core rather than a shape of its own.
    kids.push(
        commands
            .spawn((
                Mesh3d(fx.blade.clone()),
                MeshMaterial3d(fx.hot.clone()),
                Transform::from_xyz(0.0, 0.0, len * 0.15)
                    .with_scale(Vec3::new(size * 0.9, size * 0.9, 1.0)),
            ))
            .id(),
    );
    // the hot core at the bore itself - a round body has no silhouette
    // to give away, which is what carries the effect from the one angle
    // no flat quad survives
    kids.push(
        commands
            .spawn((
                Mesh3d(fx.core.clone()),
                MeshMaterial3d(fx.hot.clone()),
                Transform::from_xyz(0.0, 0.0, len * 0.22).with_scale(Vec3::new(
                    size * 1.15,
                    size * 1.15,
                    len * 0.9,
                )),
            ))
            .id(),
    );
    if light {
        // A real flash lights its own scene. Only the local player gets
        // one: sixteen bots on full auto would be sixteen point lights
        // appearing and vanishing every frame, and this renderer's
        // clustered forward path pays for each of them.
        kids.push(
            commands
                .spawn((
                    PointLight {
                        color: Color::srgb(1.0, 0.78, 0.42),
                        intensity: 220_000.0,
                        range: 9.0,
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, len * 0.5),
                ))
                .id(),
        );
    }
    for k in &kids {
        commands.entity(*k).set_parent(root);
    }
    // §2.3: `tag_viewmodel_layer` LATCHES after its first sweep, so a
    // flare spawned into the viewmodel hierarchy mid-match is never
    // stamped by it. Unstamped it would fall to layer 0 - invisible to
    // the viewmodel camera and drawn by the WORLD camera instead, which
    // is a metre-wide flare hanging in the map. Stamped here, explicitly,
    // on the root and every child.
    if let Some(l) = layer {
        commands.entity(root).insert(RenderLayers::layer(l));
        for k in &kids {
            commands.entity(*k).insert(RenderLayers::layer(l));
        }
    }
}

/// Detect fresh shots the way every other shot effect does — the shot
/// clock jumping UP — and put a flare on the barrel that fired.
fn spawn_muzzle_flashes(
    mut commands: Commands,
    game: Res<Game>,
    cam: Res<CamCtl>,
    fx: Option<Res<FlashAssets>>,
    vm: Option<Res<VmRig>>,
    rigs: Query<(&FighterVis, &FighterRig)>,
    live: Query<(), With<MuzzleFlash>>,
    mut prev_cd: Local<Vec<f32>>,
) {
    let Some(fx) = fx else { return };
    let simr = &game.sim;
    prev_cd.resize(simr.fighters.len(), 0.0);
    let mut budget = FLASH_CAP.saturating_sub(live.iter().count());
    for (i, f) in simr.fighters.iter().enumerate() {
        let clock = crate::shot_clock(f);
        let fresh = clock > prev_cd[i] + 1e-6;
        prev_cd[i] = clock;
        if !fresh || budget == 0 {
            continue;
        }
        if !flashes(f.gun, f.in_mech()) {
            continue;
        }
        let Some((tip, size)) = muzzle_tip_local(f.gun) else { continue };
        budget -= 1;
        // per-shot ROLL about the bore, from a render-side hash of the
        // fighter's index and the sim clock. Never the sim's RNG: that
        // stream is replay state, and drawing from it here would make
        // the picture change what the replay does.
        let h = ((i as f32 * 12.9898 + simr.t * 78.233).sin() * 43758.55).fract();
        let roll = h * std::f32::consts::TAU;
        for (vis, rig) in &rigs {
            if vis.index != i {
                continue;
            }
            build_flare(
                &mut commands,
                &fx,
                rig.weapon_root,
                tip,
                size,
                roll,
                None,
                i == simr.player,
            );
        }
        // ...and the first-person copy, on the viewmodel's own gun. The
        // world rig above is behind the camera in first person, so
        // without this the one player who is definitely looking at the
        // muzzle is the one who never sees it fire.
        if i == simr.player && cam.first_person {
            if let Some(vm) = &vm {
                if let Some(slot) = ALL_WEAPONS.iter().position(|w| *w == f.gun) {
                    build_flare(
                        &mut commands,
                        &fx,
                        vm.weapons[slot],
                        tip,
                        size * FP_FLARE_SCALE,
                        roll + 1.0,
                        Some(VIEWMODEL_LAYER),
                        false,
                    );
                }
            }
        }
    }
}

/// Age every flare, collapse it, and take it off the barrel.
fn update_muzzle_flashes(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut MuzzleFlash, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (e, mut fl, mut tf) in &mut q {
        fl.ttl -= dt;
        if fl.ttl <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        tf.scale = Vec3::splat(flash_scale(fl.ttl, fl.life));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim;

    /// The healing beam must not look like a gun.
    ///
    /// It drives `gatling_cd`, so it passes the shot-clock test every
    /// 0.16 s exactly like the plasma cannon does - which is how the
    /// casing system ended up throwing brass out of a beam emitter
    /// before someone spelled that exclusion out. Fails on the
    /// pre-change code trivially (there was no flash at all), and would
    /// fail on an allow-by-default mount list.
    #[test]
    fn the_repair_beam_and_the_rocket_pod_are_not_guns() {
        assert!(!mount_is_a_gun(MechWeapon::Repair));
        assert!(!mount_is_a_gun(MechWeapon::Plasma));
        assert!(!mount_is_a_gun(MechWeapon::Rockets));
        assert!(mount_is_a_gun(MechWeapon::Gatling));
        assert!(mount_is_a_gun(MechWeapon::Autocannon));
        // ...and NOTHING flashes from inside a chassis today. This half
        // is a stated DEFERRAL, not a rule: the two gun mounts should
        // flash and cannot until someone publishes their barrel tips.
        // A change that implements them is expected to change this line.
        for k in ALL_WEAPONS {
            assert!(!flashes(k, true), "{k:?} flashed from inside a chassis");
        }
    }

    /// Bows, spears and fists burn no powder.
    #[test]
    fn the_tackle_weapons_and_bare_hands_do_not_flash() {
        for k in [GunKind::Bow, GunKind::Spear, GunKind::Fists] {
            assert!(muzzle_tip_local(k).is_none(), "{k:?} was given a muzzle");
            assert!(!flashes(k, false));
        }
        // and the eight firearms plus the minigun all do
        for k in ALL_WEAPONS {
            if matches!(k, GunKind::Bow | GunKind::Spear) {
                continue;
            }
            assert!(flashes(k, false), "{k:?} does not flash");
        }
    }

    /// Every gun that CAN flash has a tip in front of its own grip and a
    /// flare small enough to stay a flash rather than a fireball.
    ///
    /// The bound is stated against the weapon's own geometry: the tip
    /// must sit at least 15 cm down-range of the root (nothing in this
    /// arsenal has a bore that short) and no flare may be wider than a
    /// fifth of a metre.
    #[test]
    fn every_firearms_tip_is_down_range_and_plausibly_sized() {
        for k in ALL_WEAPONS {
            let Some((tip, size)) = muzzle_tip_local(k) else { continue };
            assert!(tip.z > 0.15, "{k:?}: muzzle tip at z {} is inside the grip", tip.z);
            assert!(tip.z < 1.2, "{k:?}: muzzle tip at z {} is off the model", tip.z);
            assert!((0.03..0.20).contains(&size), "{k:?}: flare width {size}");
        }
    }

    /// The flare tips are not free-floating numbers - they have to track
    /// the gun models. The AWM is the longest barrel in `weapon_parts`
    /// and the Glock the shortest, so the ORDER is the claim: if someone
    /// shortens the sniper or stretches the pistol without revisiting
    /// this table, the flash detaches from the barrel.
    #[test]
    fn the_tips_are_ordered_like_the_barrels() {
        let z = |k| muzzle_tip_local(k).unwrap().0.z;
        assert!(z(GunKind::Glock) < z(GunKind::Deagle));
        assert!(z(GunKind::Deagle) < z(GunKind::Mp5));
        assert!(z(GunKind::Mp5) < z(GunKind::Minigun));
        assert!(z(GunKind::Ak47) < z(GunKind::M249));
        assert!(z(GunKind::M249) < z(GunKind::Awm));
        // and the biggest bores get the biggest flare - a pistol flash
        // the size of a machine gun's is the tell that the table was
        // filled in uniformly
        let w = |k| muzzle_tip_local(k).unwrap().1;
        assert!(w(GunKind::Glock) < w(GunKind::Ak47));
        assert!(w(GunKind::Ak47) < w(GunKind::M249));
        assert!(w(GunKind::Ak47) < w(GunKind::Awm));
    }

    /// A flash is an INSTANT. It must start at full size on its very
    /// first frame and be gone inside a rifle's own cycle, or it reads
    /// as a glow welded to the barrel.
    #[test]
    fn the_flare_starts_full_and_collapses() {
        assert!((flash_scale(FLASH_TTL_S, FLASH_TTL_S) - 1.0).abs() < 1e-5);
        let mid = flash_scale(FLASH_TTL_S * 0.5, FLASH_TTL_S);
        assert!(mid < 1.0 && mid > 0.5, "mid-life scale {mid}");
        assert!(flash_scale(0.0, FLASH_TTL_S) < 0.3);
        // shorter than the fastest carried gun's cycle, so consecutive
        // rounds are separate flashes and not one flame
        let fastest = sim::PRIMARIES
            .iter()
            .map(|k| crate::gun(*k).fire_period)
            .fold(f32::MAX, f32::min);
        assert!(
            FLASH_TTL_S < fastest,
            "the flare ({FLASH_TTL_S}s) outlives the fastest cycle ({fastest}s)"
        );
    }
}
