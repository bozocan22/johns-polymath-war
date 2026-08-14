//! CTF presentation — the two flag props, and the "you have it" banner.
//!
//! A separate module rather than another two hundred lines of `main.rs`,
//! for the reason `branding.rs` is one: the wiring is two lines
//! (`mod ctf_vis;` plus `.add_plugins(ctf_vis::CtfVisPlugin)`), so the
//! merge surface against the most-contended file in the crate is two
//! lines instead of a block in the middle of a 28 000-line file.
//!
//! ## Everything here READS the sim and never writes it
//!
//! `sim.flags[i]` is the whole input. The flag prop's position is
//! `flags[i].pos` copied straight onto a `Transform` — the client does
//! not integrate it, predict it, or re-derive where a carried flag
//! "should" be from the carrier's position. The sim already parents the
//! flag to its carrier; doing it again here is the split brain.
//!
//! The only client-side motion is the idle bob and spin on the banner,
//! which is real-time driven, frame-rate dependent, cosmetic, and feeds
//! nothing.

use bevy::prelude::*;

use crate::branding::signal;
use crate::sim::{Mode, TdmSim};
use crate::{Game, GameState};

/// Team index this prop draws. `flags[i]` is team `i`'s OWN flag — the
/// one they defend and the enemy is trying to steal.
#[derive(Component, Clone, Copy)]
pub struct FlagVis(pub usize);

/// The stand the flag belongs to. Drawn at `home` and never moved, so a
/// stolen flag leaves a visible empty socket behind it.
#[derive(Component, Clone, Copy)]
pub struct FlagHomeVis(pub usize);

/// The single line of HUD text that says you are the carrier.
#[derive(Component)]
pub struct CarryBanner;

/// Pole height in final metres. Tall enough to clear the 1.8 m fighters
/// so the banner is visible over a crowd around the stand.
const POLE_H: f32 = 2.6;
const POLE_R: f32 = 0.06;
/// Banner half-extents. A flat slab, not a cloth sim.
const BANNER_W: f32 = 0.9;
const BANNER_H: f32 = 0.55;

// The carried pose. `pos` is the CARRIER's own position, so a flag drawn
// there unchanged stands inside him.
/// How far out to the carrier's right the butt of the pole sits.
const CARRY_OUT: f32 = 0.42;
/// How far up. Roughly hip height on a 1.8 m fighter.
const CARRY_LIFT: f32 = 0.55;
/// Outward rake, radians. Enough that the banner clears the head and the
/// shoulder; not so much that the pole lies flat and reads as dropped.
const CARRY_RAKE: f32 = 0.35;

pub struct CtfVisPlugin;

