//! The MECH GALLERY — every chassis, every livery, standing still on the
//! training range.
//!
//! ## What the owner asked for
//!
//! "I want you to have 2 other mechs models also displayed both in ally
//! and enemy in training mode, this will help me visually describe the
//! player."
//!
//! Both chassis exist — the heavy `RobotSuit` and the light `ScoutMech` —
//! and each has an ally and an enemy livery, but there has never been a
//! place to SEE them. In a match you occupy one machine and shoot at the
//! other, at range, through dust; the only view of your own is over its
//! shoulder. Four machines side by side under one light is the thing that
//! answers "what am I looking at", and no capture script and no game mode
//! has ever produced that frame.
//!
//! ## Where it lives, and why
//!
//! `Mode::Training` is a real mode in this build (`sim.rs`: the range —
//! targets that hold their ground and never fire). It is the one mode
//! that already promises "nothing here is a fight", so it is where a
//! reference exhibit belongs, and it is the word the owner used.
//!
//! The Forge turntable was the alternative. It is 512x512, renders on its
//! own layer to a texture, and is framed by a UI card — you cannot walk
//! between the machines on it, and it shows exactly ONE rig at a time. The
//! owner asked to walk around and compare; the turntable structurally
//! cannot do that.
//!
//! ## What it may not do
//!
//! Nothing here is known to the sim. These are loose entities with no
//! `FighterVis` and no `FighterRig`, so `sync_fighters` never animates
//! them, no bot ever targets one, and no bullet ever stops on one — they
//! are statues, and the player walks straight through them. Every value
//! below is a constant or comes from the player's spawn position; nothing
//! is drawn from the sim's RNG, so a replay is bit-identical with the
//! gallery on the screen.
//!
//! ## Wiring
//!
//! Two lines in `main.rs`:
//! ```ignore
//! mod mech_lineup;
//! // ...
//! .add_plugins(mech_lineup::MechGalleryPlugin)
//! ```

use bevy::prelude::*;

use crate::branding;
use crate::sim::Mode;
use crate::{
    Game, GameState, LimbMeshes, MainCam, MechTrim, ModelKit, SoldierLook, MECH_HULL_SCALE,
};

/// Everything the gallery owns carries this, so teardown is one query and
/// nothing else in the crate has to know the gallery exists.
#[derive(Component)]
pub struct GalleryVis;

/// A screen-space name plate pinned to a world point.
///
/// UI rather than 3D text because Bevy has no world-space text and the
/// alternative — rendering glyphs to a texture and billboarding a quad —
/// is a hundred lines for a caption. The projection follows the crate's
/// existing `stability_bracket`: `world_to_viewport`, then PERCENT, never
/// pixels. Pixels would be scaled a second time by `UiScale` and the
/// labels would drift off their machines on any window but the authored
/// one.
#[derive(Component)]
struct GalleryLabel {
    anchor: Vec3,
}

/// One exhibit.
#[derive(Clone, Copy)]
enum Chassis {
    /// The heavy `RobotSuit` — hull rig on the soldier's torso, leg
    /// armour on the soldier's leg bones.
    Heavy {
        /// §22: the 1-in-4 royal paint. Shown as a fifth stand rather
        /// than substituted for one of the four, because it is a THIRD
        /// paint on the heavy and not a fourth livery — the ally/enemy
        /// question it does not answer.
        elite: bool,
    },
    /// The light `ScoutMech` — its own chassis, bringing its own legs.
    Scout,
}

/// The exhibit list, left to right as the player sees it.
///
/// Chassis-major, so the two questions the owner asked are two ADJACENT
/// pairs: heavy-ally beside heavy-enemy, scout-ally beside scout-enemy.
/// Grouping by side instead would have put the two liveries of the same
/// machine at opposite ends of the row, which is the one arrangement that
/// makes the comparison hard.
const STANDS: [(Chassis, bool, &str); 5] = [
    (Chassis::Heavy { elite: false }, true, "HEAVY/ALLY"),
    (Chassis::Heavy { elite: false }, false, "HEAVY/ENEMY"),
    (Chassis::Heavy { elite: true }, true, "HEAVY/ROYAL"),
    (Chassis::Scout, true, "SCOUT/ALLY"),
    (Chassis::Scout, false, "SCOUT/ENEMY"),
];

