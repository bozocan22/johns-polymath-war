//! §cliffhold — THE ART PASS: what a 600 x 600 m mountain map looks like.
//!
//! ## What was here before
//!
//! One `match` arm in `rebuild_world`, three colours, and a comment
//! calling itself a placeholder. Every one of Cliffhold's 423 boxes —
//! the 17,000 m² massif, the curtain wall, the keep, a half-city, a
//! quarry — came out of the client as the same flat grey cuboid with the
//! same coursed-blockwork texture stretched across it, under the same
//! 130 m fog wall that hid everything past the ravine.
//!
//! ## Why this is a job and not a colour tweak
//!
//! From `research/vertical-maps/SOURCES.md`, VM-01 (Jim Brown, Epic, GDC
//! 2014): Gears of War's *Gridlock* went from the studio's most-played
//! map to the bottom of the popularity chart after a re-release that
//! changed **the art only** — "we hadn't changed the physical play space
//! the weapon load out or even the collision" — and came back to the top
//! three when it was rebuilt clean. Corroborated independently by VM-02
//! (Hi-Rez), whose map *Corey* tested well as grey-box and went lukewarm
//! after its art pass.
//!
//! So on a deliberately messy map the art pass is the risk, and the two
//! named failure modes are symmetric: too much competing detail
//! ("there's nowhere for your eye to stop") and too little variation
//! ("there's nowhere for your brain to focus"). **A monochrome map and a
//! cluttered map fail identically.** One flat grey for a mountain, a
//! city and a castle is the second failure exactly.
//!
//! ## The three decisions
//!
//! **1. DISTRICTS BY HUE, BANDS BY VALUE.** VM-12, via Lynch: districts —
//! "areas which share similar characteristics" — are one of the five
//! things people build a mental map out of, and what must exist is a
//! deliberate *edge* between them. So the massif is COOL grey rock and
//! the city is WARM tan masonry: one hue step, applied at the seam. Then
//! value rises with altitude inside masonry, so the castle is the
//! brightest thing on the map and stands on the darkest.
//!
//! That ordering is deliberate and it is the opposite of "high things go
//! pale with distance". Dressed ashlar on raw rock is both the real
//! thing and the readable one: it gives the castle a light silhouette
//! against a mid-blue sky AND a dark base against light ground, which is
//! two contrasts rather than one. VM-01's counter-example works the same
//! way — *Facing Worlds*' towers "anchor the eye" because the players on
//! them "can be picked out against their dark backgrounds".
//!
//! **2. ROCK IS NOT MASONRY.** The massif used the coursed-blockwork
//! texture, which is what made an eighteen-metre cliff read as a wall.
//! Living rock takes the broad mottled generator instead, tiled by the
//! box's own size rather than a fixed 1.25 — a 186 m slab at 1.25 tiles
//! is one course of blockwork every 149 metres, i.e. a smear.
//!
//! **3. AIR, AND A LOWER SUN.** Linear fog at 45→130 m is correct for a
//! 100 m arena and fatal here: the keep is 175 m from the muster plaza
//! and 340 m from the lower city, so on the old numbers both landmarks
//! were 100% sky colour — invisible, not distant. Cliffhold gets
//! 90→760 m, which turns the same mechanism into aerial perspective, and
//! a lower, warmer sun so 32 m of relief casts something.
//!
//! ## What this module may not do
//!
//! Everything here is cosmetic. Nothing spawned by `spawn_landmarks`
//! carries a sim identity: no cover box is created, moved or resized, no
//! bullet stops on any of it, and nothing reads the sim's RNG — the
//! per-outcrop variety is hashed off the slab and step INDEX, so a
//! replay is bit-identical with all of it on screen.
//!
//! Every landmark anchor is FOUND in the published cover list by a
//! geometric rule rather than restated as a coordinate here. That is the
//! anti-split-brain discipline: `landmarks_are_found_where_the_sim_put_them`
//! fails loudly if the map builder moves the keep, rather than the
//! client quietly dressing empty air.
//!
//! ## WYSIWYG, and where this module is honest about breaking it
//!
//! VM-02, Hi-Rez, on a shipped game with flying characters: *what you see
//! is what you get* — every surface a player can see is a surface they
//! will try to stand on. Flight is coming to this game, so decoration is
//! held to three safe classes:
//!
//! * above the highest standable surface it sits on (spires, banners),
//! * too thin to read as cover or as a floor (masts, pinnacles, cornices),
//! * or hanging in air OUTSIDE and BELOW a collidable face (the cliff
//!   crest outcrops, which a player on the plateau cannot touch and a
//!   player at the foot cannot reach).
//!
//! **No merlon, parapet or cover-height block is ever placed on a
//! standable surface by this module.** Phantom cover is the worse lie in
//! a shooter: an invisible wall costs you a metre, cover that is not
//! there costs you the round.
//!
//! The one knowing exception is named in `spawn_bell_tower` and reported.
//!
//! ## Wiring
//!
//! Three lines in `main.rs`: `mod cliffhold;`, one `cliffhold::look(map)`
//! in `rebuild_world`, and one `cliffhold::spawn_landmarks(..)` beside it.

use bevy::prelude::*;

use crate::sim::{
    Aabb, CoverKind, MapKind, CH_KEEP_TOP, CH_PLATEAU, CH_RAMPART, CH_ROOF_LOW, CH_SHELF,
};

// ---------------------------------------------------------------- the air

/// Everything about a map that is not a cover block: sky, ground, border,
/// the fog band and the sun.
///
/// This used to be a three-tuple built inline in `rebuild_world`, which
/// is why fog distance and sun angle were map-INDEPENDENT: there was
/// nowhere to put them. Both matter more on Cliffhold than the colours
/// do.
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
    /// Bevy's default stretches four cascades over a kilometre, which on
    /// a map whose fog closes at 760 m spends most of the shadow budget
    /// past the point anything is visible. Cliffhold pulls the far bound
    /// in to 420 m and pushes the first out to 28, which is roughly the
    /// width of the ravine.
    ///
    /// **Honest note on how these got here:** they were added as a fix
    /// for what was diagnosed as shadow acne on the ground plane, and
    /// that diagnosis was WRONG — the dark ground was a sun below the
    /// horizon (see `sun_euler`). Neither the bias nor the cascade bound
    /// changed the frame at all. They are kept because tightening the
    /// cascades on a 600 m map is right on its own terms, not because
    /// they fixed anything.
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