impl Plugin for CtfVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_carry_banner)
            .add_systems(
                Update,
                (sync_flag_props, drive_flag_props, drive_carry_banner)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Spawn the props the first frame a CTF match is live, and remove them
/// the first frame one is not.
///
/// Spawn-on-demand rather than a hook into `rebuild_world`: that system
/// already carries a six-marker `Or<>` teardown query, and adding a
/// seventh would have put this feature back inside `main.rs`.
fn sync_flag_props(
    mut commands: Commands,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<Entity, Or<(With<FlagVis>, With<FlagHomeVis>)>>,
) {
    let want = game.sim.mode == Mode::Ctf;
    let have = !existing.is_empty();
    if want == have {
        return;
    }
    if !want {
        for e in &existing {
            commands.entity(e).despawn_recursive();
        }
        return;
    }

    // Colour is ALLY/ENEMY relative to the team the player is actually
    // on this round, the same question every other coloured thing in
    // this crate asks. Hardcoding blue/red here would be the one place
    // in the game where "my side" was not gold.
    let me = game.sim.fighters[game.sim.player].team;
    let pole_mesh = meshes.add(Cylinder::new(POLE_R, POLE_H));
    let banner_mesh = meshes.add(Cuboid::new(BANNER_W, BANNER_H, 0.03));
    let ring_mesh = meshes.add(Cylinder::new(0.75, 0.04));

    for (i, flag) in game.sim.flags.iter().enumerate() {
        let team = if i == 0 { crate::sim::Team::Blue } else { crate::sim::Team::Red };
        let side = signal::side_of(team, me);
        let (r, g, b) = side.accent_rgb();
        let banner_mat = materials.add(StandardMaterial {
            base_color: side.accent(),
            // Emissive so the flag stays legible in the shadow of a wall.
            // A flag you cannot find is a mode you cannot play.
            emissive: LinearRgba::new(r * 1.6, g * 1.6, b * 1.6, 1.0),
            ..default()
        });
        let pole_mat = materials.add(StandardMaterial {
            base_color: side.steel(),
            ..default()
        });

        // The stand: a flat ring at `home`, painted once and never moved.
        commands.spawn((
            Mesh3d(ring_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(r, g, b, 0.35),
                emissive: LinearRgba::new(r * 0.8, g * 0.8, b * 0.8, 1.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(flag.home[0], flag.home[1] + 0.03, flag.home[2]),
            FlagHomeVis(i),
        ));

        // The flag itself: pole + banner, a single parent whose
        // translation is overwritten from the sim every frame.
        commands
            .spawn((
                Transform::from_xyz(flag.pos[0], flag.pos[1], flag.pos[2]),
                Visibility::default(),
                FlagVis(i),
            ))
            .with_children(|p| {
                p.spawn((
                    Mesh3d(pole_mesh.clone()),
                    MeshMaterial3d(pole_mat),
                    Transform::from_xyz(0.0, POLE_H * 0.5, 0.0),
                ));
                p.spawn((
                    Mesh3d(banner_mesh.clone()),
                    MeshMaterial3d(banner_mat),
                    // Hung off one edge of the pole, near the top, the
                    // way a banner actually hangs.
                    Transform::from_xyz(BANNER_W * 0.5 + POLE_R, POLE_H - BANNER_H * 0.7, 0.0),
                ));
            });
    }
}

/// Cosmetic idle motion for a flag standing still, in one place so it
/// can be unit-tested without a `World`.
///
/// Returns `(y_offset, yaw)`. A flag AT HOME turns slowly and breathes;
/// a dropped or carried one is dead still, because motion on a dropped
/// flag reads as "something is happening here" when nothing is.
pub fn flag_idle(at_home: bool, carried: bool, t: f32) -> (f32, f32) {
    if carried || !at_home {
        return (0.0, 0.0);
    }
    ((t * 1.7).sin() * 0.06, t * 0.6)
}

fn drive_flag_props(
    game: Res<Game>,
    time: Res<Time>,
    mut q: Query<(&FlagVis, &mut Transform)>,
) {
    if game.sim.mode != Mode::Ctf {
        return;
    }
    let t = time.elapsed_secs();
    for (f, mut tf) in &mut q {
        let Some(flag) = game.sim.flags.get(f.0) else {
            continue;
        };
        let (bob, yaw) = flag_idle(flag.at_home, flag.carrier.is_some(), t);
        match flag.carrier.and_then(|c| game.sim.fighters.get(c)) {
            // CARRIED: slung across the carrier's back.
            //
            // The first cut just lifted the flag 1.15 m at the carrier's
            // own `pos` and left the pole vertical, which put a 2.6 m
            // pole straight through his head - it read as impalement,
            // not as a trophy. It is offset BEHIND him and raked back
            // now, which is what a man running with a standard does.
            //
            // The carrier's yaw is read from the sim and never written.
            // (`f.yaw.sin()`, `f.yaw.cos()` is the forward vector this
            // whole crate uses - see `bot_act`.)
            Some(carrier) => {
                // OUT TO THE SIDE, not behind. Raking it backwards was
                // the second wrong answer and the capture caught it: the
                // third-person boom sits directly behind the carrier, so
                // a backward rake drives the pole through the camera and
                // the banner fills a quarter of the screen. The right
                // shoulder is the one direction that is neither inside
                // the man nor inside the lens.
                let right = (carrier.yaw.cos(), -carrier.yaw.sin());
                tf.translation = Vec3::new(
                    flag.pos[0] + right.0 * CARRY_OUT,
                    flag.pos[1] + CARRY_LIFT,
                    flag.pos[2] + right.1 * CARRY_OUT,
                );
                tf.rotation =
                    Quat::from_rotation_y(carrier.yaw) * Quat::from_rotation_z(-CARRY_RAKE);
            }
            None => {
                tf.translation = Vec3::new(flag.pos[0], flag.pos[1] + bob, flag.pos[2]);
                tf.rotation = Quat::from_rotation_y(yaw);
            }
        }
    }
}

/// The one-line carry state, as text. Pure, so the wording is testable.
///
/// `None` means draw nothing at all — the HUD does not get a permanent
/// "NOT CARRYING" line.
pub fn carry_line(sim: &TdmSim) -> Option<String> {
    if sim.mode != Mode::Ctf {
        return None;
    }
    let me = sim.player;
    let my_idx = TdmSim::team_idx(sim.fighters.get(me)?.team);
    // Their flag is the one you can be holding: `flags[i]` is team i's
    // OWN flag, so the stealable one is the other index.
    if sim.flags[1 - my_idx].carrier == Some(me) {
        return Some("YOU HAVE THEIR FLAG - RUN IT HOME".to_string());
    }
    // Not carrying, but your own flag is out there: the other half of
    // the state a carrier-only indicator would hide.
    if !sim.flags[my_idx].at_home {
        return Some("YOUR FLAG IS TAKEN".to_string());
    }
    None
}

fn spawn_carry_banner(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont { font_size: 20.0, ..default() },
        TextColor(signal::ALLY_ACCENT),
        Node {
            position_type: PositionType::Absolute,
            // Below the reticle, above the weapon strip: the band the
            // HUD already uses for transient state.
            bottom: Val::Percent(26.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        CarryBanner,
    ));
}

fn drive_carry_banner(
    game: Res<Game>,
    mut q: Query<(&mut Text, &mut Visibility), With<CarryBanner>>,
) {
    let line = carry_line(&game.sim);
    for (mut text, mut vis) in &mut q {
        match &line {
            Some(s) => {
                if text.0 != *s {
                    text.0 = s.clone();
                }
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

/// The instrument. No capture script had ever entered CTF - the mode
/// landed sim-side with no way to look at it at all - so the flags would
/// have been shipped on the strength of the code reading correctly,
/// which is the exact failure that left the first-person bow unposed for
/// months.
///
/// Owned here, like `threat_sensor::capture`, so the beats live beside
/// the thing they photograph.
pub mod capture {
    use crate::{beat, CapBeat};

    pub const SCRIPT: &str = "ctf_flag";

    /// The player is planted a few metres off the ENEMY stand facing it
    /// (see the `ctf_flag` arm in `capture_quick_deploy`), so beat one
    /// is already looking at a flag. The later beats walk INTO it, which
    /// is what makes the carry state - the banner and the flag riding on
    /// the shoulder - photographable at all.
    /// The whole thing is over inside four seconds ON PURPOSE. The first
    /// cut ran to 5.8 s and its last frame was a death cam against a
    /// wall: this stages the player six metres from the ENEMY stand,
    /// which is six metres deep in their half, and the defenders shoot
    /// back. A capture script that outlives its subject photographs the
    /// respawn timer.
    pub const BEATS: &[CapBeat] = &[
        // Off-axis for the establishing frame. The subject is aimed
        // STRAIGHT at the stand so that holding W walks onto it, and the
        // boom is rigidly behind the subject - so the on-axis version of
        // this frame put the flag directly behind his head, which is the
        // one place a 2.6 m pole is invisible.
        CapBeat { orbit: Some(1.0), ..beat(0.2) },
        CapBeat { snap: Some("01-enemy-stand"), ..beat(0.9) },
        CapBeat { orbit: Some(0.0), ..beat(1.0) },
        // walk onto it - CTF_FLAG_RADIUS is 1.8 m, so the touch happens
        // en route and there is no need to stop dead on the stand
        CapBeat { press: &[crate::CapKey::K(bevy::prelude::KeyCode::KeyW)], ..beat(1.1) },
        CapBeat {
            release: &[crate::CapKey::K(bevy::prelude::KeyCode::KeyW)],
            ..beat(2.8)
        },
        CapBeat { snap: Some("02-carrying"), ..beat(3.2) },
        // A PROFILE, not a `look`: the boom is rigidly behind the
        // subject, so turning him turns the camera too and the carried
        // pose never changes silhouette. `orbit` swings the boom without
        // turning him - the only way to see the flag from the side.
        CapBeat { orbit: Some(std::f32::consts::FRAC_PI_2), ..beat(3.4) },
        CapBeat { snap: Some("03-carrying-profile"), ..beat(3.9) },
        CapBeat { end: true, ..beat(4.2) },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flag on its stand idles; a carried or dropped one does not.
    /// Before this helper existed the motion lived inline in the system
    /// and nothing could call it.
    #[test]
    fn only_a_home_flag_idles() {
        let (bob, yaw) = flag_idle(true, false, 1.0);
        assert!(bob.abs() > 0.0, "a home flag should breathe");
        assert!(yaw.abs() > 0.0, "a home flag should turn");
        assert_eq!(flag_idle(true, true, 1.0), (0.0, 0.0), "carried flags are still");
        assert_eq!(flag_idle(false, false, 1.0), (0.0, 0.0), "dropped flags are still");
    }

    /// The idle bob never lifts the flag far enough to look detached.
    #[test]
    fn idle_bob_is_small() {
        for i in 0..400 {
            let (bob, _) = flag_idle(true, false, i as f32 * 0.1);
            assert!(bob.abs() <= 0.061, "bob {bob} is a jump, not a breath");
        }
    }
}
