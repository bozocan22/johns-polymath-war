//! §7 (owner spec, 2026-08-10): THE GRENADE, IN THE HAND.
//!
//! The spec asks for the bow/spear grammar applied to throwables:
//! inventory -> equip -> **HOLD IN HAND** -> aim -> release -> physics ->
//! decrement. Every step of that already existed except the one a player
//! can see. G equipped a grenade, LMB wound it up, releasing threw it
//! and lit a 5 s fuse - and the screen showed a rifle the entire time.
//! The only evidence a grenade was in your hand at all was a line of
//! text in the ammo cluster.
//!
//! So this module is the hand. It owns:
//!
//! * four grenade models - frag, flash, smoke and a molotov that is a
//!   BOTTLE rather than a recoloured canister, because §3 of the spec's
//!   trap list applies to throwables as much as to chassis: a variant
//!   identifiable only by colour does not exist at a glance;
//! * a fist and forearm to hold them, since the weapon models carry
//!   their own hands and a grenade shown alone would float;
//! * the wind-up pose, driven off the sim's own `cook_t` against
//!   `THROW_CHARGE_MAX_S` - the arm cocks back and up as the throw
//!   charges, and settles forward when it does not.
//!
//! WHY A MODULE. `main.rs` is 28k lines and the most contended file in
//! the repo; `branding.rs` set the pattern of a self-contained unit
//! wired in with two lines. This is that: `mod held_grenade;` plus one
//! spawn and one system registration.
//!
//! WHAT IT MAY NOT DO. Everything here is cosmetic. The pose reads the
//! sim's clock and never writes it, the per-kind look is chosen by an
//! enum discriminant and never by the sim's RNG, and no number computed
//! in this file reaches a hit position, a damage figure or a fuse.

use bevy::prelude::*;

use crate::sim::{self, ThrowKind};
use crate::{Game, ModelKit};

/// The four models plus their shared arm, all parented under one root.
///
/// A single model whose material is swapped per kind would have been
/// half the code and the wrong answer: the molotov is a different SHAPE,
/// and the moment it is a different shape "swap the material" stops
/// being expressible. Four static children and a visibility pick is the
/// same trick `VmRig` already uses for the four hull mounts.
#[derive(Resource)]
pub struct HeldGrenadeVm {
    /// Parented to the viewmodel root, so it inherits bob, sway and
    /// every other carry motion for free.
    pub root: Entity,
    /// Indexed by `ThrowKind::ALL`.
    pub kinds: [Entity; 4],
}

/// Is a throwable in the player's hand right now?
///
/// Two states mean yes, and they are owned by different halves of the
/// game: `nade_ready` is the client's equip latch (G puts it in the
/// hand, B stows it), `cook_t` is the sim's wind-up clock (LMB held).
/// The second can outlive the first for the frames between the sim
/// steps, so asking only one of them makes the grenade blink.
///
/// Pure, so the visibility rule is testable without a Bevy world - the
/// same reason `vm_rendered` and `vm_hidden_while_scoped` are pure.
pub fn grenade_in_hand(nade_ready: bool, cook_t: f32, alive: bool) -> bool {
    alive && (nade_ready || cook_t > 0.0)
}

/// The carry pose, as (translation, pitch) for a wind fraction 0..1.
///
/// `wind` is `cook_t / THROW_CHARGE_MAX_S`, clamped. At 0 the grenade
/// sits low and right, in the same corner of the frame every carried
/// weapon uses. At 1 the arm has cocked back and UP past the shoulder,
/// which is where an overhand throw is thrown from - and, incidentally,
/// out of the way of the arc preview that appears in the middle of the
/// screen at exactly that moment.
///
/// Pure and tested: the placement of a thing you cannot see from a text
/// editor is the one part of this module that arithmetic can check.
/// FRAMED AGAINST A CAPTURE, twice. The first numbers put the grenade
/// at (0.155, -0.215, -0.395) and cocked it to z -0.180, and the run
/// showed both failures at once: at rest the object was cut in half by
/// the bottom edge of the frame, and at full wind it was 18 cm from a
/// 100-degree lens and filled the middle of the screen as an
/// unreadable gold slab. Held items live further out than intuition
/// says - the rifles sit at z -0.32 and are a metre long.
pub fn hold_pose(wind: f32) -> (Vec3, f32) {
    let w = wind.clamp(0.0, 1.0);
    // ease-out, so the first tenth of the wind moves the arm visibly and
    // the last tenth barely does. A linear cock reads as a machine part.
    let e = 1.0 - (1.0 - w) * (1.0 - w);
    let rest = Vec3::new(0.150, -0.130, -0.480);
    let cocked = Vec3::new(0.235, 0.075, -0.395);
    (rest + (cocked - rest) * e, -0.95 * e)
}

