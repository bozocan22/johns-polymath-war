//! THE ART PASS, per map: the air a map sits in, and what its stone is.
//!
//! ## Where this came from, and why it outlived the map it was written for
//!
//! This is the surviving half of `cliffhold.rs`, which dressed a 600 x 600 m
//! mountain map the owner has since asked to be removed: *"remove that last
//! huge map you created just put more effort in last 3 maps"*. The map is
//! gone. The findings are not, because they were never about that map.
//!
//! From `research/vertical-maps/SOURCES.md`, VM-01 (Jim Brown, Epic, GDC
//! 2014): Gears of War's *Gridlock* went from the studio's most-played map
//! to the bottom of the popularity chart after a re-release that changed
//! **the art only** — "we hadn't changed the physical play space the weapon
//! load out or even the collision" — and returned to the top three when it
//! was rebuilt clean. Corroborated by VM-02 (Hi-Rez), whose map *Corey*
//! tested well as grey-box and went lukewarm after its art pass.
//!
//! The two named failure modes are symmetric: too much competing detail
//! ("there's nowhere for your eye to stop") and too little variation
//! ("there's nowhere for your brain to focus"). **A monochrome map and a
//! cluttered map fail identically.** The three core maps are about to get
//! height, high ground and randomised structures, and one flat grey for
//! every stone box on them is the second failure exactly — so the work
//! below is waiting for them rather than being deleted with the map that
//! paid for it.
//!
//! ## What is kept, and what changed to keep it
//!
//! * **The air** (`MapLook`, `look`, `sun_dir`, `sun_euler`) — live, and
//!   always was for every map. `look` no longer names a fifth map and now
//!   has a DEFAULT arm, deliberately, so that removing a `MapKind` variant
//!   from sim.rs cannot break this file's exhaustiveness in the window
//!   between the two edits landing.
//!
//! * **`sun_euler`** — kept whole, comment and all. It encodes three
//!   capture cycles' worth of a bug that had nothing to do with terrain:
//!   an azimuth past a quarter turn silently inverts the vertical and puts
//!   the sun below the horizon, which does not look like a broken light,
//!   it looks like a palette problem. Any map that ever moves its sun can
//!   pay for that discovery again, or call this.
//!
//! * **`uv_tiles`** — live for every map now. A `Cuboid`'s UVs run 0..1 on
//!   every face, so the shared fixed 1.25 scale meant the tile size was
//!   whatever the box happened to be: one course of blockwork across a
//!   whole long wall, and the same single course across a 2.4 m crate.
//!
//! * **`Stone` / `stone_of`** — the value-banding technique, **re-expressed
//!   against the map's own tallest stone** instead of the five absolute
//!   Cliffhold heights it used to read. That is the change that makes it
//!   portable: a box is classified by how big it is and how high it stands
//!   RELATIVE to the map it is on, so it works on a 100 m arena and on
//!   whatever the core three become.
//!
//! * The landmark-silhouette half — `find`, `spawn_landmarks`, the keep
//!   crown, the gatehouse arch, the bell tower, the cliff crest — is
//!   **gone**, and honestly so: every one of its rules keyed off
//!   `CH_KEEP_TOP`, `CH_RAMPART`, `CH_SHELF` and friends, which are being
//!   deleted from sim.rs with the map. What was worth keeping from it is
//!   the discipline, recorded here rather than pretended at: **every anchor
//!   was FOUND in the sim's published cover list by a geometric rule, never
//!   restated as a coordinate**, so a test failed loudly if the map builder
//!   moved a landmark instead of the client quietly dressing empty air.
//!   Anything that dresses the enlarged maps should be built the same way.
//!
//! ## What this module may not do
//!
//! Everything here is cosmetic. It reads AABBs the sim publishes and
//! returns colours and tile counts. No cover box is created, moved or
//! resized, nothing stops a bullet, and nothing touches the sim's RNG.

use bevy::prelude::*;

use crate::sim::{Aabb, MapKind};

// ---------------------------------------------------------------- the air