/// Width of a name plate's centring box, in percent of the screen.
///
/// Small on purpose. Five stands 3.6 m apart at a 9 m stand-off are about
/// 11% of the frame apart at this FOV, and the first version's 16% boxes
/// overlapped into an unreadable band — "HEAVYALLYHEAVYENEMY". Narrower
/// than the gap, so two neighbours can touch but never merge.
const LABEL_BOX_PCT: f32 = 9.0;

/// Metres between stand centres. The heavy's hull is ~1.5 m across the
/// shoulder housings and its gatling hangs further out again, so this is
/// a machine's width plus a man's width of walking room.
const STAND_SPACING: f32 = 3.6;

/// How far in front of the player's spawn the row stands, best candidate
/// first.
///
/// Nine metres, which is CLOSE. The hip FOV here is 90 degrees VERTICAL
/// (`FOV_CHOICES`), which is very wide: at fifteen metres a two-metre
/// machine is about fifty pixels tall on a 720p screen and the exhibit
/// reads as five smudges. The row is therefore placed at the distance
/// where it fills the frame from the spawn point, and the player walks
/// closer if they want to look at a knee.
const STAND_RANGE: [f32; 5] = [9.0, 11.0, 13.0, 7.0, 15.0];

/// Plinth radius — a VISUAL number.
///
/// Cut from 1.30 after looking at the first good frame: at that size the
/// dais was the widest thing on the stand and the machine read as a model
/// on a dinner plate. It wants to be about the machine's own footprint.
const PLINTH_R: f32 = 1.05;

/// Clearance radius — a PLACEMENT number, and deliberately larger than
/// the plinth. The machine has to fit, and so does the person walking
/// round it.
const STAND_CLEAR_R: f32 = 1.45;

/// Both chassis wear the SAME armour trim across the row.
///
/// The scout's trim is a separate variant axis (`MechTrim`, four plate
/// weights) with its own lineup capture. Varying it here would mean the
/// ally and enemy scouts differed in plate as well as paint, and a
/// comparison with two variables in it is not a comparison.
const GALLERY_TRIM: MechTrim = MechTrim::Field;

pub struct MechGalleryPlugin;

impl Plugin for MechGalleryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // BEFORE `rebuild_world`, which is what clears
                // `Game::rebuild` — the flag is this system's only signal
                // that a new match exists, and running after it would
                // mean never seeing it set.
                build_gallery.before(crate::rebuild_world),
                place_gallery_labels,
            ),
        );
    }
}

/// (Re)build the row on every match start, and only on the range.
///
/// `Game::rebuild` is the signal, but the build is deferred ONE frame
/// after seeing it. The row is anchored to the player's spawn, and on the
/// frame the flag goes up that position is still being written — the
/// capture harness stages `pos` and `yaw` in `capture_quick_deploy`, and
/// nothing orders that against this. Latching and building next frame
/// makes the anchor independent of system order instead of dependent on
/// an ordering nobody declared.
fn build_gallery(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    kit: Res<ModelKit>,
    game: Res<Game>,
    old: Query<Entity, With<GalleryVis>>,
    mut pending: Local<bool>,
) {
    if game.rebuild {
        *pending = true;
        return;
    }
    if !*pending {
        return;
    }
    *pending = false;
    for e in &old {
        commands.entity(e).despawn_recursive();
    }
    if game.sim.mode != Mode::Training {
        return;
    }
    let p = &game.sim.fighters[game.sim.player];
    let yaw = p.yaw;
    let at = Vec3::new(p.pos[0], p.pos[1], p.pos[2]);
    let row = place_row(&game.sim, at, yaw);

    spawn_row(
        &mut commands,
        &mut meshes,
        &mut materials,
        &kit,
        row.centre,
        row.right,
        row.facing + std::f32::consts::PI,
    );
}

/// The row's two horizontal axes for a facing.
///
/// Extracted so it is testable: the forward/right pair is where a sign
/// error hides, and a mirrored row is invisible in a screenshot taken
/// from the front (it just relabels the machines).
fn row_axes(yaw: f32) -> (Vec3, Vec3) {
    // The crate's convention, from `sync_fighters`: a fighter's root is
    // `Quat::from_rotation_y(yaw)` and its forward is +Z, so yaw 0 looks
    // down +Z and the right hand points down -X.
    let fwd = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let right = Vec3::new(-yaw.cos(), 0.0, yaw.sin());
    (fwd, right)
}