/// Build the four models and the arm. Call from setup, BEFORE
/// `tag_viewmodel_layer` has latched - it walks `VmRig::root` once and
/// stamps the viewmodel render layer on everything under it, so a late
/// spawn would render on the world layer and be invisible in first
/// person and enormous in third.
pub fn spawn_held_grenade_vm(
    commands: &mut Commands,
    kit: &ModelKit,
    materials: &mut Assets<StandardMaterial>,
) -> HeldGrenadeVm {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::Hidden))
        .id();

    // Per-kind palette. Kept as DATA in one table - the spec's own code
    // quality note ("faction visuals kept as DATA") generalises: a
    // fifth throwable should be a row here, not a fifth builder.
    let olive = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.24, 0.14),
        perceptual_roughness: 0.85,
        metallic: 0.25,
        ..default()
    });
    let flash_body = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.56, 0.58),
        perceptual_roughness: 0.35,
        metallic: 0.85,
        ..default()
    });
    let smoke_body = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.33, 0.38),
        perceptual_roughness: 0.60,
        metallic: 0.50,
        ..default()
    });
    let glass = materials.add(StandardMaterial {
        base_color: Color::srgba(0.32, 0.48, 0.30, 0.72),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.15,
        metallic: 0.0,
        ..default()
    });
    let rag = materials.add(StandardMaterial {
        base_color: Color::srgb(0.76, 0.70, 0.55),
        perceptual_roughness: 1.0,
        ..default()
    });
    // The identifying stripe. Real grenades are colour-coded by band and
    // it is the cheapest possible per-kind signal that survives at the
    // 6 cm this object occupies on screen.
    let mut band_of = |c: Color| {
        materials.add(StandardMaterial {
            base_color: c,
            emissive: LinearRgba::new(0.25, 0.25, 0.25, 1.0),
            perceptual_roughness: 0.5,
            ..default()
        })
    };
    let band_frag = band_of(Color::srgb(0.85, 0.72, 0.20));
    let band_flash = band_of(Color::srgb(0.90, 0.90, 0.95));
    let band_smoke = band_of(Color::srgb(0.35, 0.62, 0.86));

    let mut kinds = [Entity::PLACEHOLDER; 4];
    for (i, k) in ThrowKind::ALL.into_iter().enumerate() {
        let e = match k {
            ThrowKind::Frag => canister(commands, kit, olive.clone(), band_frag.clone(), true),
            ThrowKind::Flash => {
                canister(commands, kit, flash_body.clone(), band_flash.clone(), false)
            }
            ThrowKind::Smoke => {
                canister(commands, kit, smoke_body.clone(), band_smoke.clone(), false)
            }
            ThrowKind::Molotov => bottle(commands, kit, glass.clone(), rag.clone()),
        };
        commands
            .entity(e)
            .insert((Transform::IDENTITY, Visibility::Hidden))
            .set_parent(root);
        kinds[i] = e;
    }

    // THE FIST. Spawned once on the root rather than once per kind, so
    // the hand does not flicker when G cycles the type: only the object
    // in it changes.
    //
    // REBUILT AFTER THE FIRST CAPTURE. The first attempt was a palm
    // slab, four finger bars and a thumb - eleven boxes of anatomy at a
    // scale where the whole hand is 90 px tall - and it photographed as
    // a stack of white plates under the grenade, which is worse than no
    // hand at all. The rest of this game's viewmodels do not model
    // hands either: they run a plain forearm cube from the frame edge
    // to the grip and let the weapon carry the rest. This does that,
    // plus ONE rounded mitt and three short fingers over the near face.
    //
    // The mitt is a SPHERE. A cube fist has corners, corners catch the
    // light as flat facets, and a flat facet at this size is the exact
    // thing that read as a plate.
    let fist = commands
        .spawn((
            Transform::from_xyz(0.005, -0.062, 0.040),
            Visibility::default(),
        ))
        .set_parent(root)
        .id();
    // forearm, running back and down out of the frame
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform {
                translation: Vec3::new(0.040, -0.075, 0.190),
                rotation: Quat::from_rotation_x(-0.55) * Quat::from_rotation_z(-0.18),
                scale: Vec3::new(0.062, 0.062, 0.300),
            },
        ))
        .set_parent(fist);
    // the mitt itself, cradling the grenade from below and behind
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(kit.hand.clone()),
            Transform {
                translation: Vec3::new(0.004, 0.002, 0.022),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(0.105, 0.090, 0.110),
            },
        ))
        .set_parent(fist);
    // three fingers over the near face - enough to read as a grip, few
    // enough not to become a grille
    for i in 0..3 {
        let y = 0.006 + i as f32 * 0.028;
        commands
            .spawn((
                Mesh3d(kit.cube.clone()),
                MeshMaterial3d(kit.hand.clone()),
                Transform {
                    translation: Vec3::new(-0.008, y, -0.032),
                    rotation: Quat::from_rotation_y(0.22),
                    scale: Vec3::new(0.070, 0.017, 0.026),
                },
            ))
            .set_parent(fist);
    }

    HeldGrenadeVm { root, kinds }
}

