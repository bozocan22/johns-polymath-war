//! THE SHAPES A SCALED CUBE CANNOT MAKE.
//!
//! # The one thing every hand in this game still got wrong
//!
//! Three hand builders exist (`spawn_hand_fingered`,
//! `spawn_world_hand_fingered`, `spawn_hand`) and a previous pass fixed
//! the LAYOUT properly: four different finger lengths, a knuckle arc, a
//! per-finger gauge, an outward fan, rounded fingertip pads. `VM_FINGERS`
//! and `WORLD_FINGERS` are that pass, and `hand_craft_tests` states it.
//!
//! What none of it could fix is that a finger SEGMENT is a scaled
//! `Cuboid`, and a scaled cuboid is a prism: exactly as wide at the tip
//! as at the root. The taper the old comments describe is a taper
//! BETWEEN segments (the distal box is 0.85 the gauge of the proximal
//! one), so a finger is a stack of two parallel-sided blocks with a step
//! in the middle - and a parallel-sided block with a rounded cap on the
//! end is the definition of the sausage the whole pass was trying to
//! stop drawing.
//!
//! [`taper_mesh`] is a unit box whose +Z face is smaller than its -Z
//! face. It is a DROP-IN for `kit.cube` at every finger segment: the
//! segments are already placed as `from_xyz(0, 0, len * 0.5)` with
//! `scale = (w, h, len)`, i.e. spanning local Z from 0 to `len`, so
//! swapping the mesh handle tapers the finger and moves nothing. Zero
//! extra entities, on a mesh that is built once and shared by every hand
//! in the game.
//!
//! # Why a mesh and not more boxes
//!
//! The alternative - stacking three short boxes at decreasing widths -
//! costs two entities per segment, twenty per hand, and still steps
//! rather than tapers. The world hand is spawned per PLAYER BODY and the
//! §0.3 per-fighter budget is real; a shared mesh costs nothing per
//! instance and is the only version of this that both hands can afford.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

/// How wide a finger segment's TIP is as a fraction of its root.
///
/// 0.74 rather than something subtler because of the distance this is
/// read at. The old inter-segment step was 0.85 and the comment beside
/// it says, correctly, that at 30 cm from the lens 0.91 "is not a taper,
/// it is a manufacturing tolerance". The same argument applies to the
/// segment itself: a proximal phalanx really does lose about a quarter
/// of its width between the knuckle and the joint above it.
pub const SEGMENT_TAPER: f32 = 0.74;

/// A slighter taper for the DISTAL segment.
///
/// The distal phalanx is the one that ends in a pad, and the pad is a
/// ball sized off the segment's ROOT width. Tapering the distal segment
/// as hard as the proximal one leaves the pad standing proud of the
/// shaft it caps - a bulb on a stick - which is worse than no taper. The
/// pad scale in both spawn functions is ~0.84 of the segment's own
/// gauge, so 0.86 keeps the shaft just inside it.
pub const TIP_TAPER: f32 = 0.86;

