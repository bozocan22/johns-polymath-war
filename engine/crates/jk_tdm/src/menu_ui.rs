//! The menu design system: one unit, one type scale, one plate, one row.
//!
//! Six surfaces (Splash, Intro, Paused, Settings, Manual, Controls) each
//! hand-built their own copy of the same Absolute/100%/Center/Column root
//! with the same COOL navy scrim at four different alphas — a colour that
//! is not in `branding::palette` and never was, sitting under art that is
//! warm gold and sepia. Everything here derives from the palette instead.
//!
//! ## Contrast, computed in LINEAR space
//!
//! sRGB-space figures run about 10% optimistic because that is not where
//! the GPU blends. Worst case ground below is the brightest pixel in
//! `key_art.png` (relative luminance .9975) after the `ART_TINT` multiply,
//! the scrim, and the plate:
//!
//! | ground                    | PARCHMENT | GOLD   | PARCHMENT_DIM |
//! |---------------------------|-----------|--------|---------------|
//! | plate                     | 12.5:1    | 6.9:1  | 6.3:1         |
//! | row hovered (BRONZE @.28) |  7.7:1    | 4.2:1  | —             |
//! | row selected (PLUME @.55) |  7.9:1    | 4.4:1  | —             |
//! | row pressed  (PLUME @.70) |  7.2:1    | —      | —             |
//! | keycap (SHADOW @.55)      | 13.4:1    | 7.4:1  | —             |
//! | rail   (SHADOW @.55)      |  8.0:1    | 4.4:1  | —             |
//! | bare art + scrim          |  5.2:1    | 2.9:1  | —             |
//!
//! ## Five laws that fall out of that table
//!
//! - **R1 — GOLD never touches art or a rail.** 4.4:1 at best, 2.9:1 on
//!   bare art, and it passes through 1.0:1 as the ground brightens. GOLD
//!   is confined to text on a plate, keycap glyphs, and drawn ornament.
//! - **R2 — text inside an interactive row is always PARCHMENT.** On a
//!   selected row only PARCHMENT clears AA (7.9:1 against GOLD's 4.4:1).
//!   Name and value separate by SIZE and ALIGNMENT, never by colour.
//! - **R3 — BRONZE and PLUME are ornament, never text.** BRONZE as ink on
//!   a plate is 3.3:1, below AA.
//! - **R4 — no text run ever sits on art or on a rail.** Rails carry the
//!   wordmark, the emblem and drawn ornament only. The 8.0:1 above is
//!   headroom deliberately not spent.
//! - **R5 — ink ladder is GOLD > PARCHMENT > PARCHMENT_DIM.** BRONZE and
//!   PLUME are structure, by name, not ink.
//!
//! ## ASCII only
//!
//! This crate ships `default_font` as its only font source and there is
//! no `assets/fonts`. Every Unicode thin space, en dash, bullet, arrow and
//! box-drawing character renders as tofu — five separate scars in this
//! file's history record it. Ornament is DRAWN (nodes with borders and
//! colour) or comes from a PNG. Never from a glyph.

use crate::branding;
use bevy::prelude::*;

// ---- scale ---------------------------------------------------------------

/// Every number in this module is authored at this window height.
pub const MENU_BASE_H: f32 = 720.0;
/// Quantise, so the bundled font never renders at a fractional size —
/// fractional sizes shimmer under hinting.
pub const MENU_SCALE_STEP: f32 = 0.25;
pub const MENU_SCALE_MIN: f32 = 1.0;
pub const MENU_SCALE_MAX: f32 = 3.5;

/// 720p -> 1.00, 1080p -> 1.50, 1440p -> 2.00, 4K -> 3.00.
///
/// DELIBERATE, NOT INCIDENTAL: this drives `Res<UiScale>`, which Bevy
/// multiplies into the layout scale factor AND into every
/// `TextFont.font_size` for the whole app — the gameplay HUD included.
/// That is the intent. A 17px ammo readout is unreadable at 4K.
/// `HUD_ANCHORS` are screen FRACTIONS, so the HUD's own layout test is
/// unaffected by this.
pub fn menu_ui_scale(window_h: f32) -> f32 {
    let raw = window_h / MENU_BASE_H;
    ((raw / MENU_SCALE_STEP).floor() * MENU_SCALE_STEP).clamp(MENU_SCALE_MIN, MENU_SCALE_MAX)
}