/// A ribbed canister: body, three bands, fuse cap, spoon, and - on the
/// frag only - the pull ring, because that is the one a player is meant
/// to recognise instantly and the one whose silhouette says "grenade".
fn canister(
    commands: &mut Commands,
    kit: &ModelKit,
    body: Handle<StandardMaterial>,
    band: Handle<StandardMaterial>,
    ring: bool,
) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    // ovoid body - taller than wide, the way a hand grenade is
    commands
        .spawn((
            Mesh3d(kit.ball.clone()),
            MeshMaterial3d(body.clone()),
            Transform::from_scale(Vec3::new(0.072, 0.098, 0.072)),
        ))
        .set_parent(root);
    // three ribs. Thin discs rather than a texture: this engine has no
    // imported world textures, so relief is the only detail vocabulary
    // available, and at this size three is the count that still reads.
    for (j, y) in [-0.028_f32, 0.0, 0.028].into_iter().enumerate() {
        let r = 0.076 - (y.abs() * 0.30);
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(if j == 1 { band.clone() } else { kit.grey_dark.clone() }),
                Transform {
                    translation: Vec3::new(0.0, y, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(r, 0.011, r),
                },
            ))
            .set_parent(root);
    }
    // fuse cap
    commands
        .spawn((
            Mesh3d(kit.cyl.clone()),
            MeshMaterial3d(kit.grey_mid.clone()),
            Transform {
                translation: Vec3::new(0.0, 0.056, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(0.036, 0.028, 0.036),
            },
        ))
        .set_parent(root);
    // the SPOON - the safety lever down the side. The one part of a
    // grenade that breaks its silhouette, and therefore the one that
    // stops it reading as a ball at 6 cm.
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(kit.grey_light.clone()),
            Transform {
                translation: Vec3::new(-0.040, 0.018, 0.0),
                rotation: Quat::from_rotation_z(0.10),
                scale: Vec3::new(0.012, 0.098, 0.030),
            },
        ))
        .set_parent(root);
    if ring {
        // Tucked AGAINST the fuse cap, and half the size it was on the
        // first pass: at 0.034 across, hanging 4 cm out to the side, it
        // photographed as a loose gold disc floating beside the grenade
        // rather than a pin ring on it - and at full wind it was the
        // only part of the whole assembly still on screen.
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(kit.gold.clone()),
                Transform {
                    translation: Vec3::new(0.026, 0.052, 0.0),
                    rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    scale: Vec3::new(0.019, 0.005, 0.019),
                },
            ))
            .set_parent(root);
    }
    root
}

/// The molotov: a bottle, not a canister. Different SHAPE, so the pilot
/// can tell which throwable is in the hand without reading the HUD -
/// the same reason the Royal chassis got a crown.
fn bottle(
    commands: &mut Commands,
    kit: &ModelKit,
    glass: Handle<StandardMaterial>,
    rag: Handle<StandardMaterial>,
) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    for (mat, tr, sc) in [
        // body
        (glass.clone(), Vec3::new(0.0, -0.020, 0.0), Vec3::new(0.070, 0.120, 0.070)),
        // shoulder
        (glass.clone(), Vec3::new(0.0, 0.052, 0.0), Vec3::new(0.050, 0.040, 0.050)),
        // neck
        (glass.clone(), Vec3::new(0.0, 0.086, 0.0), Vec3::new(0.028, 0.048, 0.028)),
        // the burning rag stuffed in the neck
        (rag.clone(), Vec3::new(0.0, 0.122, 0.0), Vec3::new(0.024, 0.055, 0.024)),
    ] {
        commands
            .spawn((
                Mesh3d(kit.cyl.clone()),
                MeshMaterial3d(mat),
                Transform { translation: tr, rotation: Quat::IDENTITY, scale: sc },
            ))
            .set_parent(root);
    }
    // a paper label, so the bottle is not one uniform column of glass
    commands
        .spawn((
            Mesh3d(kit.cube.clone()),
            MeshMaterial3d(rag),
            Transform {
                translation: Vec3::new(0.0, -0.022, -0.034),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(0.050, 0.058, 0.006),
            },
        ))
        .set_parent(root);
    root
}