/// Everything about a map that is not a cover block: sky, ground, border,
/// the fog band and the sun.
///
/// This used to be a three-tuple built inline in `rebuild_world`, which is
/// why fog distance and sun angle were map-INDEPENDENT: there was nowhere
/// to put them.
pub struct MapLook {
    pub sky: Color,
    pub ground: Color,
    pub border: Color,
    /// Linear fog `(start, end)` in metres.
    pub fog: (f32, f32),
    /// Sun rotation as `(pitch, yaw)` euler radians — pitch is NEGATIVE
    /// downward, so a small magnitude is a low sun.
    pub sun: (f32, f32),
    pub sun_color: Color,
    pub sun_lux: f32,
    pub ambient: Color,
    pub ambient_lux: f32,
    /// Shadow bias, and the far bound of the cascade set.
    ///
    /// Bevy's default stretches four cascades over a kilometre. On a 100 m
    /// map that is most of the shadow budget spent past anything visible;
    /// the field is here so a bigger map can say so.
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
    pub shadow_max_m: f32,
    pub shadow_first_m: f32,
}

/// Where a `DirectionalLight` actually points, given the euler pair
/// `MapLook::sun` hands to `Quat::from_euler(EulerRot::XYZ, x, y, 0.0)`.
///
/// `EulerRot::XYZ` is INTRINSIC, so the composed rotation is
/// `Rx(x) * Ry(y)` and the light travels along that times `-Z`:
///
/// ```text
///   dir = ( -sin y ,  cos y · sin x ,  -cos y · cos x )
/// ```
///
/// The middle component is the one that matters and it is the one nobody
/// can see in the number: **`cos y` gates the sign of the vertical**. Any
/// azimuth past a quarter turn flips `cos y` negative, and then a pitch
/// that reads as "aim down 35 degrees" aims the sun UP THROUGH THE FLOOR.
pub fn sun_dir(euler: (f32, f32)) -> Vec3 {
    let (x, y) = euler;
    Vec3::new(-y.sin(), y.cos() * x.sin(), -y.cos() * x.cos())
}

/// The euler pair for a sun at `elev` above the horizon and `azim` east of
/// due south (both radians), which is how a person thinks about a sun.
///
/// ## Why this exists
///
/// It was written after the bug it prevents. A map whose viewing axis
/// points north needs its sun in the SOUTH or every surface a player looks
/// at is a shaded one. Moving it there by hand meant taking the y euler
/// past a quarter turn — and that silently inverted the vertical, so the
/// "sun" spent two capture passes twenty degrees BELOW the horizon,
/// shining upward.
///
/// It did not look like a broken light. It looked like a palette problem:
/// south and east faces brightly lit, every horizontal surface on the map
/// — the ground, every roof — dark. The first diagnosis was shadow acne
/// and the first fix was a bias tweak and a `NotShadowCaster`, neither of
/// which did anything, because nothing was in shadow. Three capture cycles.
///
/// `a PI yaw on a parent silently flips a child's forward axis` is in the
/// standing brief. This is that, in a light rig.
pub fn sun_euler(elev: f32, azim: f32) -> (f32, f32) {
    let s = azim.sin() * elev.cos();
    // the NEGATIVE root: it is what keeps `cos y` on the side that leaves
    // the vertical alone
    let cb = -(1.0 - s * s).max(0.0).sqrt();
    (elev.sin().atan2(azim.cos() * elev.cos()), s.atan2(cb))
}

/// The look of every map, in one place.
///
/// Every map keeps exactly the numbers it shipped with — same sky, same
/// ground, same border, the 45→130 m fog and the sun that was hard-coded
/// at startup.
///
/// **The `_` arm is deliberate and is not laziness.** An exhaustive match
/// on `MapKind` in this file would stop compiling the instant sim.rs
/// removes a variant, and would stop compiling in the OTHER order too if
/// this file dropped an arm first. A default means the two edits can land
/// in either order, and a new map added to sim.rs renders in the shared
/// palette instead of failing the build — which is the right failure for a
/// cosmetic file to have.
pub fn look(map: MapKind) -> MapLook {
    let base = |sky: Color, ground: Color, border: Color| MapLook {
        sky,
        ground,
        border,
        fog: (45.0, 130.0),
        sun: (-0.9, 0.5),
        sun_color: Color::WHITE,
        sun_lux: 13_000.0,
        ambient: Color::srgb(0.85, 0.82, 0.78),
        ambient_lux: 380.0,
        // Bevy's own defaults, restated so the maps are pinned to what
        // they shipped with rather than to whatever the engine default
        // becomes.
        shadow_depth_bias: 0.02,
        shadow_normal_bias: 1.8,
        shadow_max_m: 1000.0,
        shadow_first_m: 5.0,
    };
    match map {
        MapKind::Bailey => base(
            Color::srgb(0.52, 0.66, 0.60),
            Color::srgb(0.30, 0.42, 0.24),
            Color::srgb(0.52, 0.52, 0.50),
        ),
        MapKind::Gardens => base(
            Color::srgb(0.55, 0.71, 0.55),
            Color::srgb(0.27, 0.48, 0.24),
            Color::srgb(0.58, 0.56, 0.50),
        ),
        MapKind::Battlefield => base(
            Color::srgb(0.60, 0.62, 0.68),
            Color::srgb(0.36, 0.40, 0.28),
            Color::srgb(0.50, 0.50, 0.48),
        ),
        // Arena, and anything added later.
        _ => base(
            Color::srgb(0.58, 0.63, 0.72),
            Color::srgb(0.45, 0.40, 0.33),
            Color::srgb(0.55, 0.52, 0.47),
        ),
    }
}

