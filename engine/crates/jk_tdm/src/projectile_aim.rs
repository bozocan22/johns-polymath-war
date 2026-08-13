//! THE PROJECTILE WEAPONS' AIMING LANGUAGE — subtle pre-aim zoom, a
//! round reticle, and a landing ring that never becomes a second
//! crosshair. Bow and spear both; the grenade is a later section.
//!
//! ## Why this is ONE module and not two
//!
//! It was `bow_aim.rs` when only the bow's section of the spec had been
//! built. The spear's requirements (§9-§11) are the bow's with two
//! numbers changed — 1.3x zoom, a round reticle, an arc from the real
//! launch point — so a `spear_aim.rs` would have been a near-identical
//! twin of this file with a different `GunKind` in the guards. Two
//! reticle systems painting the same centre pixel is the split brain
//! this codebase keeps paying for, so the weapon-specific parts are
//! DATA (`reticle_size`, `preaim_mag`) and the systems are shared.
//!
//! ## Why this is its own module
//!
//! Same reason `branding.rs`, `hud.rs` and `inventory_strip.rs` are:
//! `main.rs` is ~31k lines and is the most contended file in the repo,
//! with other lanes editing it concurrently. The wiring is three lines
//! nobody else's diff is likely to collide with:
//!
//! ```ignore
//! mod projectile_aim;
//! // ...
//! .add_plugins(projectile_aim::PreaimPlugin)
//! ```
//!
//! ...plus two call sites in `main.rs` that ask this module a QUESTION
//! (`preaim_fov_deg`, `draws_own_reticle`, `landing_ring_visible`)
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

/// The spear's, from §9: "use approximately 1.3x zoom". The same number
/// the bow asks for, kept as its OWN constant rather than one shared
/// `PREAIM_MAG` because the spec states it per weapon and a future
/// retune of one must not silently move the other.
pub const SPEAR_PREAIM_MAG: f32 = 1.3;

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
/// The SPEAR now takes the same pull. It was left at hip when only the
/// bow's section was built ("inventing its behaviour would be
/// guessing"); §9 has since asked for it in the owner's own words, so it
/// is no longer a guess.
pub fn preaim_fov_deg(hip_deg: f32, gun: GunKind) -> f32 {
    match preaim_mag(gun) {
        Some(m) => magnified_fov_deg(hip_deg, m),
        None => hip_deg,
    }
}

/// The pre-aim magnification a weapon asks for, or `None` if it does not
/// zoom at pre-aim at all. The single place that decides which weapons
/// are in this aiming language.
pub fn preaim_mag(gun: GunKind) -> Option<f32> {
    match gun {
        GunKind::Bow => Some(BOW_PREAIM_MAG),
        GunKind::Spear => Some(SPEAR_PREAIM_MAG),
        _ => None,
    }
}

// ---- §7: the circular reticle --------------------------------------------