// ---- spacing -------------------------------------------------------------

/// The spacing scale. Every gap, pad and margin across the six surfaces is
/// one of these nine values. A spacing number not on this list is a bug.
pub const U: f32 = 4.0;
pub const U2: f32 = 8.0;
pub const U3: f32 = 12.0;
pub const U4: f32 = 16.0;
pub const U5: f32 = 20.0;
pub const U6: f32 = 24.0;
pub const U8: f32 = 32.0;
pub const U12: f32 = 48.0;
pub const U16: f32 = 64.0;

// ---- type ----------------------------------------------------------------

/// Five steps. Replaces the eleven near-indistinguishable sizes currently
/// in use (14/15/16/17/18/19/22/30/34/40/42). `TextFont` has no
/// `line_height` field in Bevy 0.15, so vertical rhythm comes from node
/// margins, never from leading.
pub const T_DISPLAY: f32 = 46.0; // surface title, one per screen
pub const T_HEAD: f32 = 28.0; // plate heading / primary action
pub const T_BODY: f32 = 17.0; // row names, prose
pub const T_DATA: f32 = 13.0; // values, table cells, keycaps
pub const T_MICRO: f32 = 11.0; // eyebrows, hints, footers

// ---- alphas (all over `branding::palette` colours) -----------------------

pub const PLATE_A: f32 = 0.92;
pub const SCRIM_A: f32 = 0.35; // SHADOW over the tinted art
pub const RAIL_A: f32 = 0.55; // SHADOW, top and bottom bands
pub const RAIL_H_PCT: f32 = 18.0; // percent of viewport height
pub const FRAME_INNER_A: f32 = 0.30; // GOLD, the engraved inner line
pub const RULE_SECTION_A: f32 = 0.35; // BRONZE, the eyebrow rule
pub const KEYCAP_A: f32 = 0.55; // SHADOW
pub const ROW_HOVER_A: f32 = 0.28; // BRONZE over the plate
pub const ROW_SELECT_A: f32 = 0.55; // PLUME over the plate
pub const ROW_PRESS_A: f32 = 0.70; // PLUME over the plate
pub const ROW_DANGER_A: f32 = 0.28; // PLUME, destructive hover

/// The key art is tinted by MULTIPLY, not washed by alpha.
///
/// `ImageNode.color` is a genuine per-pixel multiply. BRONZE drops the
/// art's peak luminance .9975 -> .1638, a 6x reduction, while PRESERVING
/// every internal ratio — so it stays a photograph rather than becoming a
/// grey field, and it lands inside the palette's own hue family. An alpha
/// scrim reaches the same darkness by collapsing every pixel toward flat,
/// which is exactly why the art can be technically drawn and perceptually
/// absent.
pub const ART_TINT: Color = branding::palette::BRONZE;

// ---- rule weights and ornament -------------------------------------------

pub const RULE_HAIR_PX: f32 = 1.0; // section rule, inner frame
pub const RULE_STAMP_PX: f32 = 2.0; // the standard, the outer frame
pub const RULE_KEEL_PX: f32 = 3.0; // an interactive row's left border
pub const BOSS_PX: f32 = 10.0; // the heading rule's boss
pub const BOSS_PIP_PX: f32 = 8.0; // the intro page pips
pub const BOSS_GUTTER_PX: f32 = 24.0; // fixed gutter the row boss sits in
pub const ROW_H: f32 = 36.0; // 9U
pub const BIND_ROW_H: f32 = 28.0; // 7U
pub const KEYCAP_MIN_W: f32 = 44.0;
pub const KEYCAP_H: f32 = 22.0;
pub const ROW_LABEL_W: f32 = 120.0; // the pick-row label gutter