// -------------------------------------------------------------- the stone

/// What a stone box IS, decided from its own published AABB and the map's
/// own tallest stone, and nothing else.
///
/// This is a CLASSIFICATION of data the sim already hands over, not a
/// second copy of the layout: **there is no coordinate in this file.** If
/// the map builder moves a wall, the wall keeps its tint, because the tint
/// is a function of how tall and how big each box is rather than of where
/// it was put.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stone {
    /// living rock, low band — aprons and talus benches
    RockLow,
    /// living rock, middle band
    RockMid,
    /// living rock, the headline band — plateaus and great cliffs
    RockTop,
    /// street level: field walls, rubble stubs, spoil, hedgebanks
    Street,
    /// the low built course
    RoofLow,
    /// the middle course
    RoofMid,
    /// the tall course — a map's own skyline
    RoofHigh,
    /// curtain walls, turrets, gatehouses
    Rampart,
    /// the highest built thing on the map: the palest masonry on it
    Keep,
}

/// Footprint area, in m², above which a stone box is LIVING ROCK rather
/// than something anybody built.
///
/// A round number rather than a fitted one. On the map this was measured
/// against, the natural slabs ran 3,000–18,090 m² and the largest built
/// thing was an aqueduct deck at 1,110 — better than a 1.8x margin either
/// side. Nothing on the three core maps comes close to it today, which is
/// correct: none of them has living rock on it yet.
pub const MASSIF_AREA_M2: f32 = 2_000.0;

/// Horizontal footprint area of a box.
fn area(a: &Aabb) -> f32 {
    (a.max[0] - a.min[0]) * (a.max[2] - a.min[2])
}

/// The tallest stone top on a map — the ruler `stone_of` bands against.
///
/// Taken from the sim's published cover rather than from a constant, so
/// this cannot go stale when a map grows. Falls back to a sane positive
/// number for a map with no stone at all, because dividing by the height
/// of nothing is how a classifier starts returning `Keep` for kerbstones.
pub fn tallest_stone(cover: &[Aabb], kind: &[crate::sim::CoverKind]) -> f32 {
    cover
        .iter()
        .zip(kind.iter())
        .filter(|(_, k)| **k == crate::sim::CoverKind::Stone)
        .map(|(a, _)| a.max[1])
        .fold(0.0_f32, f32::max)
        .max(2.0)
}

/// Classify one stone box. Pure, and the only place the bands are stated.
///
/// `tallest` is the map's own tallest stone (see `tallest_stone`). Banding
/// against it rather than against absolute metres is what let this survive
/// the map it was written for: the same nine tints describe a 100 m arena
/// whose tallest wall is 3 m and a mountain whose keep is at 32.
pub fn stone_of(a: &Aabb, tallest: f32) -> Stone {
    let t = a.max[1] / tallest.max(0.01);
    if area(a) >= MASSIF_AREA_M2 {
        // living rock: three bands, split at thirds of the map's height
        return if t >= 0.67 {
            Stone::RockTop
        } else if t >= 0.34 {
            Stone::RockMid
        } else {
            Stone::RockLow
        };
    }
    // masonry, by altitude band. The steps are uneven on purpose: most of
    // a map's boxes are chest-high cover, so the low end gets the finer
    // divisions and the top two bands are reserved for the few things that
    // are meant to be landmarks.
    if t >= 0.86 {
        Stone::Keep
    } else if t >= 0.64 {
        Stone::Rampart
    } else if t >= 0.44 {
        Stone::RoofHigh
    } else if t >= 0.28 {
        Stone::RoofMid
    } else if t >= 0.14 {
        Stone::RoofLow
    } else {
        Stone::Street
    }
}