/// Show the right grenade, in the right place, for the current state.
///
/// Reads only. Nothing computed here is handed back to the sim; the
/// wind fraction is the sim's own `cook_t` scaled by the sim's own
/// `THROW_CHARGE_MAX_S`, so the pose and the throw power cannot drift
/// apart the way the mech mount recoil table did.
pub fn sync_held_grenade(
    game: Res<Game>,
    held: Res<HeldGrenadeVm>,
    mut q: Query<(&mut Transform, &mut Visibility)>,
) {
    let p = &game.sim.fighters[game.sim.player];
    // A pilot has no hands free, and a raised shield already replaces
    // the whole view - both would put a grenade inside another model.
    let show = grenade_in_hand(game.nade_ready, p.cook_t, p.alive())
        && !p.in_mech()
        && !p.shield_up;
    if let Ok((_, mut v)) = q.get_mut(held.root) {
        *v = if show { Visibility::Inherited } else { Visibility::Hidden };
    }
    if !show {
        return;
    }
    let sel = (p.throw_sel as usize).min(3);
    for (i, e) in held.kinds.iter().enumerate() {
        if let Ok((_, mut v)) = q.get_mut(*e) {
            *v = if i == sel { Visibility::Inherited } else { Visibility::Hidden };
        }
    }
    let wind = (p.cook_t / sim::THROW_CHARGE_MAX_S).clamp(0.0, 1.0);
    let (tr, pitch) = hold_pose(wind);
    if let Ok((mut t, _)) = q.get_mut(held.root) {
        t.translation = tr;
        t.rotation = Quat::from_rotation_x(pitch) * Quat::from_rotation_y(-0.35);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The visibility rule, both halves. The equip latch alone is not
    /// enough - `cook_t` can be live in the frames after the latch drops
    /// (the throw is resolved by the sim, not by the key handler), and
    /// asking only `nade_ready` made the grenade vanish one frame before
    /// it left the hand.
    #[test]
    fn in_hand_needs_either_clock_and_a_pulse() {
        assert!(!grenade_in_hand(false, 0.0, true), "empty hand");
        assert!(grenade_in_hand(true, 0.0, true), "equipped, not yet wound");
        assert!(grenade_in_hand(false, 0.4, true), "winding after the latch dropped");
        assert!(grenade_in_hand(true, 0.4, true), "both");
        assert!(!grenade_in_hand(true, 0.4, false), "the dead hold nothing");
    }

    /// The pose has to actually MOVE, monotonically, and end up
    /// somewhere a throw could come from - back, up, and further right.
    /// A pose function that returned a constant would compile, render
    /// and look like a bug, which is exactly the failure class this
    /// project keeps paying for.
    #[test]
    fn the_wind_cocks_the_arm_back_and_up() {
        let (rest, rest_pitch) = hold_pose(0.0);
        let (full, full_pitch) = hold_pose(1.0);
        assert!(full.y > rest.y + 0.15, "the hand must rise: {rest:?} -> {full:?}");
        // Back toward the shoulder, but only a little: the capture that
        // set these numbers showed that pulling the hand a long way
        // toward the lens does not read as a wind-up, it reads as the
        // object growing. The RISE carries the pose; the draw is 8 cm.
        assert!(full.z > rest.z + 0.05, "and come BACK toward the shoulder");
        assert!(full.z < -0.30, "but never into the near plane - it fills the frame there");
        assert!(full.x > rest.x, "and outboard, clearing the sight line");
        assert!(full_pitch < rest_pitch - 0.8, "the wrist cocks back");
        // monotone in y over the whole wind, so there is no point at
        // which holding longer visibly un-winds the throw
        let mut prev = f32::NEG_INFINITY;
        for i in 0..=20 {
            let y = hold_pose(i as f32 / 20.0).0.y;
            assert!(y >= prev - 1e-6, "y went backwards at {i}");
            prev = y;
        }
        // and clamped: the sim's cook clock has no ceiling of its own,
        // so an over-long hold must not walk the hand off screen
        assert_eq!(hold_pose(3.0).0, full, "over-wind must clamp");
        assert_eq!(hold_pose(-1.0).0, rest, "negative must clamp");
    }
}