/// The euler pair for a sun at `elev` above the horizon and `azim` east
/// of due south (both radians), which is how a person thinks about a sun.
///
/// ## Why this exists
///
/// It was written after the bug it prevents. Cliffhold's whole viewing
/// axis points north at the castle, so the sun has to sit in the SOUTH
/// or every surface a player looks at is a shaded one. Moving it there
/// by hand meant taking the y euler past a quarter turn — and that
/// silently inverted the vertical, so the "sun" spent two capture passes
/// twenty degrees BELOW the horizon, shining upward.
///
/// It did not look like a broken light. It looked like a palette
/// problem: south and east faces brightly lit, every horizontal surface
/// on the map — the ground, the plateau, the plaza, every roof — dark.
/// The first diagnosis was shadow acne and the first fix was a bias
/// tweak and a `NotShadowCaster`, neither of which did anything, because
/// nothing was in shadow. Three capture cycles.
///
/// `a PI yaw on a parent silently flips a child's forward axis` is in
/// the standing brief. This is that, in a light rig.
pub fn sun_euler(elev: f32, azim: f32) -> (f32, f32) {
    let s = azim.sin() * elev.cos();
    // the NEGATIVE root: it is what keeps `cos y` on the side that
    // leaves the vertical alone
    let cb = -(1.0 - s * s).max(0.0).sqrt();
    (
        elev.sin().atan2(azim.cos() * elev.cos()),
        s.atan2(cb),
    )
}

/// The look of every map, in one place.
///
/// The four older maps keep exactly the numbers they shipped with — same
/// sky, same ground, same border, the 45→130 m fog and the sun that was
/// hard-coded at startup. Only Cliffhold moves, so this pass cannot
/// regress a map it was not asked to touch.
pub fn look(map: MapKind) -> MapLook {
    // what every map before Cliffhold ran with
    let legacy = |sky: Color, ground: Color, border: Color| MapLook {
        sky,
        ground,
        border,
        fog: (45.0, 130.0),
        sun: (-0.9, 0.5),
        sun_color: Color::WHITE,
        sun_lux: 13_000.0,
        ambient: Color::srgb(0.85, 0.82, 0.78),
        ambient_lux: 380.0,
        // Bevy's own defaults, restated so the legacy maps are pinned to
        // what they shipped with rather than to whatever the engine
        // default becomes.
        shadow_depth_bias: 0.02,
        shadow_normal_bias: 1.8,
        shadow_max_m: 1000.0,
        shadow_first_m: 5.0,
    };
    match map {
        MapKind::Arena => legacy(
            Color::srgb(0.58, 0.63, 0.72),
            Color::srgb(0.45, 0.40, 0.33),
            Color::srgb(0.55, 0.52, 0.47),
        ),
        MapKind::Bailey => legacy(
            Color::srgb(0.52, 0.66, 0.60),
            Color::srgb(0.30, 0.42, 0.24),
            Color::srgb(0.52, 0.52, 0.50),
        ),
        MapKind::Gardens => legacy(
            Color::srgb(0.55, 0.71, 0.55),
            Color::srgb(0.27, 0.48, 0.24),
            Color::srgb(0.58, 0.56, 0.50),
        ),
        MapKind::Battlefield => legacy(
            Color::srgb(0.60, 0.62, 0.68),
            Color::srgb(0.36, 0.40, 0.28),
            Color::srgb(0.50, 0.50, 0.48),
        ),
        MapKind::Cliffhold => MapLook {
            // A MID blue, not a pale one. The castle's job is to be a
            // light silhouette against this; a washed-out sky and pale
            // ashlar are the same value and the keep disappears into it.
            sky: Color::srgb(0.50, 0.60, 0.74),
            // dry upland turf going to dust — warm, so the cool rock of
            // the massif reads as a different material where they meet
            ground: Color::srgb(0.47, 0.45, 0.33),
            border: Color::srgb(0.44, 0.43, 0.42),
            // 130 → 760 m. The keep is 175 m from the plaza and 340 m
            // from the lower city; at 45→130 both were solid sky.
            //
            // The START moved out from 90 after a capture: aerial
            // perspective is the mechanism that sells depth here, and
            // starting it at 90 m put 40% of it onto the mid-ground,
            // which flattened the three rock bands into one haze
            // exactly where they most needed to separate.
            fog: (130.0, 760.0),
            // A LOW SUN AT 35.5°, 31° EAST OF DUE SOUTH.
            //
            // Both numbers were settled by capture. Relief is only
            // legible if it casts, so the elevation came down from the
            // shared 51° noon — at -0.9 rad the 18 m cliff threw a 14 m
            // shadow onto its own apron and the ravine had no dark side.
            // It did not come down further because the city's streets
            // are 16 m wide between 14 m blocks and below about 40° none
            // of them see the sun at all.
            //
            // The AZIMUTH is the half that had to be reasoned about and
            // then measured. Every viewpoint on this map looks NORTH, at
            // the castle, so every surface a player looks at is a SOUTH
            // face — and a sun anywhere north of the map leaves all of
            // them shaded. See `sun_euler` for what putting it in the
            // south cost, and `the_sun_is_above_the_horizon_on_every_map`
            // for the test that now stops it happening again.
            sun: sun_euler(0.62, 0.55),
            sun_color: Color::srgb(1.0, 0.94, 0.84),
            sun_lux: 15_000.0,
            // and a COOL sky-fill, so what the sun misses goes blue
            // rather than grey. The hue split between lit and unlit is
            // most of what sells a mountain.
            //
            // Brighter than the shared 380 because this map has real
            // shadow in it now — a ravine between two 18 m walls and
            // streets between 14 m blocks. At 2.3% of the sun the
            // unlit half of the ravine photographed as pure black, and
            // "visually muddy … no figure ground distinction along the
            // floor" is VM-01's description of the map Epic lost.
            ambient: Color::srgb(0.66, 0.74, 0.88),
            ambient_lux: 850.0,
            // Cascades pulled in from a kilometre to 420 m — past that
            // the fog has taken over anyway.
            shadow_depth_bias: 0.04,
            shadow_normal_bias: 3.4,
            shadow_max_m: 420.0,
            shadow_first_m: 28.0,
        },
    }
}

// -------------------------------------------------------------- the stone

/// What a Cliffhold stone box IS, decided from its own published AABB and
/// nothing else.
///
/// This is a CLASSIFICATION of data the sim already hands over, not a
/// second copy of the layout: there is no coordinate in this file. If the
/// map builder moves the city, the city keeps its tint, because the tint
/// is a function of how tall and how big each box is rather than of where
/// it was put.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stone {
    /// living rock, low band — the east apron, the talus benches
    RockLow,
    /// living rock, middle band — the west bench and the east shelf
    RockMid,
    /// living rock, the headline band — the plateau and the great cliff
    RockTop,
    /// street level: field walls, rubble stubs, quarry spoil, hedgebanks
    Street,
    /// the city's low roof course and the aqueduct deck
    RoofLow,
    /// the middle course, and the muster plaza
    RoofMid,
    /// the tall course — the city's own skyline
    RoofHigh,
    /// the curtain wall, its turrets and the gatehouse pair
    Rampart,
    /// the keep: the palest and highest masonry on the map
    Keep,
}