/// The x/z centre of stand `k` of `n`, relative to the row centre.
fn stand_offset(k: usize, n: usize) -> f32 {
    (k as f32 - (n as f32 - 1.0) * 0.5) * STAND_SPACING
}

/// Does a stand at `p` clear every cover block, and stay inside the map?
///
/// The gallery is placed relative to the player's SPAWN, which is a
/// different point on every map and is not chosen with a five-machine row
/// in mind. Without this the row happily materialises inside the Arena's
/// central block.
fn stand_is_clear(sim: &crate::TdmSim, p: Vec3) -> bool {
    if p.x.abs() > sim.half - 2.0 || p.z.abs() > sim.half - 2.0 {
        return false;
    }
    // ...and off the pickup pads. A pad is a lit disc with a floating
    // totem on it, and a stand dropped on one puts two exhibits in the
    // same square metre - the first clean capture staged the viewer on
    // one and spent every frame with an armour-pickup toast across it.
    if sim
        .pickups
        .iter()
        .any(|q| (p.x - q.pos[0]).hypot(p.z - q.pos[2]) < STAND_CLEAR_R + 1.4)
    {
        return false;
    }
    !sim.cover.iter().any(|a| {
        p.x >= a.min[0] - STAND_CLEAR_R
            && p.x <= a.max[0] + STAND_CLEAR_R
            && p.z >= a.min[2] - STAND_CLEAR_R
            && p.z <= a.max[2] + STAND_CLEAR_R
    })
}

/// Is the straight line from `a` to `b` free of cover?
///
/// A footprint test is not enough, and the first capture proved it: on
/// Arena the row landed on perfectly clear ground BEHIND a 1.5 m timber
/// barricade, and three of the five machines were invisible from the
/// spawn. An exhibit you cannot see is not an exhibit.
///
/// Marched rather than solved. A slab test would be exact and this is a
/// placement decision taken once per match, so the cheap version is the
/// right version — and a 0.4 m step cannot miss any block this game
/// builds (the thinnest cover here is a 0.5 m rail).
fn sight_is_clear(sim: &crate::TdmSim, a: Vec3, b: Vec3) -> bool {
    let d = b - a;
    let len = d.length();
    if len < 1e-3 {
        return true;
    }
    let dir = d / len;
    let mut t = 0.5; // skip the metre around the viewer's own body
    while t < len {
        let p = a + dir * t;
        if sim
            .cover
            .iter()
            .any(|c| point_in_block(p, c))
        {
            return false;
        }
        t += 0.4;
    }
    true
}

fn point_in_block(p: Vec3, c: &crate::sim::Aabb) -> bool {
    p.x >= c.min[0]
        && p.x <= c.max[0]
        && p.y >= c.min[1]
        && p.y <= c.max[1]
        && p.z >= c.min[2]
        && p.z <= c.max[2]
}

/// Where the row ends up: its centre, its left-to-right axis, and the yaw
/// the machines are turned to.
#[derive(Clone, Copy)]
struct RowPlacement {
    centre: Vec3,
    right: Vec3,
    /// The direction the row FACES — the reverse of the axis it was laid
    /// out along, so every machine looks back at the viewer.
    facing: f32,
}

/// Yaw offsets tried, in order, from the viewer's own facing.
///
/// Straight ahead first, always: on an open map that is what a player
/// sees the instant the range loads, and it is what the capture script's
/// absolute `look` yaws are authored against. The turns are the fallback
/// for a spawn that faces a wall.
const ROW_TURNS: [f32; 12] = [
    0.0, 0.52, -0.52, 1.05, -1.05, 1.57, -1.57, 2.09, -2.09, 2.62, -2.62, 3.14,
];

/// Chest height, for the sight test. Not eye height: a machine seen only
/// over the top of a barricade is still a machine you cannot walk up to.
const SIGHT_Y: f32 = 1.1;

/// Is the row centred at `centre` clear of cover, and — if `from` is
/// given — visible from there?
fn row_fits(sim: &crate::TdmSim, centre: Vec3, right: Vec3, from: Option<Vec3>) -> bool {
    let n = STANDS.len();
    (0..n).all(|k| {
        let p = centre + right * stand_offset(k, n);
        stand_is_clear(sim, p)
            && from.map_or(true, |a| {
                sight_is_clear(sim, a + Vec3::Y * SIGHT_Y, p + Vec3::Y * SIGHT_Y)
            })
    })
}