impl Stone {
    /// Living rock takes the broad mottled generator; masonry takes the
    /// coursed blockwork. This one bit is most of why a cliff stops
    /// reading as a wall.
    pub fn is_rock(self) -> bool {
        matches!(self, Stone::RockLow | Stone::RockMid | Stone::RockTop)
    }

    /// Base colour.
    ///
    /// Two ramps, deliberately in opposite directions:
    ///
    /// * ROCK gets COOLER and DARKER as it rises, so the tallest mass on
    ///   the map is the darkest and everything standing on it has
    ///   something to be seen against.
    /// * MASONRY gets LIGHTER as it rises — dressed ashlar on raw rock,
    ///   which is both the real thing and the readable one. VM-01's own
    ///   counter-example works this way: *Facing Worlds*' towers "anchor
    ///   the eye" because the players on them "can be picked out against
    ///   their dark backgrounds".
    ///
    /// The whole set is low-chroma on purpose. VM-01's other failure mode
    /// is a scene with nowhere for the eye to stop; the world stays quiet
    /// so a fighter in team colours is the loudest thing in it.
    pub fn color(self) -> Color {
        match self {
            Stone::RockLow => Color::srgb(0.50, 0.47, 0.41),
            Stone::RockMid => Color::srgb(0.40, 0.40, 0.39),
            Stone::RockTop => Color::srgb(0.30, 0.31, 0.33),
            Stone::Street => Color::srgb(0.50, 0.46, 0.39),
            Stone::RoofLow => Color::srgb(0.57, 0.50, 0.40),
            Stone::RoofMid => Color::srgb(0.62, 0.54, 0.43),
            Stone::RoofHigh => Color::srgb(0.67, 0.58, 0.46),
            // Pulled down from 0.71 / 0.80 after a capture: at those
            // values a sunlit wall is near enough white to blow out
            // against the sky and reads as painted plaster rather than as
            // cut stone. It still has to be the palest masonry, and is.
            Stone::Rampart => Color::srgb(0.66, 0.64, 0.59),
            Stone::Keep => Color::srgb(0.74, 0.72, 0.66),
        }
    }

    /// Rock is uniformly rough; dressed stone gets smoother the better it
    /// was cut, so a castle catches a highlight a mountain does not.
    pub fn roughness(self) -> f32 {
        match self {
            Stone::RockLow | Stone::RockMid | Stone::RockTop => 1.0,
            Stone::Street => 0.97,
            Stone::RoofLow | Stone::RoofMid | Stone::RoofHigh => 0.90,
            Stone::Rampart => 0.80,
            Stone::Keep => 0.72,
        }
    }
}