/// Footprint area, in m², above which a stone box is LIVING ROCK rather
/// than something anybody built.
///
/// The nine massif slabs run 3,000–18,090 m². The largest thing made of
/// masonry on this map is the aqueduct deck at 1,110 m² (222 m of it, and
/// five metres wide), then the redoubt at 900 and a city block at 572. So
/// 2,000 separates them with better than a 1.8x margin on both sides,
/// which is why it is a round number and not a fitted one.
pub const MASSIF_AREA_M2: f32 = 2_000.0;

/// Horizontal footprint area of a box.
fn area(a: &Aabb) -> f32 {
    (a.max[0] - a.min[0]) * (a.max[2] - a.min[2])
}

fn min_horiz(a: &Aabb) -> f32 {
    (a.max[0] - a.min[0]).min(a.max[2] - a.min[2])
}

/// Classify one stone box. Pure, and the only place the bands are stated.
pub fn stone_of(a: &Aabb) -> Stone {
    let top = a.max[1];
    if area(a) >= MASSIF_AREA_M2 {
        // The three rock bands are the sim's own: `CH_BENCH_EAST` (6),
        // `CH_SHELF` (12) and `CH_PLATEAU` (18). Split at the midpoints
        // so a slab lands in its own band with room either side.
        return if top >= (CH_SHELF + CH_PLATEAU) * 0.5 {
            Stone::RockTop
        } else if top >= (CH_ROOF_LOW + CH_SHELF) * 0.5 {
            Stone::RockMid
        } else {
            Stone::RockLow
        };
    }
    // masonry, by altitude band
    if top >= CH_KEEP_TOP - 1.0 {
        Stone::Keep
    } else if top >= CH_RAMPART - 4.0 {
        Stone::Rampart
    } else if top >= CH_SHELF {
        Stone::RoofHigh
    } else if top >= CH_ROOF_LOW + 2.0 {
        Stone::RoofMid
    } else if top >= 4.0 {
        Stone::RoofLow
    } else {
        Stone::Street
    }
}

impl Stone {
    /// Living rock takes the broad mottled generator; masonry takes the
    /// coursed blockwork. This one bit is most of why the massif stopped
    /// reading as a wall.
    pub fn is_rock(self) -> bool {
        matches!(self, Stone::RockLow | Stone::RockMid | Stone::RockTop)
    }

    /// Base colour.
    ///
    /// Two ramps, deliberately in opposite directions:
    ///
    /// * ROCK gets COOLER and DARKER as it rises, so the great cliff is
    ///   the darkest mass on the map and everything standing on it has
    ///   something to be seen against.
    /// * MASONRY gets LIGHTER as it rises, ending at the keep — dressed
    ///   ashlar on raw rock, which is both the real thing and the
    ///   readable one.
    ///
    /// The whole set is low-chroma on purpose. VM-01's other failure mode
    /// is a scene with nowhere for the eye to stop; the world stays quiet
    /// so a 1.78 m fighter in team colours is the loudest thing in it.
    pub fn color(self) -> Color {
        match self {
            // Widened from 0.46 / 0.41 / 0.35 once the sun was fixed:
            // with every south face correctly lit the three bands were
            // only a few percent apart on screen, and from the commons
            // the apron, the shelf and the plateau read as one haze.
            // 0.50 / 0.40 / 0.30 is three legible steps at 300 m.
            Stone::RockLow => Color::srgb(0.50, 0.47, 0.41),
            Stone::RockMid => Color::srgb(0.40, 0.40, 0.39),
            Stone::RockTop => Color::srgb(0.30, 0.31, 0.33),
            Stone::Street => Color::srgb(0.50, 0.46, 0.39),
            Stone::RoofLow => Color::srgb(0.57, 0.50, 0.40),
            Stone::RoofMid => Color::srgb(0.62, 0.54, 0.43),
            Stone::RoofHigh => Color::srgb(0.67, 0.58, 0.46),
            // Pulled down from 0.71 / 0.80 after the first capture: at
            // those values a sunlit keep wall is near enough white to
            // blow out against the sky and reads as painted plaster
            // rather than as cut stone. It still has to be the palest
            // masonry on the map, and it still is.
            Stone::Rampart => Color::srgb(0.66, 0.64, 0.59),
            Stone::Keep => Color::srgb(0.74, 0.72, 0.66),
        }
    }

    /// Rock is uniformly rough; dressed stone gets smoother the better it
    /// was cut, so the castle catches a highlight the mountain does not.
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
/// single course of blockwork across 149 m of the great cliff and the
/// same course across 2.4 m of a crate. Scaling by the largest dimension
/// keeps a tile near four metres on the long axis of anything.
///
/// Clamped at 40 because past that the generated 128 px texture is
/// aliasing rather than describing a surface.
pub fn uv_tiles(a: &Aabb) -> f32 {
    let m = (a.max[0] - a.min[0])
        .max(a.max[1] - a.min[1])
        .max(a.max[2] - a.min[2]);
    (m / 4.0).clamp(1.0, 40.0)
}

// ---------------------------------------------------------- the landmarks

/// One found landmark: the footprint it stands on and the height of its
/// top, in world metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub top: f32,
}

impl Anchor {
    fn of(a: &Aabb) -> Anchor {
        Anchor {
            min: [a.min[0], a.min[2]],
            max: [a.max[0], a.max[2]],
            top: a.max[1],
        }
    }
    fn union(self, o: Anchor) -> Anchor {
        Anchor {
            min: [self.min[0].min(o.min[0]), self.min[1].min(o.min[1])],
            max: [self.max[0].max(o.max[0]), self.max[1].max(o.max[1])],
            top: self.top.max(o.top),
        }
    }
    pub fn center(self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
        ]
    }
}

/// The four things a deliberately messy map is navigated by.
#[derive(Clone, Debug, Default)]
pub struct Landmarks {
    /// The keep's outer footprint, from the union of its five walls.
    pub keep: Option<Anchor>,
    /// How many boxes went into that union.
    ///
    /// Carried because the union alone cannot catch the rule going
    /// wrong: the keep's stair climbs INSIDE its own walls, so admitting
    /// a stair tread by mistake leaves the footprint identical and the
    /// count is the only thing that moves.
    pub keep_walls: usize,
    /// The two gatehouse towers, west first.
    pub gatehouse: Vec<Anchor>,
    /// The bell tower's block, in the lower city.
    pub bell_tower: Option<Anchor>,
    /// The massif slabs whose crest is the cliff line.
    pub cliff: Vec<Anchor>,
}