/// A unit FRUSTUM along +Z: 1 x 1 at `z = -0.5`, `k` x `k` at
/// `z = +0.5`, centred on the origin.
///
/// Same extents as `Cuboid::new(1.0, 1.0, 1.0)` at the wide end, so a
/// transform written for the cube frames the same silhouette at the
/// root and narrows from there. Flat-shaded (24 vertices, no shared
/// normals) because every hand part in this game is, and a smoothed
/// finger among faceted ones would read as a different material.
///
/// `k` must be > 0; a zero would collapse the far face into a degenerate
/// quad whose normal cannot be computed. Clamped rather than asserted -
/// this runs at startup and a panic there costs the whole game.
pub fn taper_mesh(k: f32) -> Mesh {
    let k = k.clamp(0.02, 4.0);
    let (a, b) = (0.5_f32, 0.5 * k);
    // the eight corners: back face (z = -0.5) then front face (z = +0.5),
    // both counter-clockwise seen from OUTSIDE their own face
    let back = [
        Vec3::new(-a, -a, -0.5),
        Vec3::new(a, -a, -0.5),
        Vec3::new(a, a, -0.5),
        Vec3::new(-a, a, -0.5),
    ];
    let front = [
        Vec3::new(-b, -b, 0.5),
        Vec3::new(b, -b, 0.5),
        Vec3::new(b, b, 0.5),
        Vec3::new(-b, b, 0.5),
    ];
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(24);
    let mut nrm: Vec<[f32; 3]> = Vec::with_capacity(24);
    let mut uv: Vec<[f32; 2]> = Vec::with_capacity(24);
    let mut idx: Vec<u32> = Vec::with_capacity(36);
    // One quad, given its four corners in counter-clockwise order as
    // seen from outside. The normal is CROSSED from the corners rather
    // than typed: the four side faces of a frustum are slanted, and
    // every one of their normals is a different vector that depends on
    // `k`. Typing them is how a lighting bug gets shipped.
    let mut quad = |p: [Vec3; 4]| {
        let n = (p[1] - p[0]).cross(p[3] - p[0]).normalize_or_zero();
        let base = pos.len() as u32;
        for (i, v) in p.iter().enumerate() {
            pos.push([v.x, v.y, v.z]);
            nrm.push([n.x, n.y, n.z]);
            uv.push([(i == 1 || i == 2) as u8 as f32, (i >= 2) as u8 as f32]);
        }
        idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    };
    // front (+Z, the narrow cap) and back (-Z, the wide root)
    quad([front[0], front[1], front[2], front[3]]);
    quad([back[1], back[0], back[3], back[2]]);
    // The four slanted flanks. The two X faces were wound the other way
    // round in the first cut and `every_face_normal_points_outward`
    // caught it: their normals came out at -0.99 on X, i.e. both sides
    // of every finger segment in the game lit as if they faced inward.
    // That is not a visible-as-an-error kind of bug - it reads as an
    // odd dark band down the side of a finger, which is exactly the
    // sort of thing that gets mistaken for shading and shipped.
    quad([back[1], back[2], front[2], front[1]]); // +X
    quad([back[0], front[0], front[3], back[3]]); // -X
    quad([back[0], back[1], front[1], front[0]]); // -Y
    quad([back[3], front[3], front[2], back[2]]); // +Y
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    m.insert_indices(Indices::U32(idx));
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::mesh::VertexAttributeValues;

    fn positions(m: &Mesh) -> Vec<[f32; 3]> {
        match m.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("positions are not Float32x3"),
        }
    }
    fn normals(m: &Mesh) -> Vec<[f32; 3]> {
        match m.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
            VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("normals are not Float32x3"),
        }
    }

    /// THE WHOLE POINT: the +Z end is narrower than the -Z end, and the
    /// -Z end still measures exactly what a unit cube measures - so a
    /// transform written for `kit.cube` still frames the same root.
    ///
    /// Fails on a cube (which is what every finger segment was): a cube's
    /// two ends are the same width, so `far < near` is false for it.
    #[test]
    fn the_far_end_is_narrower_and_the_near_end_is_unit() {
        let m = taper_mesh(SEGMENT_TAPER);
        let p = positions(&m);
        let near = p
            .iter()
            .filter(|v| v[2] < 0.0)
            .fold(0.0_f32, |a, v| a.max(v[0].abs()));
        let far = p
            .iter()
            .filter(|v| v[2] > 0.0)
            .fold(0.0_f32, |a, v| a.max(v[0].abs()));
        assert!((near - 0.5).abs() < 1e-5, "root half-width is {near}, not 0.5");
        assert!(
            (far - 0.5 * SEGMENT_TAPER).abs() < 1e-5,
            "tip half-width is {far}, expected {}",
            0.5 * SEGMENT_TAPER
        );
        assert!(far < near, "the segment does not taper at all");
    }

    /// Every normal is unit length and points AWAY from the axis it
    /// belongs to - i.e. outward. A slanted flank's normal is not any of
    /// the six axis vectors, which is exactly why it is crossed rather
    /// than typed; this pins the sign.
    #[test]
    fn every_face_normal_points_outward() {
        let m = taper_mesh(SEGMENT_TAPER);
        let (p, n) = (positions(&m), normals(&m));
        assert_eq!(p.len(), 24, "expected 24 flat-shaded vertices");
        for (v, nv) in p.iter().zip(n.iter()) {
            let nn = Vec3::from_array(*nv);
            assert!((nn.length() - 1.0).abs() < 1e-4, "normal {nv:?} is not unit");
            // outward: from the solid's centre (the origin) toward the
            // vertex, the normal must not point back inward
            let out = Vec3::from_array(*v);
            assert!(
                out.dot(nn) > 0.0,
                "normal {nv:?} at {v:?} faces into the solid"
            );
        }
    }

    /// A frustum with `k == 1` IS the unit cube, to the last decimal.
    /// This is the no-op guarantee: if a segment is ever built with no
    /// taper it must be pixel-identical to what shipped.
    #[test]
    fn k_of_one_is_exactly_the_unit_cube() {
        let p = positions(&taper_mesh(1.0));
        for v in p {
            assert!(
                (v[0].abs() - 0.5).abs() < 1e-6
                    && (v[1].abs() - 0.5).abs() < 1e-6
                    && (v[2].abs() - 0.5).abs() < 1e-6,
                "{v:?} is not a corner of the unit cube"
            );
        }
    }
}