/// Lay the row out somewhere the viewer can actually see all of it.
///
/// Three passes, in falling order of how good the answer is:
///   1. around the viewer, and VISIBLE from where they stand;
///   2. around the viewer, clear ground but possibly screened;
///   3. the nearest open patch anywhere on the map.
///
/// The second and third passes exist because the first one genuinely
/// fails: Arena is 85 m square with ninety cover blocks in it, and from
/// several spawn points there is no direction at all in which fourteen
/// metres of unobstructed row will fit. The first build had a single pass
/// and a "straight ahead anyway" fallback, and that is precisely the
/// branch the first capture took — three machines behind a barricade.
fn place_row(sim: &crate::TdmSim, at: Vec3, yaw: f32) -> RowPlacement {
    for require_sight in [true, false] {
        for turn in ROW_TURNS {
            let (fwd, right) = row_axes(yaw + turn);
            for r in STAND_RANGE {
                let centre = at + fwd * r;
                if row_fits(sim, centre, right, require_sight.then_some(at)) {
                    return RowPlacement { centre, right, facing: yaw + turn };
                }
            }
        }
    }
    // Nothing near the viewer. Take the closest open patch on the map and
    // turn the row to face back at them, so walking toward it is walking
    // toward the front of five machines rather than the back of them.
    let mut best: Option<(f32, Vec3, f32)> = None;
    let lim = sim.half - 3.0;
    let mut x = -lim;
    while x <= lim {
        let mut z = -lim;
        while z <= lim {
            let centre = Vec3::new(x, 0.0, z);
            let facing = (at.x - x).atan2(at.z - z);
            let (_, right) = row_axes(facing);
            let d = centre.distance_squared(at);
            if best.map_or(true, |(bd, _, _)| d < bd) && row_fits(sim, centre, right, None) {
                best = Some((d, centre, facing));
            }
            z += 2.0;
        }
        x += 2.0;
    }
    if let Some((_, centre, facing)) = best {
        let (_, right) = row_axes(facing);
        return RowPlacement { centre, right, facing };
    }
    // A map with nowhere at all to stand five machines. Straight ahead is
    // the least surprising way to be wrong.
    let (fwd, right) = row_axes(yaw);
    RowPlacement {
        centre: at + fwd * STAND_RANGE[0],
        right,
        facing: yaw,
    }
}

/// Does the row laid STRAIGHT AHEAD from `at`, at the nearest range, come
/// out clean?
///
/// The capture script aims with absolute yaws measured off a row that is
/// dead ahead at `STAND_RANGE[0]`. Every other branch of `place_row` is
/// therefore poison for it — a run that quietly took the 2.6 rad turn
/// would photograph an empty field and exit 0. This is how the harness
/// finds a spawn where no branch is needed.
pub(crate) fn straight_ahead_is_clean(sim: &crate::TdmSim, at: Vec3, yaw: f32) -> bool {
    let (fwd, right) = row_axes(yaw);
    row_fits(sim, at + fwd * STAND_RANGE[0], right, Some(at))
}

/// A spawn point for the capture harness: somewhere the whole row stands
/// dead ahead at yaw zero, unobstructed, at the authored range.
///
/// A GRID, not a ring. The ring version found nothing at all on Arena and
/// silently handed the caller its fallback, which is how the second run
/// produced a frame indistinguishable from the first. A 2 m grid over the
/// same map finds 271 qualifying spots.
///
/// Nearest to the map centre wins, so the exhibit has the map behind it
/// rather than the inside of the border wall.
pub(crate) fn capture_stage_pos(sim: &crate::TdmSim) -> Option<[f32; 3]> {
    let lim = sim.half - 3.0;
    let mut best: Option<(f32, Vec3)> = None;
    let mut x = -lim;
    while x <= lim {
        let mut z = -lim;
        while z <= lim {
            let at = Vec3::new(x, 0.0, z);
            let d = at.length_squared();
            if best.map_or(true, |(bd, _)| d < bd)
                && stand_is_clear(sim, at)
                && straight_ahead_is_clean(sim, at, 0.0)
            {
                best = Some((d, at));
            }
            z += 2.0;
        }
        x += 2.0;
    }
    best.map(|(_, p)| [p.x, 0.0, p.z])
}