/// How far the standard's ends project PAST the plate's outer edge.
///
/// From the key art: the vexillum's finials extend past its cloth, and
/// that overhang is what makes it read as a hanging standard rather than
/// a framed picture. It is also ornament a lazy `BorderColor` cannot
/// produce, so it cannot be arrived at by accident.
pub const RULE_PROJECT_PX: f32 = 12.0;
/// Distance from the inner frame's content box to the plate's outer edge.
/// DERIVED, never hand-typed, so changing a padding cannot silently
/// un-overhang the standard.
pub const PLATE_INSET_PX: f32 = U6 + RULE_HAIR_PX + U3 + RULE_STAMP_PX;
/// The negative side margin the standard carries.
pub const RULE_OVERHANG_PX: f32 = PLATE_INSET_PX + RULE_PROJECT_PX;

// ---- plate widths (max_width caps; width is always Percent(88)) ----------

pub const PLATE_W_PAUSE: f32 = 520.0;
pub const PLATE_W_INTRO: f32 = 760.0;
pub const PLATE_W_SETTINGS: f32 = 1000.0;
pub const PLATE_W_MANUAL: f32 = 1040.0;
pub const PLATE_W_CONTROLS: f32 = 1080.0;
pub const PLATE_MAX_H_PCT: f32 = 86.0;
/// Wrap width for a bind's action text. The longest registry action is 99
/// characters; unbounded it renders about 1300px wide in the game's own
/// 1280px window and overruns both edges.
pub const BIND_ACTION_W: f32 = 300.0;
pub const BIND_GROUP_W: f32 = 460.0;

// ---- art -----------------------------------------------------------------

pub const KEY_ART_ASPECT: f32 = 1920.0 / 1080.0;

/// Never render the emblem smaller than this.
///
/// Bevy generates no mipmaps for UI textures, so drawing a 1024px source
/// at 34px is one bilinear tap over a 30x30 texel footprint — 4 texels
/// sampled out of every 900, which aliases hard. `emblem_small.png` is a
/// LANCZOS reduction to 128px shipped for exactly these placements; use
/// it below ~128px and the full-size file above.
///
/// (This floor originally also guarded a white-halo defect: both logos
/// were authored flattened onto white, so their transparent pixels stored
/// near-white RGB that bilinear filtering dragged into every edge. That
/// is repaired at the asset level now — the flatten was inverted — so
/// what remains here is purely the sampling argument.)
pub const EMBLEM_MIN_PX: f32 = 34.0;
/// Above this, use the full-resolution emblem; below, the LANCZOS copy.
pub const EMBLEM_SMALL_MAX_PX: f32 = 128.0;

/// CSS `cover`, with the one mechanism Bevy UI has: constrain exactly ONE
/// axis and let the image's own measure derive the other from its aspect.
///
/// Setting both axes stretches. Setting one axis plus a `min_` on the
/// other ALSO stretches, because the clamp is applied after the aspect
/// ratio — which is the subtle version of the same bug.
pub fn key_art_fit(win_aspect: f32) -> (Val, Val) {
    if win_aspect >= KEY_ART_ASPECT {
        // window relatively wider: match width, overflow vertically
        (Val::Percent(100.0), Val::Auto)
    } else {
        // window relatively taller: match height, overflow horizontally
        (Val::Auto, Val::Percent(100.0))
    }
}

// ---- stacking ------------------------------------------------------------

/// The ONLY `GlobalZIndex` any menu node carries. 60 clears the flash
/// overlay (40), the health vignette (20), the emblem watermark (5) and
/// the menu-heading emblem placement (35).
pub const Z_MENU_SURFACE: i32 = 60;

/// Local `ZIndex` inside a surface. NEVER `GlobalZIndex`: Bevy's UI stack
/// lifts any entity carrying a `GlobalZIndex` out of its parent and treats
/// it as a root, which would let a child escape the surface it belongs to
/// and break the single-`despawn_recursive` teardown guarantee.
pub const ZL_ART: i32 = 0;
pub const ZL_SCRIM: i32 = 10;
pub const ZL_RAIL: i32 = 20;
pub const ZL_PLATE: i32 = 30;
pub const ZL_STANDARD: i32 = 40;
pub const ZL_MARK: i32 = 50;

// ---- colour helpers ------------------------------------------------------
// Tiny, so no spawn site ever writes an alpha literal.

