//! THE BOW'S OWN AIMING LANGUAGE — subtle pre-aim zoom, a circular
//! reticle, and a landing ring that never becomes a second crosshair.
//!
//! ## Why this is its own module
//!
//! Same reason `branding.rs`, `hud.rs` and `inventory_strip.rs` are:
//! `main.rs` is ~31k lines and is the most contended file in the repo,
//! with other lanes editing it concurrently. The wiring is three lines
//! nobody else's diff is likely to collide with:
//!
//! ```ignore
//! mod bow_aim;
//! // ...
//! .add_plugins(bow_aim::BowAimPlugin)
//! ```
//!
//! ...plus two call sites in `main.rs` that ask this module a QUESTION
//! (`preaim_fov_deg`, `bow_draws_own_reticle`, `landing_ring_visible`)
//! rather than growing new logic in place.
//!
//! ## What was already right, and is NOT rebuilt here
//!
//! The hard part — the physics — already existed and is deliberately
//! untouched:
//!
//! - `sim::predict_arc` is the ONE predictor. This module does not
//!   contain a second one, and `arc_preview` in `main.rs` still calls
//!   the sim's. Re-deriving sim state client-side is the split brain.
//! - `arc_preview` already launches the preview from
//!   `sim.muzzle_origin(player)` — the REAL launch point, never screen
//!   centre. That is the owner's governing rule and it was already met.
//! - The arc already scales with draw charge
//!   (`BOW_V0_FULL * bow_power_fraction(bow_draw_t)`), so the preview
//!   grows as the string comes back.
//! - The cone reads the sim's own `aim_spread_of`, not a client copy.
//! - `preaim_shift` moves the bow OUTWARD at pre-aim on purpose, so the
//!   player can see the actual firing direction. Not touched.
//!
//! ## Cosmetic only
//!
//! Every system here takes `Res`, never `ResMut`, on anything the sim
//! reads. Nothing here produces a hit position, a damage number or a
//! spread value. The zoom changes the projection matrix and the
//! sensitivity match that already rides `fov_now`; it does not change
//! where an arrow goes.

use bevy::prelude::*;

use crate::{CamCtl, Game, GameState, GunKind};

// ---- §5/§14: the pre-aim zoom --------------------------------------------

/// The bow's pre-aim magnification. The owner's spec asks for "roughly
/// 1.3x" and is explicit that this is NOT a sniper scope: "Keep the zoom
/// subtle."
pub const BOW_PREAIM_MAG: f32 = 1.3;

/// Vertical FOV for a given magnification over a hip FOV.
///
/// ## This is a TANGENT relation, not a division of degrees
///
/// Magnification is the ratio of angular sizes on the sensor plane, so
/// `tan(new/2) = tan(hip/2) / mag`. Dividing the DEGREES instead
/// (`90 / 1.3 = 69.2`) is the mistake that is easy to make in a text
/// editor and it over-zooms: the honest 1.3x of a 90 deg hip is 75.1
/// deg, six degrees wider than the naive answer. Since the whole note
/// on this feature is "keep it subtle", getting this wrong in the
/// tighter direction would have shipped the exact thing the owner said
/// not to ship.
///
/// Clamped to a sane window so a garbage `mag` cannot invert the
/// projection or collapse the frustum.
pub fn magnified_fov_deg(hip_deg: f32, mag: f32) -> f32 {
    let hip = hip_deg.clamp(1.0, 175.0);
    let m = mag.max(1.0);
    let t = (hip.to_radians() * 0.5).tan() / m;
    (2.0 * t.atan()).to_degrees()
}

/// The FOV a PROJECTILE weapon asks for at full pre-aim.
///
/// ## The conflict this resolves, stated openly
///
/// `main.rs`'s camera zoom block carried this, and it is right about
/// what it saw:
///
/// > §owner: drawing a bow or cocking a spear TIGHTENS THE AIM - it does
/// > not zoom the world. The zoom was hiding the arc and making the draw
/// > feel like a scope instead of a weapon coming up to full power.
///
/// That comment removed a zoom that came from the gun's own `zoom_deg`.
/// For the bow that was 58 deg against a 90 deg hip — a 1.63x pull that
/// narrows the frustum by 32 degrees, which is enough to walk the low
/// half of a lobbed arc off the bottom of the screen. The complaint was
/// never "any zoom at all"; it was that THAT zoom cropped the arc.
///
/// The new spec asks for 1.3x AND (§15) requires that zooming make the
/// trajectory EASIER to read, never crop or hide it. Those are the same
/// requirement seen from two sides, so the fix is a gentler pull, not a
/// second removal: 1.3x of a 90 deg hip is 75.1 deg — a 15 degree
/// narrowing, less than half of what was reverted.
///
/// The SPEAR is deliberately left at hip. This pass is the bow section
/// of the spec only; the spear gets its own section later and inventing
/// its behaviour here would be guessing.
pub fn preaim_fov_deg(hip_deg: f32, gun: GunKind) -> f32 {
    if gun == GunKind::Bow {
        magnified_fov_deg(hip_deg, BOW_PREAIM_MAG)
    } else {
        hip_deg
    }
}