#[allow(clippy::too_many_arguments)]
fn spawn_row(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    kit: &ModelKit,
    centre: Vec3,
    right: Vec3,
    facing: f32,
) {
    // The limb capsules and the shared soldier materials, built once for
    // the whole row — exactly as `spawn_fighter_rigs` does for a match.
    let limbs = LimbMeshes {
        thigh: meshes.add(Capsule3d::new(0.072, 0.15)),
        shin: meshes.add(Capsule3d::new(0.060, 0.15)),
        upper: meshes.add(Capsule3d::new(0.055, 0.14)),
        fore: meshes.add(Capsule3d::new(0.048, 0.12)),
    };
    let plinth_mesh = meshes.add(Cylinder::new(PLINTH_R, 0.10));
    let ring_mesh = meshes.add(Cylinder::new(PLINTH_R + 0.13, 0.04));
    let plinth_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.10, 0.09),
        perceptual_roughness: 0.85,
        ..default()
    });
    let looks = [look_for(materials, true), look_for(materials, false)];
    let ring_mat = [
        materials.add(unlit(branding::signal::ALLY_ACCENT)),
        materials.add(unlit(branding::signal::ENEMY_ACCENT)),
    ];

    let n = STANDS.len();
    for (k, (chassis, ally, label)) in STANDS.into_iter().enumerate() {
        let side = usize::from(!ally);
        let pos = centre + right * stand_offset(k, n);

        // ---- the plinth: a dark dais with a lit rim in the side colour.
        // The rim is the cheapest possible second statement of ally vs
        // enemy, and it stays true when a machine is seen from behind.
        commands.spawn((
            Mesh3d(ring_mesh.clone()),
            MeshMaterial3d(ring_mat[side].clone()),
            Transform::from_translation(pos + Vec3::Y * 0.02),
            GalleryVis,
        ));
        commands.spawn((
            Mesh3d(plinth_mesh.clone()),
            MeshMaterial3d(plinth_mat.clone()),
            Transform::from_translation(pos + Vec3::Y * 0.05),
            GalleryVis,
        ));

        // ---- the machine.
        //
        // A real gameplay rig, built the way `spawn_fighter_rigs` builds
        // one: the soldier body is the SKELETON both chassis hang on, and
        // rebuilding a standalone mech mesh here would be the split brain
        // this project has shipped four times already. `None` for the
        // helmet — the head is hidden under either chassis, so a helmet
        // is geometry nobody will ever see.
        let parts = crate::spawn_soldier_body(
            commands,
            kit,
            &limbs,
            &looks[side],
            false,
            crate::sim::Class::Line,
            None,
        );
        commands.entity(parts.root).insert((
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(facing)),
            GalleryVis,
        ));
        // Both chassis swallow the pilot: head, arms and the carried
        // rifle all go. This is the same set `sync_fighters` hides for a
        // fighter in a mech, written once here because nothing animates
        // these rigs and the visibility is therefore permanent.
        for e in [
            parts.head,
            parts.arm_l[0],
            parts.arm_r[0],
            parts.weapon_root,
        ] {
            commands.entity(e).insert(Visibility::Hidden);
        }

        let head_y = match chassis {
            Chassis::Heavy { elite } => {
                let (rig, _det) = crate::spawn_armor_rig(commands, kit, ally, elite);
                commands
                    .entity(rig)
                    .insert(Transform::from_scale(Vec3::splat(MECH_HULL_SCALE)))
                    .set_parent(parts.torso);
                // the forearm barrier module, stowed — part of the
                // heavy's silhouette in every match, so it is part of the
                // heavy's portrait too. Its FIELD spawns hidden.
                let (barrier, _) = crate::spawn_mech_barrier(commands, kit);
                commands
                    .entity(barrier)
                    .insert(Transform {
                        translation: Vec3::new(-0.645, 0.145, 0.30) * MECH_HULL_SCALE,
                        rotation: Quat::from_rotation_x(-0.10),
                        scale: Vec3::splat(MECH_HULL_SCALE),
                    })
                    .set_parent(rig);
                // D.1 leg armour rides the soldier's own leg bones.
                //
                // Its three group roots spawn HIDDEN — `sync_fighters`
                // owns them in a match, and nothing syncs a statue. The
                // first capture of this exhibit showed three heavy hulls
                // standing on the pilot's own bare white thighs, which
                // is a thing no match has ever rendered. Shown here by
                // hand, once, because these rigs never change state.
                let l = parts.leg_l;
                let r = parts.leg_r;
                for la in [
                    crate::spawn_mech_leg_armor(commands, kit, l[0], l[1], l[2], -1.0, ally, elite),
                    crate::spawn_mech_leg_armor(commands, kit, r[0], r[1], r[2], 1.0, ally, elite),
                ] {
                    for e in la.roots {
                        commands.entity(e).insert(Visibility::Inherited);
                    }
                }
                2.35
            }
            Chassis::Scout => {
                let rig = crate::spawn_scout_chassis(commands, kit, ally, GALLERY_TRIM);
                commands
                    .entity(rig)
                    .insert(Transform::IDENTITY)
                    .set_parent(parts.torso);
                // the light chassis brings its own legs; the soldier's
                // are hidden outright, exactly as in a match.
                for e in [parts.leg_l[0], parts.leg_r[0]] {
                    commands.entity(e).insert(Visibility::Hidden);
                }
                1.95
            }
        };

        // ---- the name plate.
        commands.spawn((
            Text::new(label),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(if ally {
                branding::signal::ALLY_ACCENT
            } else {
                branding::signal::ENEMY_ACCENT
            }),
            Node {
                position_type: PositionType::Absolute,
                // A fixed-width centred box, so the caption sits over the
                // machine's midline whatever the glyph metrics do. A bare
                // text node would be anchored at its LEFT edge and every
                // label would sit half a word to the right of its stand.
                width: Val::Percent(LABEL_BOX_PCT),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            GalleryLabel {
                anchor: pos + Vec3::Y * head_y,
            },
            GalleryVis,
        ));
    }

    // ---- the exhibit light.
    //
    // ONE point light over the middle of the row. The maps light with a
    // low sun and a fog tint, which is right for a battlefield and wrong
    // for a lineup: half the machines end up rim-lit and the near faces
    // of the dark liveries go to black. Warm and wide, so the khaki stays
    // khaki and the iron picks up enough specular to keep its plates.
    commands.spawn((
        PointLight {
            intensity: 2_400_000.0,
            range: 26.0,
            color: Color::srgb(1.0, 0.95, 0.88),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(centre + Vec3::Y * 6.5),
        GalleryVis,
    ));
}