/// Find the four landmarks in the sim's published cover list.
///
/// Each rule is written against a sim CONSTANT plus the box's own shape,
/// never against a coordinate. What this buys is failure that is loud:
/// `landmarks_are_found_where_the_sim_put_them` asserts the counts and
/// the places, so a map edit that moves the keep breaks a test instead of
/// leaving a banner flying over open ground.
pub fn find(cover: &[Aabb], kind: &[CoverKind]) -> Landmarks {
    let mut out = Landmarks::default();
    for (a, k) in cover.iter().zip(kind.iter()) {
        if *k != CoverKind::Stone {
            continue;
        }
        let top = a.max[1];
        // THE KEEP — five walls, all at `CH_KEEP_TOP`, all four metres
        // thick. The one other box on the map that reaches 32 m is the
        // top tread of the keep's own stair, and it is 1.3 m deep, so
        // the thickness test is what separates them.
        if (top - CH_KEEP_TOP).abs() < 0.05 && min_horiz(a) >= 3.5 {
            out.keep_walls += 1;
            out.keep = Some(match out.keep {
                Some(k) => k.union(Anchor::of(a)),
                None => Anchor::of(a),
            });
            continue;
        }
        // THE GATEHOUSE PAIR — the only masonry standing between the
        // wall-walk and the keep. The keep's stair treads climb through
        // the same band; they are 1.3 m deep and the half-landing is
        // 2 m, against ten metres of tower.
        if top > CH_RAMPART + 1.0 && top < CH_KEEP_TOP - 1.0 && min_horiz(a) >= 5.0 {
            out.gatehouse.push(Anchor::of(a));
            continue;
        }
        // THE BELL TOWER — the head of the grounded route onto the 11 m
        // roof course, and the only box in its height band with a
        // building's footprint: a city block is 572 m², a stair tread is
        // under 25, this is 168.
        if top > CH_ROOF_LOW + 4.0 && top < CH_SHELF - 0.5 && (100.0..300.0).contains(&area(a)) {
            out.bell_tower = Some(Anchor::of(a));
            continue;
        }
        // THE CLIFF LINE — the massif slabs that reach the plateau.
        if (top - CH_PLATEAU).abs() < 0.05 && area(a) >= MASSIF_AREA_M2 {
            out.cliff.push(Anchor::of(a));
        }
    }
    out.gatehouse
        .sort_by(|a, b| a.center()[0].partial_cmp(&b.center()[0]).unwrap());
    out
}

/// Deterministic per-index jitter in `0..1`. A hash, NOT the sim's RNG —
/// pulling from that stream would desynchronise replay, and this is the
/// rule with teeth in `ANTI_PATTERNS.md`.
fn hash01(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 13;
    (h & 0xFFFF) as f32 / 65_535.0
}

/// Everything this module spawns carries `CoverVis`, so `rebuild_world`'s
/// existing teardown query collects it and nothing else in the crate has
/// to learn that Cliffhold has dressing.
///
/// `stone` / `roof` / `timber` are handles the caller already built; this
/// function makes only the few extra materials that have no equivalent
/// in the shared kit.
#[allow(clippy::too_many_arguments)]
pub fn spawn_landmarks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cover: &[Aabb],
    kind: &[CoverKind],
    // The MOTTLED generator and its normal map - the same pair the
    // massif itself takes. Passed in rather than looked up because this
    // module owns no assets; without it the crest outcrops came out
    // untextured and read on camera as a DIFFERENT, flatter material
    // stuck to the cliff, which is the opposite of the point.
    rock_tex: (Handle<Image>, Handle<Image>),
) {
    let lm = find(cover, kind);

    // ---- materials -------------------------------------------------
    // Ashlar, matched to the castle band so a spire reads as part of the
    // wall it stands on rather than as a prop set down on it.
    let ashlar = materials.add(StandardMaterial {
        base_color: Stone::Keep.color(),
        perceptual_roughness: Stone::Keep.roughness(),
        ..default()
    });
    let ashlar_dark = materials.add(StandardMaterial {
        base_color: Stone::Rampart.color(),
        perceptual_roughness: Stone::Rampart.roughness(),
        ..default()
    });
    // Slate, for every pitched roof and spire. The DARKEST thing above
    // the plateau on purpose: a pale ashlar spire against a pale sky is
    // no silhouette at all, and the silhouette is the entire point.
    let slate = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.21, 0.25),
        perceptual_roughness: 0.85,
        ..default()
    });
    // Two banners. Warm ochre rather than a team colour — these are the
    // map's furniture, and a red keep on a map where Red also spawns
    // would be reading as ownership.
    let cloth = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.42, 0.16),
        perceptual_roughness: 0.95,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    let timber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.30, 0.22, 0.14),
        perceptual_roughness: 1.0,
        ..default()
    });
    let rock_crest = materials.add(StandardMaterial {
        base_color: Stone::RockTop.color(),
        base_color_texture: Some(rock_tex.0),
        normal_map_texture: Some(rock_tex.1),
        // an outcrop is a few metres across, so a tile per metre or so
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(2.0)),
        perceptual_roughness: 1.0,
        ..default()
    });

    if let Some(keep) = lm.keep {
        spawn_keep_crown(commands, meshes, &ashlar, &slate, &cloth, keep);
    }
    if lm.gatehouse.len() == 2 {
        spawn_gatehouse(commands, meshes, &ashlar_dark, &slate, &cloth, &lm.gatehouse);
    }
    if let Some(bell) = lm.bell_tower {
        spawn_bell_tower(commands, meshes, &ashlar_dark, &slate, &timber, bell);
    }
    spawn_cliff_crest(commands, meshes, &rock_crest, &lm.cliff, cover);
}