// ---- §7: the circular reticle --------------------------------------------

/// Does the bow draw its OWN aiming mark right now, displacing the
/// normal crosshair?
///
/// This is the same rung of the ladder as `optic_hides_crosshair` in
/// `main.rs` — "this weapon draws its own mark, so do not stack the
/// game crosshair under it" — which is why the call site ORs it into
/// `optic_hidden` rather than inventing a new `CrossFeedback` variant.
/// Hitmarkers still win over both: feedback outranks aiming furniture,
/// and that priority already existed.
///
/// At the HIP the bow keeps the normal crosshair. The circular reticle
/// is a pre-aim mark, not the bow's permanent identity.
pub fn bow_draws_own_reticle(gun: GunKind, ads: bool, in_mech: bool) -> bool {
    gun == GunKind::Bow && ads && !in_mech
}

/// Outer diameter of the reticle circle, in the 720p authoring space the
/// rest of the HUD type ramp uses (`UiScale` handles resolution).
///
/// The owner: "Keep it simple. Do not make it huge. Do not cover the
/// target." A man-sized target at 30 m is roughly 40 px tall at the
/// zoomed FOV, so a 14 px circle sits INSIDE the silhouette without
/// obscuring the head or the centre of mass. The centre mark is 2 px, a
/// point rather than a blob.
const RETICLE_D: f32 = 14.0;
const RETICLE_BORDER: f32 = 1.5;
const RETICLE_DOT: f32 = 2.0;

/// The reticle red.
///
/// §1 of the spec makes RED the colour language. These channels are the
/// palette's already-validated hot red (`branding::team::ENEMY_ACCENT`
/// is `srgb(1.00, 0.13, 0.10)`) rather than a fresh guess, because the
/// sight pass learned that pushing ALL THREE channels high renders
/// salmon-pink under TonyMcMapface tonemapping. Red is doubled; green
/// and blue stay down.
const RETICLE_RGBA: (f32, f32, f32, f32) = (1.0, 0.13, 0.10, 0.92);

/// Marker for the reticle root, so the update system can find it.
#[derive(Component)]
struct BowReticleRoot;

/// The circle itself — the border is what draws, so this carries the
/// `BorderColor` that fades.
#[derive(Component)]
struct BowReticleRing;

/// The centre mark.
#[derive(Component)]
struct BowReticleDot;