/// How many texture tiles to run across a box, from the box's own size.
///
/// A `Cuboid`'s UVs run 0..1 on every face, so ONE fixed scale means the
/// tile size is whatever the box happens to be — the shared 1.25 puts a
/// single course of blockwork across a whole long wall and the same course
/// across a 2.4 m crate. Scaling by the largest dimension keeps a tile
/// near four metres on the long axis of anything.
///
/// Clamped at 40 because past that the generated 128 px texture is
/// aliasing rather than describing a surface.
pub fn uv_tiles(a: &Aabb) -> f32 {
    let m = (a.max[0] - a.min[0])
        .max(a.max[1] - a.min[1])
        .max(a.max[2] - a.min[2]);
    (m / 4.0).clamp(1.0, 40.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::CoverKind;

    /// Relative luminance, because the ramp is a VALUE ramp and the red
    /// channel is not it: `Rampart` is a cooler, brighter grey than
    /// `RoofHigh` and has LESS red in it while being lighter overall. The
    /// first version of these tests used red as a proxy and failed on
    /// exactly that pair, which is the proxy being wrong rather than the
    /// palette.
    fn lum(s: Stone) -> f32 {
        let c = s.color().to_srgba();
        0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue
    }

    fn boxx(w: f32, h: f32, d: f32) -> Aabb {
        Aabb {
            min: [0.0, 0.0, 0.0],
            max: [w, h, d],
        }
    }

    /// THE bug this module exists to have caught: a sun below the horizon
    /// on any map lights every vertical face and leaves every horizontal
    /// one black, and reads as a palette problem rather than as a light.
    #[test]
    fn the_sun_is_above_the_horizon_on_every_map() {
        for m in MapKind::ALL {
            let d = sun_dir(look(m).sun);
            assert!(
                d.y < -0.05,
                "{m:?}: the sun travels {d:?} - a non-negative y is a light \
                 shining UP through the floor"
            );
        }
    }

    /// `sun_euler` must put the sun where a person asked for it, including
    /// in the southern half where the naive euler flips the vertical.
    #[test]
    fn sun_euler_round_trips_through_the_quarter_turn() {
        for azim in [-1.2_f32, -0.5, 0.0, 0.55, 1.2] {
            for elev in [0.35_f32, 0.62, 1.0] {
                let d = sun_dir(sun_euler(elev, azim));
                assert!(
                    d.y < 0.0,
                    "elev {elev} azim {azim}: sun points up ({})",
                    d.y
                );
                // travelling downward by `elev` means the vertical
                // component is -sin(elev)
                assert!(
                    (d.y + elev.sin()).abs() < 1e-4,
                    "elev {elev} azim {azim}: wanted y {}, got {}",
                    -elev.sin(),
                    d.y
                );
            }
        }
    }

    /// The salvage that had to change to survive: banding is RELATIVE, so
    /// the same nine tints describe a 3 m arena wall and a 32 m keep.
    ///
    /// Fails on the pre-change code, which read five absolute Cliffhold
    /// heights — on a map whose tallest stone is 3 m, every box on it
    /// classified as `Street` and the whole ramp collapsed to one tone.
    #[test]
    fn the_bands_scale_to_whatever_map_they_are_given() {
        for tallest in [3.0_f32, 12.0, 32.0] {
            let at = |frac: f32| stone_of(&boxx(2.0, tallest * frac, 2.0), tallest);
            assert_eq!(at(0.05), Stone::Street, "tallest {tallest}");
            assert_eq!(at(0.95), Stone::Keep, "tallest {tallest}");
            // and the ramp is monotone in brightness from bottom to top
            let mut prev = -1.0_f32;
            for k in 1..=9 {
                let s = at(k as f32 / 10.0);
                let l = lum(s);
                assert!(
                    l >= prev - 1e-6,
                    "masonry got darker as it rose at {tallest} m: {s:?}"
                );
                prev = l;
            }
        }
    }

    /// Living rock is chosen by FOOTPRINT, not by height — a tall thin
    /// tower is masonry however high it stands, and a wide low slab is
    /// rock however low.
    #[test]
    fn rock_is_told_from_masonry_by_how_much_ground_it_covers() {
        let tall_thin = boxx(4.0, 30.0, 4.0);
        assert!(!stone_of(&tall_thin, 32.0).is_rock());
        let wide_low = boxx(80.0, 6.0, 80.0);
        assert!(stone_of(&wide_low, 32.0).is_rock());
        // and rock darkens as it rises, the opposite of masonry
        let low = stone_of(&boxx(80.0, 3.0, 80.0), 32.0);
        let high = stone_of(&boxx(80.0, 30.0, 80.0), 32.0);
        assert!(
            lum(high) < lum(low),
            "rock must get darker with altitude, not lighter"
        );
    }

    /// The tile count must follow the box, or one course of blockwork
    /// stretches across a whole wall.
    #[test]
    fn a_long_wall_gets_more_courses_than_a_crate() {
        let crate_box = boxx(2.4, 1.2, 2.4);
        let wall = boxx(40.0, 4.0, 1.0);
        assert!(
            uv_tiles(&wall) > uv_tiles(&crate_box) * 4.0,
            "a 40 m wall tiles {} against a crate's {}",
            uv_tiles(&wall),
            uv_tiles(&crate_box)
        );
        assert!(uv_tiles(&boxx(400.0, 4.0, 4.0)) <= 40.0, "the clamp holds");
        assert!(uv_tiles(&boxx(0.2, 0.2, 0.2)) >= 1.0, "and the floor holds");
    }

    /// The ruler must come from the sim's own cover, and must never be
    /// zero — a zero divisor makes every kerbstone a `Keep`.
    #[test]
    fn the_ruler_is_read_off_the_map_and_is_never_zero() {
        let cover = vec![boxx(2.0, 1.2, 2.0), boxx(3.0, 7.5, 3.0), boxx(2.0, 9.9, 2.0)];
        let kind = vec![CoverKind::Stone, CoverKind::Stone, CoverKind::Crate];
        // the 9.9 is a CRATE and must not set the stone ruler
        assert!((tallest_stone(&cover, &kind) - 7.5).abs() < 1e-6);
        assert!(tallest_stone(&[], &[]) >= 2.0, "an empty map still has a ruler");
    }
}