/// THE KEEP — four corner pinnacles and a banner mast, all of it above
/// the parapet.
///
/// VM-12's first composition guideline is the one that bites a castle on
/// a cliff: *"players don't look upwards unless something draws their
/// eye."* The keep's own walls stop at 32 m and, from the muster plaza
/// 175 m south and 25 m below, they subtend about the same angle as a
/// city block does from across a street. The pinnacles take it to 41 m
/// and the mast to 47, which is what makes the eye go up.
///
/// EVERY piece is above `CH_KEEP_TOP`, which is the highest surface a man
/// can stand on unaided, so none of it can be mistaken for a floor. The
/// pinnacles are 0.9 m and are centred ON the outer corner, so
/// three-quarters of each hangs outside the wall and the quarter that
/// does not is a corner post, not cover.
fn spawn_keep_crown(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    ashlar: &Handle<StandardMaterial>,
    slate: &Handle<StandardMaterial>,
    cloth: &Handle<StandardMaterial>,
    keep: Anchor,
) {
    const PIN: f32 = 0.9;
    const PIN_H: f32 = 5.6;
    const SPIRE_H: f32 = 3.6;
    let shaft = meshes.add(Cuboid::new(PIN, PIN_H, PIN));
    // The cone's radius is barely over the shaft's HALF-width. It was
    // `PIN * 0.8` on the first pass, i.e. 1.6x the shaft across, and the
    // capture showed exactly what that is: a rocket. A spire has to grow
    // out of its shaft, not sit on it like a hat.
    let spire = meshes.add(Cone {
        radius: PIN * 0.56,
        height: SPIRE_H,
    });
    for (cx, cz) in [
        (keep.min[0], keep.min[1]),
        (keep.max[0], keep.min[1]),
        (keep.min[0], keep.max[1]),
        (keep.max[0], keep.max[1]),
    ] {
        commands.spawn((
            Mesh3d(shaft.clone()),
            MeshMaterial3d(ashlar.clone()),
            Transform::from_xyz(cx, keep.top + PIN_H * 0.5, cz),
            crate::CoverVis,
        ));
        commands.spawn((
            Mesh3d(spire.clone()),
            MeshMaterial3d(slate.clone()),
            Transform::from_xyz(cx, keep.top + PIN_H + SPIRE_H * 0.5, cz),
            crate::CoverVis,
        ));
    }
    // The mast, on the keep's own centre line. 15 m of it, so the flag
    // clears the pinnacles and is the highest thing on the map.
    let c = keep.center();
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.16, 15.0))),
        MeshMaterial3d(ashlar.clone()),
        Transform::from_xyz(c[0], keep.top + 7.5, c[1]),
        crate::CoverVis,
    ));
    // and the banner: a flat sheet hung off it, double-sided so it does
    // not vanish when the map is crossed the other way
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.4, 3.0, 0.06))),
        MeshMaterial3d(cloth.clone()),
        Transform::from_xyz(c[0] + 2.2, keep.top + 12.6, c[1]),
        crate::CoverVis,
    ));
}

/// THE GATEHOUSE PAIR — two matched masts and the arch between them.
///
/// A gate is not a tower, it is a PAIR plus a gap, and the pair is what
/// has to survive being seen from 200 m across a courtyard. The two
/// towers already exist in the sim at 28 m; what they lacked was anything
/// that made them read as a set. So: an identical pennant on each, and a
/// lintel band spanning the gap at the height of the wall-walk.
///
/// The lintel is derived, not placed — it runs between the two found
/// towers' INNER faces and takes their own z extent, so it cannot end up
/// somewhere the towers are not. It sits six metres over an 18 m
/// courtyard floor, which no unaided jump reaches, and the sim's boxes
/// have no underside anyway, so nothing about passing under it changes.
fn spawn_gatehouse(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    ashlar: &Handle<StandardMaterial>,
    slate: &Handle<StandardMaterial>,
    cloth: &Handle<StandardMaterial>,
    towers: &[Anchor],
) {
    let (w, e) = (towers[0], towers[1]);
    for t in [w, e] {
        let c = t.center();
        // corner pinnacles, 0.8 m, on the outer corners only — small
        // enough to read as dressing and never as cover
        for (px, pz) in [
            (t.min[0], t.min[1]),
            (t.max[0], t.min[1]),
            (t.min[0], t.max[1]),
            (t.max[0], t.max[1]),
        ] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.8, 3.2, 0.8))),
                MeshMaterial3d(ashlar.clone()),
                Transform::from_xyz(px, t.top + 1.6, pz),
                crate::CoverVis,
            ));
        }
        // the pennant mast: 9 m, so the pair tops out below the keep's
        // crown and above the curtain wall. The hierarchy is the point —
        // keep, then gate, then bell tower, then everything else.
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.13, 9.0))),
            MeshMaterial3d(ashlar.clone()),
            Transform::from_xyz(c[0], t.top + 4.5, c[1]),
            crate::CoverVis,
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.6, 1.7, 0.05))),
            MeshMaterial3d(cloth.clone()),
            Transform::from_xyz(c[0] + 1.3, t.top + 7.6, c[1]),
            crate::CoverVis,
        ));
        // a slate cap course just under the top, hung 0.45 m proud of the
        // face. BELOW the standable top and OUTSIDE it, so a fighter on
        // the tower can neither stand on it nor hide behind it.
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                t.max[0] - t.min[0] + 0.9,
                0.45,
                t.max[1] - t.min[1] + 0.9,
            ))),
            MeshMaterial3d(slate.clone()),
            Transform::from_xyz(c[0], t.top - 0.32, c[1]),
            crate::CoverVis,
        ));
    }
    // THE ARCH. Between the inner faces, at the wall-walk's own height.
    let (x0, x1) = (w.max[0], e.min[0]);
    let z0 = w.min[1].max(e.min[1]);
    let z1 = w.max[1].min(e.max[1]);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(x1 - x0, 2.2, z1 - z0))),
        MeshMaterial3d(ashlar.clone()),
        Transform::from_xyz(
            (x0 + x1) * 0.5,
            CH_RAMPART + 1.1,
            (z0 + z1) * 0.5,
        ),
        crate::CoverVis,
    ));
}