/// The circle is a `Node` with a full `BorderRadius` and a border, not a
/// sprite and not a glyph: there are no UI image assets in this project
/// and the bundled font tofus non-ASCII, so a ring glyph is not an
/// option. `inventory_strip` builds its icons the same way.
fn spawn_reticle(mut commands: Commands) {
    let (r, g, b, _) = RETICLE_RGBA;
    commands
        .spawn((
            BowReticleRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // Above the world, below nothing in particular — the game
            // crosshair is hidden whenever this is up, so the two never
            // contend for the centre pixel.
            GlobalZIndex(6),
            Visibility::Hidden,
        ))
        .with_children(|c| {
            c.spawn((
                BowReticleRing,
                Node {
                    width: Val::Px(RETICLE_D),
                    height: Val::Px(RETICLE_D),
                    border: UiRect::all(Val::Px(RETICLE_BORDER)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::MAX,
                BorderColor(Color::srgba(r, g, b, 0.0)),
                BackgroundColor(Color::NONE),
            ))
            .with_children(|c| {
                c.spawn((
                    BowReticleDot,
                    Node {
                        width: Val::Px(RETICLE_DOT),
                        height: Val::Px(RETICLE_DOT),
                        ..default()
                    },
                    BorderRadius::MAX,
                    BackgroundColor(Color::srgba(r, g, b, 0.0)),
                ));
            });
        });
}

/// Alpha for the reticle at a given pre-aim progress.
///
/// It fades IN with `ads_t` rather than popping, on the same clock the
/// FOV rides, so the circle arrives as the world tightens instead of a
/// frame before it. Squared so it stays out of the way through the first
/// half of the raise.
pub fn reticle_alpha(ads_t: f32, active: bool) -> f32 {
    if !active {
        return 0.0;
    }
    let t = ads_t.clamp(0.0, 1.0);
    RETICLE_RGBA.3 * t * t
}

fn paint_reticle(
    game: Res<Game>,
    cam: Res<CamCtl>,
    state: Res<State<GameState>>,
    mut root_q: Query<&mut Visibility, With<BowReticleRoot>>,
    mut ring_q: Query<&mut BorderColor, With<BowReticleRing>>,
    mut dot_q: Query<&mut BackgroundColor, With<BowReticleDot>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    // The state check is INSIDE rather than a `run_if`: gated by
    // `run_if(in_state(Playing))` the system would simply stop running
    // on a pause or a result screen and the circle would be left frozen
    // on top of the menu, which is the failure mode a run condition
    // looks like it prevents and does not.
    let active = *state.get() == GameState::Playing
        && p.alive()
        && bow_draws_own_reticle(p.gun, cam.ads, p.in_mech());
    let a = reticle_alpha(cam.ads_t, active);
    for mut v in &mut root_q {
        *v = if a > 0.004 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let (r, g, b, _) = RETICLE_RGBA;
    for mut bc in &mut ring_q {
        bc.0 = Color::srgba(r, g, b, a);
    }
    for mut bg in &mut dot_q {
        bg.0 = Color::srgba(r, g, b, a);
    }
}

// ---- §12/§15: the landing marker -----------------------------------------

/// Rotation that lays a Y-axis ring flat against a surface.
///
/// Bevy's `Torus` lies in the XZ plane with +Y as its axis, so the ring
/// sits ON the ground when +Y is rotated onto the surface normal.
/// `from_rotation_arc` degenerates when the two are exactly opposed, so
/// a floor normal that arrives as -Y is handled rather than producing a
/// NaN quaternion that would blank the mesh.
pub fn landing_ring_rot(normal: [f32; 3]) -> Quat {
    let n = Vec3::from_array(normal).normalize_or(Vec3::Y);
    if n.dot(Vec3::Y) < -0.9995 {
        Quat::from_rotation_x(std::f32::consts::PI)
    } else {
        Quat::from_rotation_arc(Vec3::Y, n)
    }
}

/// How much to grow the ring with range.
///
/// A fixed world-size ring is unreadable far away and a
/// screen-size-locked one is a giant graphic up close — §17 says keep it
/// subtle. This grows gently and CLAMPS, so the ring never becomes the
/// "huge trajectory graphic" the brief forbids.
pub fn landing_ring_scale(dist_m: f32) -> f32 {
    (1.0 + 0.035 * dist_m.max(0.0)).clamp(1.0, 2.2)
}

/// Should the landing ring be drawn at all?
///
/// ## The conflict this exists to respect
///
/// §12/§15 say the player must see the trajectory through to the landing
/// area. But `arc_preview` carries a hard-won note explaining why the
/// dots are trimmed to `ARC_PREVIEW_SPAN`:
///
/// > the arc SURVIVES, the marker does not. It is trimmed to its near
/// > span so the dots never walk out to the impact point - which for a
/// > flat shot projects onto the centre pixel and would be a landing
/// > marker made of dots.
///
/// and a second note recording that the ring itself was removed because
/// "a flat shot puts the impact point exactly ON the crosshair, so the
/// ring was literally a second reticle the player did not ask for".
///
/// Both are true, and raising the span to 1.0 would recreate exactly
/// that. The resolution is not to choose a side but to notice that the
/// two requirements never apply to the same shot: a landing marker is
/// only INFORMATION when the arrow lands somewhere other than where you
/// are pointing. So the ring is gated on the angular separation between
/// the aim direction and the direction to the impact point.
///
/// - Flat shot, close wall: separation ~0, ring HIDDEN. The circular
///   reticle is the only mark at the centre pixel, as before.
/// - Lobbed shot, or any real range where gravity has bitten: the
///   impact sits visibly below the aim line, separation exceeds the
///   threshold, and the ring appears out there where it is telling the
///   player something they could not otherwise know.
///
/// The dot trail stays trimmed at 0.62 either way — the ring is what
/// closes the gap to the landing point, not more dots.
pub fn landing_ring_visible(eye: Vec3, aim_dir: Vec3, impact: Vec3, min_sep_rad: f32) -> bool {
    let to_impact = impact - eye;
    let d = to_impact.length();
    // Degenerate: impact on top of the muzzle. Nothing useful to say.
    if d < 0.5 {
        return false;
    }
    let a = aim_dir.normalize_or_zero();
    if a == Vec3::ZERO {
        return false;
    }
    let cos = (to_impact / d).dot(a).clamp(-1.0, 1.0);
    cos.acos() > min_sep_rad
}

/// The separation at which the landing ring stops being a second
/// reticle and starts being information. ~1.4 deg: at 25 m that is the
/// impact sitting about 0.6 m below the aim line, which is a drop the
/// player can see and wants marked.
pub const LANDING_SEP_RAD: f32 = 0.024;

/// Lift the ring off the surface by this much along the normal, so it
/// does not z-fight with the ground it is lying on.
pub const LANDING_LIFT_M: f32 = 0.03;

pub struct BowAimPlugin;

impl Plugin for BowAimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_reticle)
            .add_systems(Update, paint_reticle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1.3x of a 90 deg hip is 75.1 deg by the tangent relation, NOT the
    /// 69.2 deg that dividing degrees gives. This test is the whole
    /// reason `magnified_fov_deg` is a named function: mutate the body
    /// to `hip / mag` and this fails.
    #[test]
    fn magnification_is_a_tangent_relation_not_a_division() {
        let got = magnified_fov_deg(90.0, 1.3);
        // Derived independently of the function, not read off its
        // output: hip 90 gives tan(45) = 1 exactly, so the answer is
        // 2*atan(1/1.3) = 2 * 37.5686 = 75.137 degrees. (The first
        // draft of this test asserted 75.24 from arithmetic done by
        // hand in the editor and the test caught it, which is the
        // whole argument for writing the expected value out longhand.)
        let want = 2.0 * (1.0_f32 / 1.3).atan().to_degrees();
        assert!((want - 75.137).abs() < 0.01, "closed form drifted: {want}");
        assert!((got - want).abs() < 1e-3, "1.3x of 90 deg was {got}");
        // and it is strictly WIDER than the naive degree division, which
        // is the direction that keeps the zoom subtle
        assert!(got > 90.0 / 1.3);
    }

    /// The identity case, and the clamp that stops a bad `mag` from
    /// widening the frustum.
    #[test]
    fn magnification_of_one_is_the_hip_fov_and_below_one_cannot_widen() {
        assert!((magnified_fov_deg(90.0, 1.0) - 90.0).abs() < 1e-3);
        assert!((magnified_fov_deg(90.0, 0.5) - 90.0).abs() < 1e-3);
    }

    /// The zoom is SUBTLE, per the owner. Pin the band: a real scope
    /// pull would be far more than this. If someone retunes
    /// `BOW_PREAIM_MAG` up to scope territory this fails and makes them
    /// argue for it.
    #[test]
    fn bow_preaim_zoom_is_subtle_not_a_scope() {
        let hip = 90.0;
        let z = preaim_fov_deg(hip, GunKind::Bow);
        assert!(z < hip, "the bow must actually zoom");
        // narrower than hip, but by less than 20 degrees - the reverted
        // `zoom_deg` pull was 32 degrees and cropped the arc
        assert!(hip - z < 20.0, "zoom pulled {} degrees", hip - z);
        assert!(hip - z > 5.0, "zoom is imperceptible");
    }

    /// The spear is NOT part of this pass and must be untouched.
    #[test]
    fn only_the_bow_zooms() {
        assert_eq!(preaim_fov_deg(90.0, GunKind::Spear), 90.0);
        assert_eq!(preaim_fov_deg(90.0, GunKind::M4), 90.0);
    }

    /// The reticle is a PRE-AIM mark. At the hip the bow keeps the
    /// normal crosshair, and a pilot in a mech is firing hull mounts.
    #[test]
    fn reticle_replaces_the_crosshair_only_while_pre_aiming_a_bow() {
        assert!(bow_draws_own_reticle(GunKind::Bow, true, false));
        assert!(!bow_draws_own_reticle(GunKind::Bow, false, false));
        assert!(!bow_draws_own_reticle(GunKind::Bow, true, true));
        assert!(!bow_draws_own_reticle(GunKind::M4, true, false));
    }

    /// It fades in rather than popping, and is fully absent when not
    /// active however far along `ads_t` happens to be.
    #[test]
    fn reticle_fades_in_and_is_gone_when_inactive() {
        assert_eq!(reticle_alpha(1.0, false), 0.0);
        assert_eq!(reticle_alpha(0.0, true), 0.0);
        let mid = reticle_alpha(0.5, true);
        let full = reticle_alpha(1.0, true);
        assert!(mid > 0.0 && mid < full);
        // squared, so the first half of the raise stays out of the way:
        // linear would put mid at half of full
        assert!(mid < full * 0.5 + 1e-4);
        assert!((full - RETICLE_RGBA.3).abs() < 1e-5);
    }

    /// A FLAT shot must not put a ring on the centre pixel. This is the
    /// regression the previous lane's comment paid for.
    #[test]
    fn a_flat_shot_draws_no_landing_ring() {
        let eye = Vec3::ZERO;
        let aim = Vec3::Z;
        // impact 30 m straight down the aim line
        let impact = Vec3::new(0.0, 0.0, 30.0);
        assert!(!landing_ring_visible(eye, aim, impact, LANDING_SEP_RAD));
    }

    /// ...but a LOBBED shot, where the arrow lands well below where the
    /// player is pointing, does get its marker.
    #[test]
    fn a_lobbed_shot_marks_where_it_lands() {
        let eye = Vec3::ZERO;
        let aim = Vec3::Z;
        // 30 m out and 2 m down: about 3.8 deg below the aim line
        let impact = Vec3::new(0.0, -2.0, 30.0);
        assert!(landing_ring_visible(eye, aim, impact, LANDING_SEP_RAD));
    }

    /// The threshold sits between those two, and is crossed in the
    /// vicinity the doc comment claims (0.6 m of drop at 25 m).
    #[test]
    fn the_separation_threshold_is_where_the_doc_says_it_is() {
        let eye = Vec3::ZERO;
        let aim = Vec3::Z;
        // 0.2 m of drop at 25 m: below threshold, still a "flat" shot
        assert!(!landing_ring_visible(
            eye,
            aim,
            Vec3::new(0.0, -0.2, 25.0),
            LANDING_SEP_RAD
        ));
        // 0.9 m of drop at 25 m: above it
        assert!(landing_ring_visible(
            eye,
            aim,
            Vec3::new(0.0, -0.9, 25.0),
            LANDING_SEP_RAD
        ));
    }

    /// Degenerate inputs must not draw a ring in the player's face.
    #[test]
    fn a_ring_is_never_drawn_on_top_of_the_muzzle() {
        assert!(!landing_ring_visible(
            Vec3::ZERO,
            Vec3::Z,
            Vec3::new(0.0, 0.1, 0.2),
            LANDING_SEP_RAD
        ));
        assert!(!landing_ring_visible(
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::new(0.0, -2.0, 30.0),
            LANDING_SEP_RAD
        ));
    }

    /// The ring lies ON the surface: +Y goes to the normal.
    #[test]
    fn the_ring_lies_flat_against_the_surface_normal() {
        let n = Vec3::new(0.3, 0.6, -0.74).normalize();
        let q = landing_ring_rot(n.to_array());
        let up = q * Vec3::Y;
        assert!(up.distance(n) < 1e-4, "ring axis {up} vs normal {n}");
        // flat ground is the identity case
        assert!((landing_ring_rot([0.0, 1.0, 0.0]) * Vec3::Y).distance(Vec3::Y) < 1e-5);
    }

    /// An exactly-opposed normal is the case `from_rotation_arc` cannot
    /// answer; it must not produce a NaN that blanks the mesh.
    #[test]
    fn an_inverted_normal_does_not_produce_a_nan_quaternion() {
        let q = landing_ring_rot([0.0, -1.0, 0.0]);
        assert!(q.is_finite());
        let up = q * Vec3::Y;
        assert!(up.distance(Vec3::NEG_Y) < 1e-5, "got {up}");
    }

    /// Growth is gentle and CLAMPED, so the ring never becomes the huge
    /// graphic §17 forbids.
    #[test]
    fn the_ring_grows_gently_and_is_capped() {
        assert!((landing_ring_scale(0.0) - 1.0).abs() < 1e-5);
        assert!(landing_ring_scale(10.0) > landing_ring_scale(5.0));
        assert_eq!(landing_ring_scale(400.0), 2.2);
        // a close ring is never SMALLER than its authored size
        assert!(landing_ring_scale(-5.0) >= 1.0);
    }
}
