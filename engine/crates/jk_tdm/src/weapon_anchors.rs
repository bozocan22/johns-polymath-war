//! NAMED HAND ANCHORS on the bow and the spear.
//!
//! # The contract
//!
//! Every bow and spear model built by `spawn_weapon_model` - the
//! player's viewmodel one AND every body's - carries one child entity
//! per anchor listed in `anchors_for`. Each child has:
//!
//! * a [`WeaponAnchor`] component naming which anchor it is,
//! * a `Transform` in the WEAPON MODEL's own local space, and
//! * therefore a `GlobalTransform` that is already in world space.
//!
//! A hand system attaches in one of two ways, both supported:
//!
//! 1. **Read it.** Query `(&WeaponAnchor, &GlobalTransform)`, filter for
//!    the kind you want, and drive an IK target at its translation. This
//!    is the right way for a hand that belongs to a BODY rig, because
//!    the hand stays parented to the arm.
//! 2. **Parent to it.** `commands.entity(hand).set_parent(anchor)`. The
//!    hand then rides the anchor for free, including the live ones.
//!
//! The live ones matter: `BowNock` and `BowDrawHand` MOVE, every frame,
//! with the draw. They are re-posed by `bow_string_sync` from the same
//! `bow_nock_local` the string and the nocked arrow use, so a hand on
//! `BowDrawHand` and the string it is holding cannot disagree - which is
//! precisely the bug the string, the arrow and the body's hand each
//! shipped their own copy of before `bow_string_sync` existed.
//!
//! There are no anchors on any other weapon yet. `anchors_for` returns
//! an empty slice for them rather than a guess, so a hand system can
//! tell "this weapon has no published grip" from "the grip is at the
//! origin".
//!
//! # What this module does NOT do
//!
//! It does not model, pose, or own a hand. It publishes where a hand
//! WOULD go. Everything here is cosmetic: no anchor transform is ever
//! read by the sim.

use bevy::prelude::*;

/// The published anchors. Named for the part of the weapon, not for the
/// limb - the spear's grip is the spear's grip whether the thrower is
/// left- or right-handed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorKind {
    /// THE BOW HAND. The leather wrap at the centre of the riser -
    /// where the bow is held, not where it is drawn. Static.
    BowGrip,
    /// THE NOCK ITSELF: the point on the string the arrow's tail sits
    /// in. LIVE - it travels `BOW_DRAW_PULL` back over a full draw.
    ///
    /// This is the string's point, so a hand placed exactly here would
    /// occupy the string rather than hook it. Use `BowDrawHand` for a
    /// hand; use this one for anything that belongs ON the string.
    BowNock,
    /// THE DRAW HAND: the nock plus `BOW_HAND_OFF` - fingers hooked
    /// behind and slightly outboard of the string. LIVE, same clock as
    /// `BowNock`.
    BowDrawHand,
    /// THE SPEAR GRIP. The swell at z = 0 on the shaft, which is both
    /// the balance point and the middle of the three shaft runs
    /// `spear_profile` is deliberately split into. Static in the
    /// weapon's own space; the whole weapon is what the javelin wind
    /// moves.
    SpearGrip,
}

impl AnchorKind {
    /// The stable string name, for logs and for anything that has to
    /// match an anchor by text rather than by enum.
    pub fn name(self) -> &'static str {
        match self {
            AnchorKind::BowGrip => "bow.grip",
            AnchorKind::BowNock => "bow.nock",
            AnchorKind::BowDrawHand => "bow.draw_hand",
            AnchorKind::SpearGrip => "spear.grip",
        }
    }

    /// Does this anchor move with the draw? `false` means the spawn
    /// transform is the whole story and nothing has to update it.
    pub fn is_live(self) -> bool {
        matches!(self, AnchorKind::BowNock | AnchorKind::BowDrawHand)
    }
}

/// A published attachment point on a weapon model. Sits on a child of
/// the model root; its `Transform` is in the model root's local space.
#[derive(Component, Clone, Copy, Debug)]
pub struct WeaponAnchor(pub AnchorKind);

/// Which anchors a weapon publishes. Empty for anything not yet
/// covered - an empty list is an honest answer, a `[0,0,0]` grip is not.
///
/// Keyed by a discriminant rather than by `GunKind` so this file does
/// not have to depend on the sim's weapon enum.
pub fn anchors_for_bow() -> &'static [AnchorKind] {
    &[AnchorKind::BowGrip, AnchorKind::BowNock, AnchorKind::BowDrawHand]
}

/// See `anchors_for_bow`.
pub fn anchors_for_spear() -> &'static [AnchorKind] {
    &[AnchorKind::SpearGrip]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every anchor has a distinct stable name - a hand system matching
    /// by text must not be able to hit two of them.
    #[test]
    fn the_anchor_names_are_unique_and_namespaced() {
        let all = [
            AnchorKind::BowGrip,
            AnchorKind::BowNock,
            AnchorKind::BowDrawHand,
            AnchorKind::SpearGrip,
        ];
        let mut seen: Vec<&str> = Vec::new();
        for a in all {
            assert!(a.name().contains('.'), "{:?} is not namespaced", a);
            assert!(!seen.contains(&a.name()), "duplicate anchor name {}", a.name());
            seen.push(a.name());
        }
    }

    /// The two that move must SAY they move - `bow_string_sync` only
    /// re-poses the live ones, so a mis-flagged anchor would silently
    /// freeze at its spawn pose.
    #[test]
    fn exactly_the_draw_anchors_are_live() {
        assert!(AnchorKind::BowNock.is_live());
        assert!(AnchorKind::BowDrawHand.is_live());
        assert!(!AnchorKind::BowGrip.is_live());
        assert!(!AnchorKind::SpearGrip.is_live());
        // and the bow publishes every live one it has
        assert!(anchors_for_bow().iter().filter(|a| a.is_live()).count() == 2);
        assert!(anchors_for_spear().iter().all(|a| !a.is_live()));
    }
}