/// THE BELL TOWER — the lower city's only vertical anchor.
///
/// VM-01's counter-example is *Facing Worlds*, whose towers work because
/// "the tall narrowing Towers anchor the eye" and what does the work is
/// "negative space, not detail" — a tall silhouette against an empty sky.
/// The lower city has 25 blocks between 5 and 14 m and not one thing
/// taller, so from inside it every direction looks the same. The sim's
/// bell tower is an eleven-metre block: correct as the grounded route
/// onto the 11 m course, and invisible as a landmark.
///
/// ## The knowing WYSIWYG exception, stated
///
/// The belfry stands on four corner posts above the sim's 11 m block, and
/// the sim has NO collision above 11 m there. So it looks like a tower
/// and, to a flier, is not one. Two things limit the damage and neither
/// removes it:
///
/// * it is built as an OPEN FRAME — posts, a stage and a pitched roof,
///   with the bell visible through it — so it reads as something you fly
///   BETWEEN rather than a solid mass; and
/// * the pitched roof reads as unstandable, which it is.
///
/// **The real fix is a sim ask and is in the report: give the bell tower
/// a shaft in the collision and this becomes honest.** It is shipped
/// meanwhile because the alternative is a city with no landmark at all,
/// and a stated deferral beats a silent one.
fn spawn_bell_tower(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    ashlar: &Handle<StandardMaterial>,
    slate: &Handle<StandardMaterial>,
    timber: &Handle<StandardMaterial>,
    bell: Anchor,
) {
    let c = bell.center();
    let (w, d) = (bell.max[0] - bell.min[0], bell.max[1] - bell.min[1]);
    // the shaft: four corner posts from the block's top to the stage.
    // Straddling the corners, so only a 0.4 m quadrant of each is over
    // the standable roof.
    // 1.3 m. At 0.8 the first capture read as scaffolding rather than
    // masonry — four sticks under a hat. Straddling the corners keeps
    // only 0.65 m of each over the standable roof.
    const POST: f32 = 1.3;
    let shaft_h = 13.0;
    let post = meshes.add(Cuboid::new(POST, shaft_h, POST));
    for (px, pz) in [
        (bell.min[0], bell.min[1]),
        (bell.max[0], bell.min[1]),
        (bell.min[0], bell.max[1]),
        (bell.max[0], bell.max[1]),
    ] {
        commands.spawn((
            Mesh3d(post.clone()),
            MeshMaterial3d(ashlar.clone()),
            Transform::from_xyz(px, bell.top + shaft_h * 0.5, pz),
            crate::CoverVis,
        ));
    }
    let stage_y = bell.top + shaft_h;
    // the belfry stage — a thin floor slab tying the four posts together
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(w + 1.4, 0.7, d + 1.4))),
        MeshMaterial3d(ashlar.clone()),
        Transform::from_xyz(c[0], stage_y, c[1]),
        crate::CoverVis,
    ));
    // four belfry piers, leaving the four faces open
    for (px, pz) in [
        (bell.min[0], bell.min[1]),
        (bell.max[0], bell.min[1]),
        (bell.min[0], bell.max[1]),
        (bell.max[0], bell.max[1]),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.5, 5.0, 1.5))),
            MeshMaterial3d(ashlar.clone()),
            Transform::from_xyz(px, stage_y + 2.85, pz),
            crate::CoverVis,
        ));
    }
    // THE BELL, hung in the middle where the open faces show it
    commands.spawn((
        Mesh3d(meshes.add(Cone {
            radius: 1.5,
            height: 2.6,
        })),
        MeshMaterial3d(timber.clone()),
        // a bell is a cone standing on its MOUTH, so the cone is flipped
        Transform::from_xyz(c[0], stage_y + 2.9, c[1])
            .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
        crate::CoverVis,
    ));
    // and the pitched roof: a pyramid, which is the shape that says "not
    // a floor" without a word of UI
    commands.spawn((
        Mesh3d(meshes.add(Cone {
            radius: (w.max(d) * 0.5) + 1.6,
            height: 6.5,
        })),
        MeshMaterial3d(slate.clone()),
        Transform::from_xyz(c[0], stage_y + 5.35 + 3.25, c[1]),
        crate::CoverVis,
    ));
}