#[inline]
pub fn shadow_a(a: f32) -> Color {
    branding::palette::SHADOW.with_alpha(a)
}
#[inline]
pub fn gold_a(a: f32) -> Color {
    branding::palette::GOLD.with_alpha(a)
}
#[inline]
pub fn bronze_a(a: f32) -> Color {
    branding::palette::BRONZE.with_alpha(a)
}
#[inline]
pub fn plume_a(a: f32) -> Color {
    branding::palette::PLUME.with_alpha(a)
}

// ---- text ----------------------------------------------------------------

/// ASCII letterspacing: one space between characters. Exact in a monospace
/// face, and a real typewriter convention for a heading.
///
/// This exists because the alternative — a Unicode thin space — is tofu
/// in this build. See the module header.
pub fn letterspaced(s: &str) -> String {
    s.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

// ---- the row painter -----------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RowKind {
    Normal,
    Destructive,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RowState {
    Idle,
    Hovered,
    Selected,
    Pressed,
}

pub fn row_state(selected: bool, i: Interaction) -> RowState {
    match (selected, i) {
        (_, Interaction::Pressed) => RowState::Pressed,
        (true, _) => RowState::Selected,
        (false, Interaction::Hovered) => RowState::Hovered,
        _ => RowState::Idle,
    }
}

/// The three colours a row's state is made of: `(fill, keel, boss)`.
///
/// Selection is a PLUME GROUND — the shield off the wall: an oxide-red
/// plate carrying a gold device — and PARCHMENT on it measures 7.9:1,
/// BETTER than a gold tint would give. Destruction is a PLUME BOSS.
/// Different organs, unmistakable, and PLUME never becomes text (R3).
pub fn row_colors(kind: RowKind, state: RowState) -> (Color, Color, Color) {
    use branding::palette::{BRONZE, GOLD, PLUME};
    match (kind, state) {
        (RowKind::Normal, RowState::Idle) => (Color::NONE, BRONZE, Color::NONE),
        (RowKind::Normal, RowState::Hovered) => (bronze_a(ROW_HOVER_A), GOLD, Color::NONE),
        (RowKind::Normal, RowState::Selected) => (plume_a(ROW_SELECT_A), GOLD, GOLD),
        (RowKind::Normal, RowState::Pressed) => (plume_a(ROW_PRESS_A), GOLD, GOLD),
        (RowKind::Destructive, RowState::Idle) => (Color::NONE, PLUME, Color::NONE),
        (RowKind::Destructive, _) => (plume_a(ROW_DANGER_A), PLUME, PLUME),
    }
}

// ---- tests ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // EVERY COMPOSITE BELOW IS DONE IN LINEAR SPACE, because that is
    // where the GPU blends. Doing it in sRGB and only converting at the
    // end is the easy mistake and it is optimistic by roughly 10-20%: an
    // earlier draft of this module composited in sRGB and scored GOLD at
    // 5.20:1 on a selected row, which passes AA and would have quietly
    // demolished R2's justification. In linear it is 4.38:1, which fails.
    // The law is real; the sRGB arithmetic was not.

    type Lin = [f32; 3];

    fn to_lin(c: Color) -> Lin {
        let s = c.to_srgba();
        let f = |v: f32| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        [f(s.red), f(s.green), f(s.blue)]
    }

    fn lum(l: Lin) -> f32 {
        0.2126 * l[0] + 0.7152 * l[1] + 0.0722 * l[2]
    }

    /// Composite `src` at its own alpha over an opaque linear ground.
    fn over(src: Color, dst: Lin) -> Lin {
        let a = src.to_srgba().alpha;
        let s = to_lin(src);
        [
            s[0] * a + dst[0] * (1.0 - a),
            s[1] * a + dst[1] * (1.0 - a),
            s[2] * a + dst[2] * (1.0 - a),
        ]
    }

    fn contrast_lin(ink: Color, ground: Lin) -> f32 {
        let (a, b) = (lum(to_lin(ink)), lum(ground));
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// The plate ground the laws are computed against: the brightest pixel
    /// in the key art, tinted by multiply, scrimmed, then plated.
    fn plate_ground() -> Lin {
        let art = to_lin(Color::srgb(0.9975, 0.9975, 0.9975));
        let t = to_lin(ART_TINT);
        // ImageNode.color is a per-channel multiply, and it happens on the
        // linear texel - not on an sRGB value.
        let tinted = [art[0] * t[0], art[1] * t[1], art[2] * t[2]];
        let scrimmed = over(shadow_a(SCRIM_A), tinted);
        over(branding::palette::SHADOW.with_alpha(PLATE_A), scrimmed)
    }

    /// R2 is the law most likely to be broken by someone reaching for gold
    /// on a selected row because it looks richer. It fails AA; parchment
    /// does not. Assert the ordering, from the real colours.
    #[test]
    fn r2_only_parchment_clears_aa_on_a_selected_row() {
        let ground = over(plume_a(ROW_SELECT_A), plate_ground());
        let parchment = contrast_lin(branding::palette::PARCHMENT, ground);
        let gold = contrast_lin(branding::palette::GOLD, ground);
        assert!(
            parchment >= 4.5,
            "PARCHMENT must clear AA on a selected row, got {parchment:.2}:1"
        );
        assert!(
            gold < 4.5,
            "GOLD must FAIL AA on a selected row - if this passes, R2's \
             whole justification is gone and the law should be revisited, \
             not quietly ignored. Got {gold:.2}:1"
        );
        assert!(parchment > gold, "the ladder must hold");
    }

    /// R3: BRONZE and PLUME are structure, not ink. Both fail as text on
    /// the plate, which is why `row_colors` never returns them for a text
    /// colour - only for keels and bosses.
    #[test]
    fn r3_bronze_and_plume_are_never_legible_enough_to_be_text() {
        let g = plate_ground();
        for (name, c) in [
            ("BRONZE", branding::palette::BRONZE),
            ("PLUME", branding::palette::PLUME),
        ] {
            let r = contrast_lin(c, g);
            assert!(r < 4.5, "{name} must not be usable as ink, got {r:.2}:1");
        }
        // and the two that ARE ink must clear it comfortably
        for (name, c) in [
            ("PARCHMENT", branding::palette::PARCHMENT),
            ("GOLD", branding::palette::GOLD),
            ("PARCHMENT_DIM", branding::palette::PARCHMENT_DIM),
        ] {
            let r = contrast_lin(c, g);
            assert!(r >= 4.5, "{name} must be usable as ink on a plate, got {r:.2}:1");
        }
    }

    /// R5: the ink ladder is an ordering, and it must actually order.
    #[test]
    fn r5_ink_ladder_is_ordered() {
        let g = plate_ground();
        let gold = contrast_lin(branding::palette::GOLD, g);
        let parch = contrast_lin(branding::palette::PARCHMENT, g);
        let dim = contrast_lin(branding::palette::PARCHMENT_DIM, g);
        assert!(parch > gold, "PARCHMENT reads harder than GOLD: {parch:.2} vs {gold:.2}");
        assert!(gold > dim, "GOLD reads harder than DIM: {gold:.2} vs {dim:.2}");
    }

    /// The scale must quantise, clamp, and never hand the layout a
    /// fractional step that makes the font shimmer.
    #[test]
    fn the_ui_scale_quantises_and_clamps() {
        assert_eq!(menu_ui_scale(720.0), 1.00, "720p is the authoring size");
        assert_eq!(menu_ui_scale(1080.0), 1.50);
        assert_eq!(menu_ui_scale(1440.0), 2.00);
        assert_eq!(menu_ui_scale(2160.0), 3.00);
        // below the authoring size the layout does not shrink - it would
        // fall under the minimum legible font size
        assert_eq!(menu_ui_scale(480.0), MENU_SCALE_MIN);
        assert_eq!(menu_ui_scale(100000.0), MENU_SCALE_MAX);
        // every result is a whole number of steps
        for h in [700.0, 900.0, 1234.0, 1600.0, 2400.0_f32] {
            let s = menu_ui_scale(h);
            let steps = s / MENU_SCALE_STEP;
            assert!(
                (steps - steps.round()).abs() < 1e-5,
                "scale {s} at {h}px is not a whole step"
            );
        }
    }

    /// Cover-fit constrains exactly ONE axis. Constraining both stretches,
    /// and that is the defect this function exists to avoid - so assert
    /// the shape of the answer, not just its value.
    #[test]
    fn key_art_cover_fit_constrains_exactly_one_axis() {
        for aspect in [4.0 / 3.0, 16.0 / 10.0, 16.0 / 9.0, 21.0 / 9.0, 32.0 / 9.0] {
            let (w, h) = key_art_fit(aspect);
            let fixed = (w != Val::Auto) as u8 + (h != Val::Auto) as u8;
            assert_eq!(fixed, 1, "aspect {aspect}: exactly one axis may be constrained");
        }
        // wider than the art -> match width; taller -> match height
        assert_eq!(key_art_fit(21.0 / 9.0).0, Val::Percent(100.0));
        assert_eq!(key_art_fit(21.0 / 9.0).1, Val::Auto);
        assert_eq!(key_art_fit(4.0 / 3.0).1, Val::Percent(100.0));
        assert_eq!(key_art_fit(4.0 / 3.0).0, Val::Auto);
        // exactly 16:9 must not fall in a gap
        assert_eq!(key_art_fit(KEY_ART_ASPECT).0, Val::Percent(100.0));
    }

    /// The overhang is derived. If someone retunes a padding, the standard
    /// must still project past the plate rather than silently tucking in.
    #[test]
    fn the_standard_always_projects_past_the_plate() {
        assert!(
            RULE_OVERHANG_PX > PLATE_INSET_PX,
            "the standard must overhang, not sit flush"
        );
        assert_eq!(RULE_OVERHANG_PX - PLATE_INSET_PX, RULE_PROJECT_PX);
    }

    /// Letterspacing must stay ASCII. A thin space here is tofu.
    #[test]
    fn letterspacing_is_ascii_only() {
        let s = letterspaced("ARENA");
        assert_eq!(s, "A R E N A");
        assert!(s.is_ascii(), "no Unicode spacing may leak in: {s:?}");
        assert_eq!(letterspaced(""), "");
        assert_eq!(letterspaced("X"), "X");
    }

    /// A destructive row must never be mistakable for a normal one in any
    /// state - that is the whole point of the kind.
    #[test]
    fn destructive_rows_never_paint_like_normal_ones() {
        for st in [
            RowState::Idle,
            RowState::Hovered,
            RowState::Selected,
            RowState::Pressed,
        ] {
            let n = row_colors(RowKind::Normal, st);
            let d = row_colors(RowKind::Destructive, st);
            assert_ne!(n.1, d.1, "the keel must differ in state {st:?}");
        }
        // idle rows carry no fill - a menu of seven filled bars is a wall
        assert_eq!(row_colors(RowKind::Normal, RowState::Idle).0, Color::NONE);
        assert_eq!(row_colors(RowKind::Destructive, RowState::Idle).0, Color::NONE);
        // and only a selected/pressed row shows a boss
        assert_eq!(row_colors(RowKind::Normal, RowState::Hovered).2, Color::NONE);
        assert_ne!(row_colors(RowKind::Normal, RowState::Selected).2, Color::NONE);
    }

    /// `row_state` resolves the three inputs in the right priority. Pressed
    /// beats selected beats hovered - a selected row being clicked must
    /// show the press, or the click has no feedback.
    #[test]
    fn row_state_priority_is_pressed_then_selected_then_hovered() {
        assert_eq!(row_state(true, Interaction::Pressed), RowState::Pressed);
        assert_eq!(row_state(false, Interaction::Pressed), RowState::Pressed);
        assert_eq!(row_state(true, Interaction::Hovered), RowState::Selected);
        assert_eq!(row_state(true, Interaction::None), RowState::Selected);
        assert_eq!(row_state(false, Interaction::Hovered), RowState::Hovered);
        assert_eq!(row_state(false, Interaction::None), RowState::Idle);
    }

    /// The spacing scale is a scale. If someone adds a value that is not a
    /// multiple of the base unit, the rhythm is gone.
    #[test]
    fn the_spacing_scale_is_a_real_scale() {
        for v in [U, U2, U3, U4, U5, U6, U8, U12, U16] {
            assert!((v / U - (v / U).round()).abs() < 1e-6, "{v} is not a multiple of U");
        }
        // and it ascends
        let all = [U, U2, U3, U4, U5, U6, U8, U12, U16];
        for w in all.windows(2) {
            assert!(w[1] > w[0], "the scale must ascend: {:?}", w);
        }
    }
}