fn unlit(c: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: c,
        unlit: true,
        ..default()
    }
}

/// The soldier material set for one side.
///
/// The same six roles `spawn_fighter_rigs` builds, from the same
/// `branding::signal` constants — the pilot under the chassis is only
/// visible at the ankles under a heavy, but "only visible at the ankles"
/// is exactly where a second, drifting palette would hide.
fn look_for(materials: &mut Assets<StandardMaterial>, ally: bool) -> SoldierLook {
    let side = if ally {
        branding::signal::Side::Ally
    } else {
        branding::signal::Side::Enemy
    };
    let metal = |c: Color, m: f32, r: f32| StandardMaterial {
        base_color: c,
        metallic: m,
        perceptual_roughness: r,
        ..default()
    };
    let shade = |c: Color, k: f32| {
        let s = c.to_srgba();
        Color::srgb(s.red * k, s.green * k, s.blue * k)
    };
    let (ar, ag, ab) = side.accent_rgb();
    let accent = materials.add(StandardMaterial {
        base_color: side.accent(),
        perceptual_roughness: 0.35,
        emissive: LinearRgba::new(
            ar * if ally { 0.40 } else { 0.85 },
            ag * if ally { 0.40 } else { 0.85 },
            ab * if ally { 0.40 } else { 0.85 },
            1.0,
        ),
        ..default()
    });
    let joint = materials.add(metal(Color::srgb_u8(0x17, 0x19, 0x1D), 0.85, 0.22));
    SoldierLook {
        ally,
        shell: materials.add(metal(side.body(), 0.0, 0.42)),
        shell2: materials.add(metal(shade(side.body(), if ally { 0.92 } else { 0.82 }), 0.0, 0.45)),
        accent: accent.clone(),
        stripe: accent,
        hat: materials.add(metal(side.steel(), 0.05, 0.85)),
        joint: joint.clone(),
        knee: materials.add(metal(Color::srgb_u8(0x0E, 0x10, 0x13), 0.20, 0.08)),
        eye: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(0x0A, 0x0B, 0x0D),
            perceptual_roughness: 0.15,
            emissive: LinearRgba::new(0.016, 0.021, 0.028, 1.0),
            ..default()
        }),
        emblem_center: joint,
    }
}