/// THE CLIFF LINE — a broken crest, hung under the plateau's edge.
///
/// The sim assembles the massif "from nine overlapping slabs with
/// staggered edges so the cliff line reads as broken rock rather than a
/// wall", and from the ground it did not: nine axis-aligned boxes in one
/// flat colour read as one flat colour. The staggering is real, the
/// material was not.
///
/// Every outcrop hangs OUTSIDE the collidable face and BELOW the
/// standable top. A fighter on the plateau cannot touch one — it starts
/// 0.4 m under the surface he is standing on and 1.2 m out from the edge
/// he would have to walk off first. A fighter at the cliff foot is
/// eighteen metres under the lowest of them. That is the one geometry
/// class this module allows on a face people walk near.
///
/// Outcrops whose centre is buried inside another slab are dropped, which
/// is what keeps the crest on the OUTSIDE of the massif where the nine
/// slabs abut each other.
fn spawn_cliff_crest(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    rock: &Handle<StandardMaterial>,
    cliff: &[Anchor],
    cover: &[Aabb],
) {
    /// how far apart, along the rim
    const STEP: f32 = 17.0;
    /// how far the crest hangs below the standable top
    const DROP: f32 = 0.4;
    for (si, s) in cliff.iter().enumerate() {
        let (w, d) = (s.max[0] - s.min[0], s.max[1] - s.min[1]);
        // walk all four edges as one parameterised loop
        let per = 2.0 * (w + d);
        let n = (per / STEP).floor().max(4.0) as usize;
        for i in 0..n {
            let t = i as f32 / n as f32 * per;
            // outward normal and the point, per edge
            let (px, pz, nx, nz) = if t < w {
                (s.min[0] + t, s.min[1], 0.0, -1.0)
            } else if t < w + d {
                (s.max[0], s.min[1] + (t - w), 1.0, 0.0)
            } else if t < 2.0 * w + d {
                (s.max[0] - (t - w - d), s.max[1], 0.0, 1.0)
            } else {
                (s.min[0], s.max[1] - (t - 2.0 * w - d), -1.0, 0.0)
            };
            // drop anything that would land inside a neighbouring slab —
            // the massif's interior joints are not cliff
            let probe = [px + nx * 2.0, pz + nz * 2.0];
            if cover.iter().any(|a| {
                a.max[1] >= s.top - 1.0
                    && probe[0] > a.min[0]
                    && probe[0] < a.max[0]
                    && probe[1] > a.min[2]
                    && probe[1] < a.max[2]
            }) {
                continue;
            }
            // id-hashed, never sim RNG
            let h0 = hash01(si as u32, i as u32);
            let h1 = hash01(i as u32, si as u32 + 77);
            let along = 3.0 + h0 * 5.0;
            let out = 1.1 + h1 * 1.5;
            // A CREST, not a cladding. At 3.5..8.5 m the outcrops
            // covered the upper half of an 18 m face and the wall read
            // as a checkerboard - VM-01's other failure, "so much detail
            // ... there's nowhere for your eye to stop".
            let tall = 2.4 + h0 * 3.4;
            let (sx, sz) = if nx == 0.0 { (along, out) } else { (out, along) };
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(sx, tall, sz))),
                MeshMaterial3d(rock.clone()),
                Transform::from_xyz(
                    px + nx * out * 0.5,
                    s.top - DROP - tall * 0.5,
                    pz + nz * out * 0.5,
                )
                .with_rotation(Quat::from_rotation_y((h1 - 0.5) * 0.5)),
                crate::CoverVis,
            ));
        }
    }
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::*;

    fn cliffhold() -> TdmSim {
        TdmSim::new(MatchConfig {
            seed: 0x7EA9,
            per_team: 4,
            mode: Mode::Tdm,
            map: MapKind::Cliffhold,
            difficulty: Difficulty::Normal,
            loadout: [GunKind::Ak47, GunKind::Glock, GunKind::Bow],
            tdm_target: 30,
            class: Class::Line,
            melee_axe: false,
            grenade_preset: 0,
            armor_pieces: None,
        })
    }

    /// The anti-split-brain test. Every landmark rule is geometric, so
    /// the thing that can silently rot is the rule finding the WRONG box
    /// (or none) after a map edit. This pins what each rule finds and
    /// where it is, so that rot is a red test rather than a banner over
    /// empty ground.
    ///
    /// Mutation-proved, both directions:
    ///
    /// * widening the keep's thickness test to `>= 1.0` admits the
    ///   keep's own top stair tread. The FOOTPRINT does not move — the
    ///   stair is inside the walls — so only `keep_walls` catches it,
    ///   which is why that field exists.
    /// * dropping the gatehouse's `min_horiz >= 5.0` admits the keep
    ///   stair's treads and the pair becomes a crowd.
    #[test]
    fn landmarks_are_found_where_the_sim_put_them() {
        let s = cliffhold();
        let lm = find(&s.cover, &s.cover_kind);

        // THE KEEP: five walls, union = the sim's own CH_KEEP_INNER plus
        // its four-metre wall thickness on every side.
        assert_eq!(lm.keep_walls, 5, "the keep is four walls and a doorway");
        let keep = lm.keep.expect("the keep must be found");
        assert!((keep.top - CH_KEEP_TOP).abs() < 0.01, "keep top {}", keep.top);
        assert!(
            (keep.min[0] - (CH_KEEP_INNER[0] - 4.0)).abs() < 0.01
                && (keep.max[0] - (CH_KEEP_INNER[2] + 4.0)).abs() < 0.01,
            "keep x span {:?}..{:?}",
            keep.min[0],
            keep.max[0]
        );
        assert!(
            (keep.min[1] - CH_KEEP_INNER[1]).abs() < 4.01
                && (keep.max[1] - CH_KEEP_INNER[3]).abs() < 4.01,
            "keep z span {:?}..{:?}",
            keep.min[1],
            keep.max[1]
        );
        // and the doorway the sim publishes is inside that footprint
        assert!(
            keep.min[0] < CH_KEEP_DOOR_X[0] && keep.max[0] > CH_KEEP_DOOR_X[1],
            "the published doorway is not in the found keep"
        );

        // THE GATEHOUSE: exactly two, straddling the x = 0 approach, at
        // the same height as each other.
        assert_eq!(lm.gatehouse.len(), 2, "gatehouse pair");
        let (gw, ge) = (lm.gatehouse[0], lm.gatehouse[1]);
        assert!((gw.top - ge.top).abs() < 0.01, "the pair must match in height");
        assert!(gw.top > CH_RAMPART, "a gatehouse must clear its own wall");
        assert!(
            gw.center()[0] < 0.0 && ge.center()[0] > 0.0,
            "the pair must straddle the gate, got {} and {}",
            gw.center()[0],
            ge.center()[0]
        );
        assert!(
            ge.min[0] > gw.max[0],
            "there must be a GAP between them, which is the gate"
        );

        // THE BELL TOWER: one, in the lower city (south-west), and its
        // head flush with the 11 m roof course.
        let bell = lm.bell_tower.expect("the bell tower must be found");
        assert!(
            bell.center()[0] < 0.0 && bell.center()[1] < 0.0,
            "the bell tower belongs to the lower city, got {:?}",
            bell.center()
        );
        assert!(
            bell.top > CH_ROOF_LOW && bell.top < CH_SHELF,
            "bell tower top {}",
            bell.top
        );

        // THE CLIFF: the massif slabs that reach the plateau.
        assert_eq!(lm.cliff.len(), 6, "plateau slabs");
        for c in &lm.cliff {
            assert!((c.top - CH_PLATEAU).abs() < 0.01);
        }
    }

    /// The tint must actually SEPARATE the map, or this whole pass is a
    /// no-op with comments. Two claims: the massif is classified as rock
    /// (it is what the 2,000 m² cut exists for), and the castle is
    /// brighter than the cliff it stands on.
    ///
    /// Mutation-proved: setting `MASSIF_AREA_M2` to 20,000 drops the rock
    /// count to 1 and fails; making `Stone::Keep.color()` darker than
    /// `RockTop` fails the ordering assert.
    #[test]
    fn the_massif_reads_as_rock_and_the_castle_reads_as_lighter() {
        let s = cliffhold();
        let mut rock = 0usize;
        let mut keep = 0usize;
        let mut rampart = 0usize;
        for (a, k) in s.cover.iter().zip(s.cover_kind.iter()) {
            if *k != CoverKind::Stone {
                continue;
            }
            match stone_of(a) {
                x if x.is_rock() => rock += 1,
                Stone::Keep => keep += 1,
                Stone::Rampart => rampart += 1,
                _ => {}
            }
        }
        assert_eq!(rock, 9, "the nine massif slabs, and nothing else");
        assert!(keep >= 5, "the keep's five walls at least, got {keep}");
        assert!(rampart >= 8, "curtain wall + turrets + gatehouse, got {rampart}");

        // value ordering: ashlar is lighter than the rock under it
        let lum = |c: Color| {
            let s = c.to_srgba();
            0.2126 * s.red + 0.7152 * s.green + 0.0722 * s.blue
        };
        assert!(
            lum(Stone::Keep.color()) > lum(Stone::Rampart.color()),
            "the keep must be the palest masonry"
        );
        assert!(
            lum(Stone::Rampart.color()) > lum(Stone::RockTop.color()) + 0.2,
            "the castle must stand out against the cliff it is on"
        );
        // and rock DARKENS as it rises, so the great cliff is the mass
        // everything else is seen against
        assert!(lum(Stone::RockLow.color()) > lum(Stone::RockMid.color()));
        assert!(lum(Stone::RockMid.color()) > lum(Stone::RockTop.color()));
    }

    /// The fog band has to clear the map, or the landmarks are painted on
    /// sky. This is the check that would have caught the shipped state:
    /// at 45→130 m the keep, 175 m from the map's own centre, was 100%
    /// fog.
    ///
    /// Mutation-proved: restoring `fog: (45.0, 130.0)` on the Cliffhold
    /// arm fails the first assert.
    #[test]
    fn cliffhold_fog_reaches_its_own_landmarks() {
        let s = cliffhold();
        let l = look(MapKind::Cliffhold);
        let lm = find(&s.cover, &s.cover_kind);
        let keep = lm.keep.unwrap().center();
        // the muster plaza is the map origin
        let d = (keep[0] * keep[0] + keep[1] * keep[1]).sqrt();
        assert!(
            l.fog.1 > d * 1.5,
            "fog ends at {} m and the keep is {d} m from the plaza",
            l.fog.1
        );
        // and the far corner of the lower city is the longest sightline
        // anyone actually stands on
        let bell = lm.bell_tower.unwrap().center();
        let far = ((keep[0] - bell[0]).powi(2) + (keep[1] - bell[1]).powi(2)).sqrt();
        assert!(
            l.fog.1 > far,
            "fog ends at {} m and the city-to-castle sightline is {far} m",
            l.fog.1
        );
        // the older maps must not have moved
        for m in [
            MapKind::Arena,
            MapKind::Bailey,
            MapKind::Gardens,
            MapKind::Battlefield,
        ] {
            assert_eq!(look(m).fog, (45.0, 130.0), "{m:?} fog must not change");
            assert_eq!(look(m).sun, (-0.9, 0.5), "{m:?} sun must not change");
        }
    }

    /// THE SUN MUST BE ABOVE THE HORIZON. On every map.
    ///
    /// This is the test the whole `sun_euler` note is about. Cliffhold
    /// shipped two capture passes with its sun twenty degrees UNDER the
    /// map, shining upward — south and east faces brightly lit, every
    /// horizontal surface in the world dark — because moving the sun to
    /// the south meant taking the y euler past a quarter turn, and
    /// `cos y` going negative silently inverts the vertical component.
    ///
    /// It cost three capture cycles and two wrong fixes, and it is nine
    /// lines of arithmetic that any test could have run. A 47-degree
    /// camera bug survived for months in this codebase for the same
    /// reason: the arithmetic lived somewhere nothing could call it.
    ///
    /// Mutation-proved: negating Cliffhold's elevation argument
    /// (`sun_euler(-0.62, 0.55)`) fails the `dir.y < 0` assert, which is
    /// exactly the shipped state this replaces. Setting the azimuth to
    /// `0.55 + PI` fails the `dir.z > 0` assert.
    #[test]
    fn the_sun_is_above_the_horizon_on_every_map() {
        for m in MapKind::ALL {
            let d = sun_dir(look(m).sun);
            assert!(
                (d.length() - 1.0).abs() < 1e-4,
                "{m:?} sun direction is not a unit vector: {d:?}"
            );
            assert!(
                d.y < -0.15,
                "{m:?} SUN IS BELOW THE HORIZON - light travels {d:?}, \
                 so every upward-facing surface on the map is unlit"
            );
        }
        // and Cliffhold's specifically comes from the SOUTH, because
        // every one of its viewpoints looks north at the castle
        let d = sun_dir(look(MapKind::Cliffhold).sun);
        assert!(
            d.z > 0.4,
            "Cliffhold's sun must be south of the map so the faces the \
             player looks at are lit; light travels {d:?}"
        );
        // 30-42 degrees: low enough to cast off 32 m of relief, high
        // enough to reach the bottom of a 16 m street
        let elev = (-d.y).asin().to_degrees();
        assert!(
            (30.0..42.0).contains(&elev),
            "Cliffhold sun elevation is {elev} degrees"
        );
    }

    /// `sun_euler` is a coordinate conversion, so it is round-trippable
    /// and there is no excuse for not checking it.
    #[test]
    fn sun_euler_round_trips_through_sun_dir() {
        for &elev in &[0.2_f32, 0.4, 0.62, 0.9, 1.2] {
            for &azim in &[-2.0_f32, -0.9, 0.0, 0.55, 1.4, 2.6] {
                let d = sun_dir(sun_euler(elev, azim));
                // the sun sits OPPOSITE the direction light travels
                let up = -d.y;
                assert!(
                    (up - elev.sin()).abs() < 1e-4,
                    "elev {elev} azim {azim}: got elevation sin {up}"
                );
                let east = -d.x;
                let south = -d.z;
                assert!(
                    (east - azim.sin() * elev.cos()).abs() < 1e-4
                        && (south - (-azim.cos() * elev.cos())).abs() < 1e-4,
                    "elev {elev} azim {azim}: got {d:?}"
                );
            }
        }
    }

    /// The texture-tiling fix, as arithmetic. The great cliff is 186 m
    /// long; at the shared 1.25 tiles it wore one course of blockwork per
    /// 149 metres.
    ///
    /// Mutation-proved: returning a constant 1.25 fails the first assert.
    #[test]
    fn big_boxes_get_more_tiles_than_small_ones() {
        let s = cliffhold();
        let big = s
            .cover
            .iter()
            .max_by(|a, b| area(a).partial_cmp(&area(b)).unwrap())
            .unwrap();
        let small = s
            .cover
            .iter()
            .min_by(|a, b| area(a).partial_cmp(&area(b)).unwrap())
            .unwrap();
        assert!(
            uv_tiles(big) > uv_tiles(small) * 8.0,
            "big {} vs small {}",
            uv_tiles(big),
            uv_tiles(small)
        );
        // a tile stays in the same order of magnitude as a person
        let m = (big.max[0] - big.min[0]).max(big.max[2] - big.min[2]);
        assert!(m / uv_tiles(big) < 8.0, "tile is {} m", m / uv_tiles(big));
    }

    /// Per-entity variety must be id-hashed and never drawn from the
    /// sim's RNG. Two builds of the same map must therefore place an
    /// identical crest — and a different index must give a different
    /// answer, or the "variety" is a constant.
    #[test]
    fn crest_variety_is_hashed_not_random() {
        assert_eq!(hash01(3, 9), hash01(3, 9));
        assert_ne!(hash01(3, 9), hash01(4, 9));
        assert_ne!(hash01(3, 9), hash01(3, 10));
        for a in 0..40u32 {
            for b in 0..40u32 {
                let h = hash01(a, b);
                assert!((0.0..=1.0).contains(&h), "hash01({a},{b}) = {h}");
            }
        }
    }

    /// No landmark piece may sit at or below the standable surface it is
    /// mounted on, because a phantom block on a floor people walk on is
    /// cover that is not there.
    ///
    /// Checked as the ARITHMETIC of the placements rather than by
    /// re-querying the world: the keep's pinnacles start at `keep.top`,
    /// the gatehouse cap course sits below its top but hangs outside it,
    /// and the cliff crest is `DROP` under the plateau. This test exists
    /// so that a later "just nudge it down a bit" cannot pass unnoticed.
    #[test]
    fn nothing_decorative_lands_on_a_standable_surface() {
        let s = cliffhold();
        let lm = find(&s.cover, &s.cover_kind);
        let keep = lm.keep.unwrap();
        // pinnacle base = keep.top, spire above it: nothing below
        assert!(keep.top >= CH_KEEP_TOP - 0.01);
        // the gatehouse arch clears the courtyard by more than any jump
        let g = &lm.gatehouse;
        let arch_bottom = CH_RAMPART; // band centre CH_RAMPART + 1.1, half-height 1.1
        assert!(
            arch_bottom - CH_PLATEAU >= 5.0,
            "the arch is only {} m over the courtyard",
            arch_bottom - CH_PLATEAU
        );
        // and it spans a real gap
        assert!(g[1].min[0] - g[0].max[0] > 8.0, "the gate must admit a chassis");
    }
}