/// Does this weapon draw its OWN aiming mark right now, displacing the
/// normal crosshair?
///
/// This is the same rung of the ladder as `optic_hides_crosshair` in
/// `main.rs` — "this weapon draws its own mark, so do not stack the
/// game crosshair under it" — which is why the call site ORs it into
/// `optic_hidden` rather than inventing a new `CrossFeedback` variant.
/// Hitmarkers still win over both: feedback outranks aiming furniture,
/// and that priority already existed.
///
/// At the HIP the weapon keeps the normal crosshair. The round reticle
/// is a pre-aim mark, not the weapon's permanent identity.
///
/// Bow and spear share this rung. The old name was
/// `bow_draws_own_reticle`; a second `spear_draws_own_reticle` beside it
/// would have been the same predicate twice.
pub fn draws_own_reticle(gun: GunKind, ads: bool, in_mech: bool) -> bool {
    preaim_mag(gun).is_some() && ads && !in_mech
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

/// §11: the spear's reticle must be "slightly circular", visually
/// DIFFERENT from the gun ADS reticle but in the same red language — and
/// in practice also different from the bow's, or the two weapons have
/// one identity between them.
///
/// So the spear's is an OVAL: the same ring node, taller than it is
/// wide. `BorderRadius::MAX` on a non-square `Node` gives an ellipse
/// rather than a circle, so this costs no new geometry and no second
/// entity — it is the bow's ring with one number changed. "Slightly"
/// is the operative word in the spec and 16x22 is a 1.4 ratio: read as
/// an upright oval at a glance, still unmistakably the same family as
/// the bow's circle, and nothing like the gun's cross.
///
/// Returns `(width_px, height_px)` in the 720p authoring space.
pub fn reticle_size(gun: GunKind) -> (f32, f32) {
    match gun {
        GunKind::Spear => (16.0, 22.0),
        _ => (RETICLE_D, RETICLE_D),
    }
}

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
    mut ring_q: Query<(&mut BorderColor, &mut Node), With<BowReticleRing>>,
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
        && draws_own_reticle(p.gun, cam.ads, p.in_mech());
    let a = reticle_alpha(cam.ads_t, active);
    // The ring is RESIZED per weapon rather than there being one ring
    // entity per weapon: the spear's oval and the bow's circle are the
    // same node with a different `width`/`height`. Only written while
    // the mark is actually up, so an inactive frame does not thrash the
    // UI layout every tick.
    let (rw, rh) = reticle_size(p.gun);
    for mut v in &mut root_q {
        *v = if a > 0.004 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let (r, g, b, _) = RETICLE_RGBA;
    for (mut bc, mut n) in &mut ring_q {
        bc.0 = Color::srgba(r, g, b, a);
        if active {
            n.width = Val::Px(rw);
            n.height = Val::Px(rh);
        }
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
    match angular_sep(eye, aim_dir, impact) {
        Some(sep) => sep > min_sep_rad,
        None => false,
    }
}

/// Angle in radians between `dir` and the direction from `eye` to
/// `target` — i.e. HOW FAR FROM THE CENTRE OF THE SCREEN a world point
/// lands, when `eye`/`dir` are the camera's.
///
/// `None` for the degenerate cases: a target on top of the eye, or a
/// zero direction. Both would otherwise produce a NaN that reads as
/// "very far from centre" and would draw furniture in the player's face.
///
/// Extracted so the landing ring and the dot cull below are ONE piece of
/// arithmetic asked two questions, rather than two copies of an
/// `acos(dot)` drifting apart.
pub fn angular_sep(eye: Vec3, dir: Vec3, target: Vec3) -> Option<f32> {
    let to = target - eye;
    let d = to.length();
    if d < 0.5 {
        return None;
    }
    let a = dir.normalize_or_zero();
    if a == Vec3::ZERO {
        return None;
    }
    Some((to / d).dot(a).clamp(-1.0, 1.0).acos())
}

/// The separation at which the landing ring stops being a second
/// reticle and starts being information. ~1.4 deg: at 25 m that is the
/// impact sitting about 0.6 m below the aim line, which is a drop the
/// player can see and wants marked.
pub const LANDING_SEP_RAD: f32 = 0.024;

/// Lift the ring off the surface by this much along the normal, so it
/// does not z-fight with the ground it is lying on.
pub const LANDING_LIFT_M: f32 = 0.03;

// ---- the arc's SAMPLING WINDOW ------------------------------------------

/// Where along the predicted flight the dot trail starts and stops, as
/// fractions of total arc length.
///
/// ## What the capture actually showed
///
/// `arc_preview` sampled `total * 0.62 * (i + 0.5) / n` — the NEAR 62%
/// of the flight, starting at the muzzle. The reasoning was sound and is
/// quoted in `landing_ring_visible`: don't let the dots walk out to the
/// impact point, because for a flat shot that lands on the centre pixel.
///
/// The first capture ever taken of bow pre-aim (`bow_aim`, which is also
/// the first script in this project's history to press RMB with a bow in
/// hand) shows what it produced instead: a single filled red-orange
/// blob, ~32 px, sitting exactly on the crosshair and covering the
/// target. Not a trail of dots — a blob.
///
/// The mechanism is that the trim optimised for the wrong end. Near the
/// muzzle the arc has not yet dropped, so those samples lie almost
/// exactly ON the aim line and project onto the centre pixel anyway; and
/// they are the samples CLOSEST to the camera, so they are also the
/// biggest. A dot 1 m from the eye is enormous. The trim therefore kept
/// precisely the samples that overlap into a blob at screen centre and
/// discarded precisely the samples where the drop is legible.
///
/// Diagnostic numbers from that run, printed out of `predict_arc`: a
/// level shot from eye height 1.62 m lands 14.1 m away undrawn and
/// 19.0 m away at full draw, with 6-12 predicted points over a 13-18 m
/// arc. The whole of the interesting drop is in the FAR half.
///
/// So the window moves rather than widening. It still stops short of the
/// impact — `END` is 0.92, and the landing ring marks the last 8% — so
/// the original rule ("the dots never walk out to the impact point") is
/// preserved. What changes is that the near third, which was never
/// showing the player anything except a red square over whatever they
/// were aiming at, is skipped.
pub const ARC_SPAN_START: f32 = 0.30;
pub const ARC_SPAN_END: f32 = 0.92;

/// Fraction of total arc length for dot `i` of `n`.
///
/// Kept out of the Bevy system so it can be tested at all — the 47 deg
/// camera bug in this codebase survived for months because its
/// arithmetic lived inside a system nothing could call.
pub fn arc_sample_frac(i: usize, n: usize) -> f32 {
    if n == 0 {
        return ARC_SPAN_START;
    }
    let u = (i as f32 + 0.5) / n as f32;
    ARC_SPAN_START + (ARC_SPAN_END - ARC_SPAN_START) * u
}

// ---- §10 + the SHARED near-dot fix ---------------------------------------

/// The launch speed the SPEAR preview must draw, given the weapon's base
/// speed and the sim's own charge state.
///
/// ## The trap this exists to avoid, which the bow already fell into
///
/// `arc_preview` carries the bow's version of this note: reading the
/// GunSpec's static speed drew a preview that was "completely static
/// across the draw, hiding the one thing the draw mechanic exists to
/// teach". The spear has the SAME mechanic and had the same bug — the
/// preview read `spec.projectile`'s flat 22 m/s while the sim's
/// `try_fire` launches at `v0 * fighters[i].spear_power`, which is
/// `sim::spear_charge_mult(spear_charge_t)` over a 0.90..1.30 band. A
/// 1.44x spread between a flick and a committed wind, and none of it
/// was on screen.
///
/// This does NOT reimplement the curve. It calls the sim's own
/// `spear_charge_mult`, exactly as the bow branch calls the sim's
/// `bow_power_fraction`. There is no client-side spear speed here, and
/// in particular the deleted `SPEAR_V0_MIN` branch is NOT resurrected:
/// that mirrored a hip/pre-aim halving the sim has since removed, and
/// this mirrors a charge the sim still applies.
///
/// The RUNNING-THROW bonus is included because the sim decides it at
/// throw INITIATION — i.e. at the instant the player releases — so
/// "what happens if I let go now" is the honest thing to draw. It reads
/// the same `running_momentum_t >= RUNNING_THROW_MIN_S` gate and the
/// same `RUNNING_THROW_MULT` the sim does, both `pub` in `sim`.
pub fn spear_preview_v0(base_v0: f32, charge_t: f32, momentum_t: f32) -> f32 {
    let running = if momentum_t >= crate::sim::RUNNING_THROW_MIN_S {
        crate::sim::RUNNING_THROW_MULT
    } else {
        1.0
    };
    base_v0 * running * crate::sim::spear_charge_mult(charge_t)
}

/// The distance at which an arc dot is drawn at its authored world size.
///
/// The dot mesh is a 0.09 m cube. Under a 90 deg vertical FOV at 720p a
/// world size `s` at distance `d` covers `s * 360 / d` pixels, so the
/// authored cube is 32 px at 1 m and 5 px at 6.5 m. 5 px is the size the
/// trail was always meant to be — a dotted line, not a bar.
const DOT_REF_DIST_M: f32 = 6.5;

/// How far the compensation is allowed to run at each end.
///
/// The near clamp stops a dot that is somehow ON the camera from
/// collapsing to nothing; the far clamp stops the world-space cube from
/// growing without bound down a long throw, so past ~26 m dots DO start
/// shrinking on screen again. That residual shrink is wanted: §17 asks
/// for restraint and a trail that tapers with range reads as depth.
const DOT_SCALE_MIN: f32 = 0.35;
const DOT_SCALE_MAX: f32 = 4.0;

/// World-space scale for an arc dot `dist_m` from the camera, so its
/// SCREEN size stays roughly constant along the flight.
///
/// ## Why this is the fix and the sampling window was only half of it
///
/// The bow section moved the sampling window to 30%..92% because the
/// near samples piled into a 32 px blob on the crosshair. That removed
/// the worst offenders but did not address the cause, which its own
/// closing note admitted: "dots are fixed world-size, so near ones are
/// large on screen... a distance-compensated dot scale is the fix and I
/// did NOT do it." On a level full-draw shot the nearest surviving dot
/// still sat inside the reticle circle.
///
/// Fixed world size means screen size `∝ 1/d`. Making the world size
/// `∝ d` cancels it. The two halves are complementary: the window
/// decides WHICH samples are worth drawing, this decides how big they
/// are once drawn. Neither replaces the other, and raising the window
/// back to the near field would still be wrong.
pub fn dot_screen_scale(dist_m: f32) -> f32 {
    (dist_m.max(0.0) / DOT_REF_DIST_M).clamp(DOT_SCALE_MIN, DOT_SCALE_MAX)
}

/// How far from screen centre a dot must land before it is drawn at all.
///
/// ## What the capture showed, after the scale fix above did NOT work
///
/// `dot_screen_scale` was written against the bow section's stated
/// diagnosis — "dots are fixed world-size, so near ones are large on
/// screen". A before/after of the same `bow_aim` script says that
/// diagnosis was wrong for the shot it was made on. Counting saturated
/// red pixels in the 120 px box around the crosshair:
///
/// ```text
///   frame 03 (quarter draw)  before 155   after 137
///   frame 04 (full draw)     before 102   after 124
/// ```
///
/// Unchanged, and frame 04 slightly worse. The arithmetic explains it:
/// with the window already at 30%..92%, a full-draw arc's samples sit
/// 5-17 m away, not 1 m. At 5 m the old dot was already only ~6 px, so
/// there was no bloat left to remove — and the far samples, which the
/// old code shrank to under a pixel, got BIGGER when their screen size
/// was stabilised. Net: more red, not less.
///
/// The real driver is PROJECTION, not size. On a level shot the flight
/// lies along the view axis, so two dozen dots spread over 12 m of world
/// all land within a few pixels of the centre and sum into a solid
/// square, however small each one is. No per-dot size can fix a pile.
///
/// So a dot that projects inside the reticle is CULLED. This costs the
/// player nothing: a dot on the aim line is telling them the arrow
/// starts out going where they are pointing, which they know. What they
/// cannot know is where it has dropped to, and every one of those dots
/// is outside this cone and still drawn.
///
/// It is deliberately the same question `landing_ring_visible` asks, on
/// the same helper, because it is the same rule: nothing but the reticle
/// may occupy the centre pixel. A level shot therefore shows the reticle
/// alone — which is the "crosshair is sacred" rule the trim, the removed
/// range readout and the ring gate were all reaching for.
///
/// ## Sizing
///
/// The pre-aim frame is ~90 deg over 720 authored px, so about 0.125
/// deg per px. The largest reticle here is the spear's oval at 22 px
/// tall, an 11 px half-extent — 1.375 deg, or 0.024 rad. Rounded to
/// 0.026 so a dot clears the ring's outer edge rather than kissing it.
/// It is FOV-independent by choice: at the zoomed 75 deg the reticle is
/// the same pixel size but subtends less angle, so this cone is
/// slightly generous there, which errs toward a clean centre.
pub const RETICLE_CLEAR_RAD: f32 = 0.026;

/// Should this arc dot be drawn, given where the CAMERA is looking?
///
/// Note the arguments are the camera's, not the muzzle's: the question
/// is what overlaps the reticle on screen, and the reticle is at the
/// centre of the camera's frame. The arc still LAUNCHES from
/// `sim.muzzle_origin` — this changes which of its samples are visible,
/// never where it goes.
pub fn dot_is_clear_of_reticle(cam_pos: Vec3, cam_fwd: Vec3, dot: Vec3) -> bool {
    match angular_sep(cam_pos, cam_fwd, dot) {
        Some(sep) => sep > RETICLE_CLEAR_RAD,
        // Degenerate (a dot on the lens) is exactly the case that would
        // paint the centre pixel, so it is hidden, not shown.
        None => false,
    }
}

pub struct PreaimPlugin;

impl Plugin for PreaimPlugin {
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

    /// §9: the spear takes the same 1.3x pull the bow does, and it is
    /// the PROJECTILE weapons only - a rifle's pre-aim is not this.
    ///
    /// FAILS on the pre-change code, where `preaim_fov_deg` returned the
    /// hip FOV for everything that was not a bow.
    #[test]
    fn the_spear_zooms_like_the_bow_and_the_guns_do_not() {
        let hip = 90.0;
        let spear = preaim_fov_deg(hip, GunKind::Spear);
        assert!(spear < hip, "the spear must actually zoom, got {spear}");
        // approximately 1.3x, by the same tangent relation - derived
        // longhand rather than read off the function under test
        let want = 2.0 * (1.0_f32 / 1.3).atan().to_degrees();
        assert!((spear - want).abs() < 1e-3, "spear pre-aim FOV {spear}");
        // and subtle, by the same band the bow is held to
        assert!(hip - spear < 20.0 && hip - spear > 5.0);
        // guns are untouched
        assert_eq!(preaim_fov_deg(hip, GunKind::M4), hip);
        assert_eq!(preaim_fov_deg(hip, GunKind::Glock), hip);
        assert!(preaim_mag(GunKind::M4).is_none());
        assert!(preaim_mag(GunKind::Spear).is_some());
    }

    /// §11: the spear's reticle is "slightly circular" and must not be
    /// the bow's circle wearing the same size - the two weapons need
    /// distinguishable marks. It is an OVAL, and "slightly" bounds how
    /// far from round it is allowed to get.
    ///
    /// FAILS on the pre-change code, which had one hard-coded 14x14.
    #[test]
    fn the_spear_reticle_is_an_oval_and_the_bow_reticle_is_a_circle() {
        let (bw, bh) = reticle_size(GunKind::Bow);
        assert_eq!(bw, bh, "the bow's mark is a circle");
        let (sw, sh) = reticle_size(GunKind::Spear);
        assert!(sh > sw, "the spear's mark is upright: {sw}x{sh}");
        // "SLIGHTLY circular" - an oval, not a slot. Anything past 2:1
        // stops reading as the same family as the bow's circle.
        let ratio = sh / sw;
        assert!(
            (1.15..2.0).contains(&ratio),
            "aspect {ratio} is either indistinguishable from a circle or no longer one"
        );
        // and distinguishable from the bow at a glance, without being
        // the "giant" graphic §17 forbids
        assert!(sh != bh, "the two marks are the same height");
        assert!(sh <= 28.0, "reticle {sh} px tall is getting big");
    }

    /// The reticle is a PRE-AIM mark. At the hip the bow keeps the
    /// normal crosshair, and a pilot in a mech is firing hull mounts.
    #[test]
    fn reticle_replaces_the_crosshair_only_while_pre_aiming_a_bow() {
        assert!(draws_own_reticle(GunKind::Bow, true, false));
        assert!(!draws_own_reticle(GunKind::Bow, false, false));
        assert!(!draws_own_reticle(GunKind::Bow, true, true));
        assert!(!draws_own_reticle(GunKind::M4, true, false));
        // and the spear joined it, under exactly the same conditions -
        // at the HIP a javelin keeps the normal crosshair.
        assert!(draws_own_reticle(GunKind::Spear, true, false));
        assert!(!draws_own_reticle(GunKind::Spear, false, false));
        assert!(!draws_own_reticle(GunKind::Spear, true, true));
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
        // Squared, so the first half of the raise stays out of the way.
        //
        // This assertion was `mid < full * 0.5 + 1e-4` and a mutation
        // run caught it: replacing `t * t` with `t` puts `mid` at
        // EXACTLY `full * 0.5`, and the epsilon then let the mutant
        // through. A tolerance added on the wrong side of a boundary
        // makes the test vacuous at precisely the value it is meant to
        // exclude, which is rule 12's failure mode in miniature. The
        // margin now cuts the other way: at t = 0.5 squaring gives a
        // quarter of full, so demand comfortably less than a third.
        assert!(
            mid < full / 3.0,
            "mid {mid} is not well below a third of full {full} - is the fade still squared?"
        );
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

    /// The dot trail must not start AT the muzzle. Those samples are a
    /// metre from the eye, have not dropped yet, and project onto the
    /// centre pixel as a blob over the target - which is what the first
    /// pre-aim capture photographed.
    #[test]
    fn the_dot_trail_skips_the_near_field() {
        let first = arc_sample_frac(0, 24);
        assert!(
            first >= ARC_SPAN_START,
            "first dot at {first} is inside the near field"
        );
        // and it is a real skip, not a rounding of zero
        assert!(first > 0.25);
    }

    /// ...and it still stops SHORT of the impact, which is the rule the
    /// original 0.62 trim existed to enforce. The landing ring covers
    /// the last stretch, not more dots.
    #[test]
    fn the_dot_trail_still_stops_short_of_the_impact() {
        let last = arc_sample_frac(23, 24);
        assert!(last < 1.0, "last dot at {last} reaches the impact point");
        assert!(last <= ARC_SPAN_END);
    }

    /// Monotonic and inside the window for every index - a dot that goes
    /// backwards along the flight would read as noise.
    #[test]
    fn the_dot_trail_walks_forward_across_the_window() {
        let n = 24;
        let mut prev = -1.0;
        for i in 0..n {
            let f = arc_sample_frac(i, n);
            assert!(f > prev, "dot {i} went backwards: {f} after {prev}");
            assert!((ARC_SPAN_START..=ARC_SPAN_END).contains(&f), "dot {i} at {f}");
            prev = f;
        }
        assert_eq!(arc_sample_frac(0, 0), ARC_SPAN_START);
    }

    /// §10: the preview must TRACK the wind. This is the bug the bow
    /// had - a static speed hides the one thing the charge mechanic
    /// exists to teach - and the spear had it too.
    ///
    /// FAILS on the pre-change code, where `arc_preview` read the flat
    /// `spec.projectile` speed for the spear, so the two ends of the
    /// charge drew the SAME arc.
    #[test]
    fn the_spear_preview_speed_follows_the_wind() {
        let base = 22.0;
        let flick = spear_preview_v0(base, crate::sim::SPEAR_CHARGE_MIN_S, 0.0);
        let wound = spear_preview_v0(base, crate::sim::SPEAR_CHARGE_FULL_S, 0.0);
        assert!(
            wound > flick * 1.25,
            "a wound throw must preview visibly faster: {flick} vs {wound}"
        );
        // The band is the SIM's (0.90..1.30), not one invented here.
        assert!((flick - base * crate::sim::SPEAR_CHARGE_V0_MIN).abs() < 1e-4);
        assert!((wound - base * crate::sim::SPEAR_CHARGE_V0_MAX).abs() < 1e-4);
        // Holding past full stops paying, so the arc stops growing -
        // otherwise the preview promises what the throw will not pay.
        let overheld = spear_preview_v0(base, crate::sim::SPEAR_CHARGE_FULL_S * 3.0, 0.0);
        assert!((overheld - wound).abs() < 1e-4, "the arc kept growing past full");
    }

    /// ...and the RUNNING-THROW bonus is in the preview too, on the
    /// sim's own gate. A player sprinting into a throw sees the longer
    /// arc they are about to get.
    #[test]
    fn a_running_throw_previews_the_bonus_it_will_actually_get() {
        let base = 22.0;
        let charge = crate::sim::SPEAR_CHARGE_FULL_S;
        let standing = spear_preview_v0(base, charge, 0.0);
        let running = spear_preview_v0(base, charge, crate::sim::RUNNING_THROW_MIN_S);
        assert!(
            (running - standing * crate::sim::RUNNING_THROW_MULT).abs() < 1e-3,
            "running {running} must be standing {standing} x the sim's multiplier"
        );
        // and the gate is a THRESHOLD, not a ramp - just under it pays
        // nothing, which is the sim's rule and must not be softened
        // into a lerp on the presentation side.
        let nearly = spear_preview_v0(base, charge, crate::sim::RUNNING_THROW_MIN_S - 0.01);
        assert!((nearly - standing).abs() < 1e-4, "the momentum gate leaked");
    }

    /// THE SHARED NEAR-DOT FIX. A dot's SCREEN size is its world size
    /// over its distance, so a screen-stable dot needs a world size
    /// proportional to distance.
    ///
    /// FAILS on the pre-change code, which had no such function: the
    /// scale was `1.0 - 0.55 * frac` with no distance term at all, so
    /// this test's central ratio was 1.0 at every range.
    #[test]
    fn near_dots_are_screen_stable_instead_of_bloating() {
        // Across the working band the arc actually occupies, screen
        // size is CONSTANT: size/dist is flat. Computed from the
        // definition of perspective, not from the function's output.
        let screen_px = |d: f32| dot_screen_scale(d) * 0.09 * 360.0 / d;
        let near = screen_px(3.0);
        let far = screen_px(20.0);
        assert!(
            (near - far).abs() < 0.05 * near,
            "screen size drifted across the band: {near} px at 3 m vs {far} px at 20 m"
        );
        // and it is around the 5 px the doc claims, not a bar
        assert!((4.0..6.5).contains(&near), "dot is {near} px on screen");

        // THE ACTUAL BUG: a dot 1.2 m from the eye used to be ~27 px -
        // wider than the 14 px reticle it sat inside. Now it is small.
        let very_near = screen_px(1.2);
        assert!(
            very_near < RETICLE_D,
            "a 1.2 m dot is {very_near} px, still bigger than the {RETICLE_D} px reticle"
        );

        // Monotone, so the trail never jumps in size mid-flight.
        let mut prev = 0.0;
        let mut d = 0.5;
        while d < 60.0 {
            let s = dot_screen_scale(d);
            assert!(s >= prev - 1e-6, "scale went down at {d} m");
            prev = s;
            d += 0.25;
        }
        // Clamped at both ends: no zero-size dot, no unbounded cube.
        assert_eq!(dot_screen_scale(0.0), DOT_SCALE_MIN);
        assert_eq!(dot_screen_scale(1000.0), DOT_SCALE_MAX);
    }

    /// THE ACTUAL NEAR-DOT FIX. A dot that projects onto the reticle is
    /// not drawn, so nothing but the reticle occupies the centre.
    ///
    /// FAILS on the pre-change code, which had no cull at all: every
    /// sample in the window was `Visibility::Visible` unconditionally,
    /// which is what put a solid red square inside the ring in
    /// `bow_aim/04`.
    #[test]
    fn a_dot_on_the_aim_line_is_not_drawn_over_the_reticle() {
        let cam = Vec3::ZERO;
        let fwd = Vec3::Z;
        // dead on the aim line at 10 m: this is the pile-up
        assert!(!dot_is_clear_of_reticle(cam, fwd, Vec3::new(0.0, 0.0, 10.0)));
        // 1 cm off at 10 m - still inside the ring, still culled
        assert!(!dot_is_clear_of_reticle(cam, fwd, Vec3::new(0.01, 0.0, 10.0)));
        // ...but a dot that has DROPPED half a metre by 10 m is 2.9 deg
        // off centre, well outside the ring, and is the whole point of
        // drawing an arc at all
        assert!(dot_is_clear_of_reticle(cam, fwd, Vec3::new(0.0, -0.5, 10.0)));
        // a dot behind the camera is not "far from centre" and drawn
        assert!(dot_is_clear_of_reticle(cam, fwd, Vec3::new(0.0, 0.0, -10.0)));
        // degenerate: a dot on the lens must never paint the centre
        assert!(!dot_is_clear_of_reticle(cam, fwd, Vec3::new(0.0, 0.0, 0.1)));
    }

    /// The cull cone must actually cover the reticle it is protecting,
    /// and not much more - a cone far wider than the mark would eat the
    /// informative near-drop as well as the pile-up.
    #[test]
    fn the_cull_cone_is_the_size_of_the_reticle_it_protects() {
        // 90 deg over 720 authored px = 0.125 deg/px. The spear's oval
        // is the tallest mark at 22 px, so an 11 px half-extent.
        let deg_per_px = 90.0 / 720.0;
        let (_, tallest) = reticle_size(GunKind::Spear);
        let half_extent_rad = (tallest * 0.5 * deg_per_px).to_radians();
        assert!(
            RETICLE_CLEAR_RAD >= half_extent_rad,
            "the cone {RETICLE_CLEAR_RAD} is narrower than the {half_extent_rad} rad reticle, \
             so dots can still land inside the ring"
        );
        assert!(
            RETICLE_CLEAR_RAD < half_extent_rad * 2.0,
            "the cone is more than twice the mark - it will swallow real drop"
        );
    }

    /// The two gates are ONE piece of arithmetic. If they ever disagree
    /// about what "at the centre of the screen" means, a level shot gets
    /// a landing ring under a culled dot trail or vice versa.
    #[test]
    fn the_ring_gate_and_the_dot_cull_share_their_geometry() {
        let eye = Vec3::ZERO;
        let aim = Vec3::Z;
        let target = Vec3::new(0.0, -2.0, 30.0);
        let sep = angular_sep(eye, aim, target).expect("a real separation");
        // ~3.8 deg, computed independently of the helper
        let want = (2.0_f32 / 30.0).atan();
        assert!((sep - want).abs() < 1e-4, "sep {sep} vs {want}");
        assert_eq!(landing_ring_visible(eye, aim, target, sep - 1e-4), true);
        assert_eq!(landing_ring_visible(eye, aim, target, sep + 1e-4), false);
        assert!(angular_sep(eye, aim, Vec3::new(0.0, 0.0, 0.2)).is_none());
        assert!(angular_sep(eye, Vec3::ZERO, target).is_none());
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