/// Where on screen a world anchor lands, as a (left%, top%) pair, or
/// `None` if it is behind the camera or off the edges.
///
/// Split out from the system because a projection is arithmetic and
/// arithmetic is testable, and because this crate has already paid for
/// leaving camera maths inside a Bevy system it could not call.
fn label_screen_pct(ndc: Vec2, box_w_pct: f32) -> Option<(f32, f32)> {
    // NDC is x,y in -1..1 with +y UP; the UI's origin is TOP-left.
    if !ndc.x.is_finite() || !ndc.y.is_finite() {
        return None;
    }
    // Hard-clipped at the frame edge, not loosely. A label whose machine
    // is off-screen is a label pointing at a crate.
    if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
        return None;
    }
    let left = (ndc.x * 0.5 + 0.5) * 100.0 - box_w_pct * 0.5;
    let top = (0.5 - ndc.y * 0.5) * 100.0;
    Some((left, top))
}

/// Pin each name plate over its machine.
fn place_gallery_labels(
    state: Res<State<GameState>>,
    cam: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut labels: Query<(&GalleryLabel, &mut Node, &mut Visibility)>,
) {
    if labels.is_empty() {
        return;
    }
    // Hidden anywhere but in play: these are absolutely-positioned nodes
    // with no `HudRoot`, so `hud_visibility` does not own them and they
    // would otherwise sit over the pause menu.
    let playing = *state.get() == GameState::Playing;
    let Ok((camera, cam_tf)) = cam.get_single() else {
        for (_, _, mut v) in &mut labels {
            *v = Visibility::Hidden;
        }
        return;
    };
    // BEHIND is answered in world space, not by the NDC z.
    //
    // A perspective projection maps points behind the eye to mirrored
    // positions in front of it, and the reverse-z depth this renderer
    // uses does not make that obvious: the previous version filtered on
    // `ndc.z` and still put a "HEAVY/ALLY" plate over a stack of crates
    // in a frame where that machine was 140 degrees off the view axis.
    let cam_pos = cam_tf.translation();
    let cam_fwd = cam_tf.forward();
    for (label, mut node, mut vis) in &mut labels {
        let in_front = (label.anchor - cam_pos).dot(*cam_fwd) > 0.2;
        let want = (playing && in_front)
            .then(|| camera.world_to_ndc(cam_tf, label.anchor))
            .flatten()
            .and_then(|ndc| label_screen_pct(ndc.truncate(), LABEL_BOX_PCT));
        match want {
            Some((l, t)) => {
                node.left = Val::Percent(l);
                node.top = Val::Percent(t);
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The row must be a ROW: five distinct stands, evenly spaced, and
    /// centred on the point handed in. An off-by-one in the centring term
    /// pushes the whole exhibit half a machine sideways, which is exactly
    /// the kind of thing a screenshot taken from the front cannot show.
    #[test]
    fn the_row_is_centred_and_evenly_spaced() {
        let n = STANDS.len();
        let offs: Vec<f32> = (0..n).map(|k| stand_offset(k, n)).collect();
        let sum: f32 = offs.iter().sum();
        assert!(sum.abs() < 1e-4, "the row is not centred: offsets sum to {sum}");
        for w in offs.windows(2) {
            assert!(
                (w[1] - w[0] - STAND_SPACING).abs() < 1e-4,
                "uneven spacing: {w:?}"
            );
        }
        assert!(
            offs[n - 1] - offs[0] > 2.0 * STAND_CLEAR_R * (n as f32 - 1.0) * 0.5,
            "the stands overlap - there is no room to walk between them"
        );
    }

    /// Forward and right must be perpendicular, horizontal, and correctly
    /// handed for every facing. A flipped `right` mirrors the exhibit:
    /// the labels stay attached to their machines, so the picture still
    /// looks plausible and is simply the wrong way round.
    #[test]
    fn the_row_axes_are_a_right_handed_ground_frame() {
        for k in 0..8 {
            let yaw = k as f32 * std::f32::consts::TAU / 8.0;
            let (f, r) = row_axes(yaw);
            assert!(f.y.abs() < 1e-6 && r.y.abs() < 1e-6, "the row must stay level");
            assert!((f.length() - 1.0).abs() < 1e-5);
            assert!((r.length() - 1.0).abs() < 1e-5);
            assert!(f.dot(r).abs() < 1e-5, "forward and right are not square");
            // Bevy is right-handed, Y up: right x forward = up.
            assert!(
                r.cross(f).y > 0.9,
                "the row is mirrored at yaw {yaw}: right x forward points down"
            );
        }
    }

    /// The exhibit must actually answer the owner's question: both
    /// chassis, and for each of them BOTH sides.
    #[test]
    fn every_chassis_appears_in_both_liveries() {
        let has = |want_heavy: bool, want_ally: bool| {
            STANDS.iter().any(|(c, ally, _)| {
                matches!(c, Chassis::Heavy { elite: false }) == want_heavy && *ally == want_ally
            })
        };
        assert!(has(true, true), "no ally heavy");
        assert!(has(true, false), "no enemy heavy");
        assert!(
            STANDS
                .iter()
                .any(|(c, a, _)| matches!(c, Chassis::Scout) && *a),
            "no ally scout"
        );
        assert!(
            STANDS
                .iter()
                .any(|(c, a, _)| matches!(c, Chassis::Scout) && !*a),
            "no enemy scout"
        );
        assert!(
            STANDS
                .iter()
                .any(|(c, _, _)| matches!(c, Chassis::Heavy { elite: true })),
            "the royal paint is not on show"
        );
        // and every stand says which it is
        for (_, _, name) in STANDS {
            assert!(!name.is_empty());
        }
    }

    /// The projection: centre of screen is centre of screen, the box is
    /// centred on the anchor, and +y NDC is UP while +top is DOWN.
    #[test]
    fn a_label_projects_to_the_top_left_of_a_centred_box() {
        let w = LABEL_BOX_PCT;
        let (l, t) = label_screen_pct(Vec2::ZERO, w).unwrap();
        assert!((l - (50.0 - w * 0.5)).abs() < 1e-4, "left was {l}");
        assert!((t - 50.0).abs() < 1e-4, "top was {t}");
        // an anchor high on the screen must have a SMALL top
        let (_, high) = label_screen_pct(Vec2::new(0.0, 0.8), w).unwrap();
        assert!(high < t, "the y axis is flipped: {high} !< {t}");
        // and one to the right must have a larger left
        let (rl, _) = label_screen_pct(Vec2::new(0.6, 0.0), w).unwrap();
        assert!(rl > l);
        assert!(label_screen_pct(Vec2::new(f32::NAN, 0.0), w).is_none());
        assert!(label_screen_pct(Vec2::new(4.0, 0.0), w).is_none());
    }

    /// A block standing between the viewer and a stand must be caught.
    /// The first capture put three of five machines behind a barricade
    /// on ground that passed the footprint test, so this is the guard
    /// for the defect that actually happened.
    #[test]
    fn a_block_on_the_sight_line_is_seen() {
        let mut sim = crate::TdmSim::new(crate::sim::MatchConfig {
            seed: 0x4242,
            per_team: 2,
            mode: Mode::Training,
            map: crate::sim::MapKind::Arena,
            difficulty: crate::sim::Difficulty::Normal,
            loadout: [crate::sim::GunKind::M4; 3],
            tdm_target: 30,
            class: crate::sim::Class::Line,
            melee_axe: false,
            grenade_preset: 0,
            armor_pieces: None,
        });
        // a wall right across the middle of a 10 m sight line
        sim.cover = vec![crate::sim::Aabb {
            min: [-2.0, 0.0, 4.0],
            max: [2.0, 2.0, 4.6],
        }];
        let a = Vec3::new(0.0, SIGHT_Y, 0.0);
        assert!(
            !sight_is_clear(&sim, a, Vec3::new(0.0, SIGHT_Y, 10.0)),
            "a wall across the line was not seen"
        );
        // and a line that misses it laterally is fine
        assert!(sight_is_clear(&sim, a, Vec3::new(9.0, SIGHT_Y, 10.0)));
        // ...as is one that passes OVER it
        assert!(sight_is_clear(
            &sim,
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::new(0.0, 3.0, 10.0)
        ));
    }
}

