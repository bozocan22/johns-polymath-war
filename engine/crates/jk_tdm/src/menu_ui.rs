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
/// Option pills are shorter than action rows. The soldier page stacks
/// SEVEN of them plus a spec readout, a nav row, pips and a seal, and
/// at 36 each that column overran the plate.
pub const PILL_H: f32 = 28.0; // 7U
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

// ---- components ----------------------------------------------------------

/// Every menu surface root. One marker, so a surface can be found without
/// knowing which screen it is.
#[derive(Component)]
pub struct MenuSurface;

/// The clip frame the key art is cover-fitted inside.
#[derive(Component)]
pub struct ArtFrame;

/// The key art image itself. `key_art_refit` re-picks its constrained
/// axis on resize.
#[derive(Component)]
pub struct KeyArtImage;

/// The plaque.
#[derive(Component)]
pub struct MenuPlate;

/// An interactive row. Carries only its KIND - the boss is found through
/// `Children`, because `ChildBuilder::spawn` borrows a temporary and the
/// boss entity does not exist until after the row's closure has run, so
/// there is no point at which its id could be stored here.
#[derive(Component)]
pub struct PlateRow {
    pub kind: RowKind,
}

/// The square stamp inside a row's left gutter.
#[derive(Component)]
pub struct RowBoss;

// ---- 1. the ground -------------------------------------------------------

fn full_bleed() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// Spawn a surface root and its three ground layers, returning the root
/// so the caller can attach its own marker and its plate.
///
/// Replaces five hand-typed roots that each re-declared the same
/// Absolute/100%/Center/Column node and the same cool navy scrim at four
/// different alphas.
pub fn spawn_surface(
    commands: &mut Commands,
    brand: Option<&branding::BrandAssets>,
    win_aspect: f32,
) -> Entity {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(Z_MENU_SURFACE),
            MenuSurface,
        ))
        .id();

    commands.entity(root).with_children(|p| {
        // L0 ART. A clip frame with one cover-fitted child. The frame's
        // own background is the non-fatal fallback the branding contract
        // promises: an empty assets/branding/ still boots, and still
        // lands on a warm ground rather than a hole.
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(branding::palette::SHADOW),
            ZIndex(ZL_ART),
            ArtFrame,
        ))
        .with_children(|f| {
            if let Some(b) = brand {
                let (w, h) = key_art_fit(win_aspect);
                f.spawn((
                    Node {
                        width: w,
                        height: h,
                        // MANDATORY. Node defaults flex_shrink to 1.0, and
                        // in the tall-window branch the art's main axis is
                        // Auto and larger than the frame - so flex would
                        // shrink it back to fit and the "cover" becomes a
                        // squash, which is the exact bug this recipe
                        // exists to avoid.
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ImageNode {
                        image: b.key_art.clone(),
                        color: ART_TINT,
                        ..default()
                    },
                    KeyArtImage,
                ));
            }
        });
        // L1 SCRIM
        p.spawn((
            full_bleed(),
            BackgroundColor(shadow_a(SCRIM_A)),
            ZIndex(ZL_SCRIM),
        ));
        // L2 RAILS, top and bottom - where the art is brightest, and
        // where the wordmark and the seal sit.
        for top in [true, false] {
            let mut n = Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Vh(RAIL_H_PCT),
                ..default()
            };
            if top {
                n.top = Val::Px(0.0);
            } else {
                n.bottom = Val::Px(0.0);
            }
            p.spawn((n, BackgroundColor(shadow_a(RAIL_A)), ZIndex(ZL_RAIL)));
        }
    });
    root
}

/// Re-pick the constrained axis on resize, so the cover-fit survives a
/// window that changes aspect.
pub fn key_art_refit(
    mut ev: EventReader<bevy::window::WindowResized>,
    mut q: Query<&mut Node, With<KeyArtImage>>,
) {
    let Some(last) = ev.read().last() else { return };
    if last.height <= 0.0 {
        return;
    }
    let (w, h) = key_art_fit(last.width / last.height);
    for mut n in &mut q {
        n.width = w;
        n.height = h;
    }
}

// ---- 2. the plate --------------------------------------------------------

/// The plaque: a two-line engraved frame - 2px bronze outer, 1px gold
/// inner, U3 of dark between - with square corners throughout. `body`
/// receives the INNER frame's builder.
///
/// `overflow` is deliberately left visible: the standard overhangs the
/// plate by `RULE_OVERHANG_PX` and a clip would guillotine it.
pub fn plate(p: &mut ChildBuilder, max_w: f32, body: impl FnOnce(&mut ChildBuilder)) {
    p.spawn((
        Node {
            width: Val::Percent(88.0),
            max_width: Val::Px(max_w),
            max_height: Val::Percent(PLATE_MAX_H_PCT),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(RULE_STAMP_PX)),
            padding: UiRect::all(Val::Px(U3)),
            ..default()
        },
        BackgroundColor(shadow_a(PLATE_A)),
        BorderColor(branding::palette::BRONZE),
        BorderRadius::ZERO,
        ZIndex(ZL_PLATE),
        MenuPlate,
    ))
    .with_children(|outer| {
        outer
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(RULE_HAIR_PX)),
                    padding: UiRect::axes(Val::Px(U6), Val::Px(U5)),
                    // Clip the Y axis only.
                    //
                    // `max_height` caps the plate, but `UiScale` grows the
                    // CONTENT with the window while that cap stays a fixed
                    // percentage - so past about 900px tall the densest
                    // page overran its own frame and the seal footer drew
                    // OUTSIDE the plate border. Clipping vertically makes
                    // the frame a real boundary instead of a suggestion.
                    //
                    // X stays visible on purpose: the standard overhangs
                    // by RULE_OVERHANG_PX and clipping both axes would
                    // guillotine it.
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BorderColor(gold_a(FRAME_INNER_A)),
                BackgroundColor(Color::NONE),
                BorderRadius::ZERO,
            ))
            .with_children(body);
    });
}

/// The surface's own name. One per screen, GOLD, on a plate (R1).
pub fn title(p: &mut ChildBuilder, text: &str) {
    p.spawn((
        Text::new(letterspaced(text)),
        TextFont {
            font_size: T_DISPLAY,
            ..default()
        },
        TextColor(branding::palette::GOLD),
        Node {
            margin: UiRect::bottom(Val::Px(U2)),
            ..default()
        },
    ));
}

// ---- 3. the signature: rule + boss ---------------------------------------

/// A gold rule interrupted at dead centre by a square boss, both ends
/// projecting past the plate's outer edge.
///
/// Lifted from the art, not invented: the wordmark's own ink is a long
/// thin gold bar broken at centre by the laurel-gear-star, and the key
/// art's shield wall is a locked row of plates each carrying one bright
/// boss. Rule plus centre stamp is the whole visual idea of the artwork,
/// reproducible in four nodes.
///
/// The overhang comes from leaving `width` UNSET and letting the column
/// parent's default cross-axis stretch size it. Setting `width: 100%`
/// with negative margins yields an ASYMMETRIC overhang instead.
pub fn rule_and_boss(p: &mut ChildBuilder, filled: bool) {
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        height: Val::Px(BOSS_PX),
        column_gap: Val::Px(U2),
        margin: UiRect {
            left: Val::Px(-RULE_OVERHANG_PX),
            right: Val::Px(-RULE_OVERHANG_PX),
            top: Val::Px(U4),
            bottom: Val::Px(U3),
        },
        ..default()
    })
    .with_children(|r| {
        for i in 0..3 {
            if i == 1 {
                r.spawn((
                    Node {
                        width: Val::Px(BOSS_PX),
                        height: Val::Px(BOSS_PX),
                        border: UiRect::all(Val::Px(RULE_STAMP_PX)),
                        ..default()
                    },
                    BorderColor(branding::palette::GOLD),
                    BackgroundColor(if filled {
                        branding::palette::GOLD
                    } else {
                        Color::NONE
                    }),
                    BorderRadius::ZERO,
                ));
            } else {
                r.spawn((
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(RULE_STAMP_PX),
                        ..default()
                    },
                    BackgroundColor(branding::palette::GOLD),
                ));
            }
        }
    });
}

/// The signature's minor key: the same rule broken by a WORD rather than
/// a mark, running to the plate edge only. Groups sections, and replaces
/// the bare `"\n"` used as a section break today.
pub fn eyebrow(p: &mut ChildBuilder, label: &str) {
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(U3),
        margin: UiRect {
            top: Val::Px(U6),
            bottom: Val::Px(U3),
            ..default()
        },
        ..default()
    })
    .with_children(|r| {
        r.spawn((
            Text::new(letterspaced(label)),
            TextFont {
                font_size: T_MICRO,
                ..default()
            },
            TextColor(branding::palette::PARCHMENT_DIM),
        ));
        r.spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Px(RULE_HAIR_PX),
                ..default()
            },
            BackgroundColor(bronze_a(RULE_SECTION_A)),
        ));
    });
}

// ---- 4. the row ----------------------------------------------------------

/// A keycap. Gold glyph on a dark cap - 7.4:1, well clear of AA.
pub fn keycap(p: &mut ChildBuilder, key: &str, essential: bool) {
    p.spawn((
        Node {
            min_width: Val::Px(KEYCAP_MIN_W),
            height: Val::Px(KEYCAP_H),
            border: UiRect::all(Val::Px(RULE_HAIR_PX)),
            padding: UiRect::horizontal(Val::Px(U2)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(if essential {
            branding::palette::GOLD
        } else {
            branding::palette::BRONZE
        }),
        BackgroundColor(shadow_a(KEYCAP_A)),
        BorderRadius::ZERO,
    ))
    .with_children(|c| {
        c.spawn((
            Text::new(key.to_string()),
            TextFont {
                font_size: T_DATA,
                ..default()
            },
            TextColor(branding::palette::GOLD),
        ));
    });
}

/// One interactive row: `[boss gutter][name ..grows.. value][keycap]`,
/// keeled on its left edge. THE one shape behind the pause actions, the
/// settings rows, the intro pills and the nav buttons.
///
/// Returns `(row, boss, value)` so a caller can attach a label component
/// to the VALUE node only - the settings refresh query is unscoped, so a
/// label on two nodes per row would write the same string into both.
pub fn menu_row(
    p: &mut ChildBuilder,
    tag: impl Bundle,
    kind: RowKind,
    name: &str,
    value: Option<&str>,
    keycap_hint: Option<&str>,
) -> Entity {
    p.spawn((
        Button,
        tag,
        PlateRow { kind },
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Px(ROW_H),
            padding: UiRect::horizontal(Val::Px(U3)),
            column_gap: Val::Px(U3),
            border: UiRect::left(Val::Px(RULE_KEEL_PX)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor(branding::palette::BRONZE),
        BorderRadius::ZERO,
    ))
    .with_children(|r| {
        // The boss is a DIRECT child, so the painter finds it with one
        // `Children` hop rather than two.
        r.spawn((
            Node {
                width: Val::Px(BOSS_PX),
                height: Val::Px(BOSS_PX),
                margin: UiRect::right(Val::Px(BOSS_GUTTER_PX - BOSS_PX)),
                border: UiRect::all(Val::Px(RULE_STAMP_PX)),
                flex_shrink: 0.0,
                ..default()
            },
            BorderColor(branding::palette::BRONZE),
            BackgroundColor(Color::NONE),
            BorderRadius::ZERO,
            RowBoss,
        ));
        // name - R2: PARCHMENT, always
        r.spawn((
            Text::new(name.to_string()),
            TextFont { font_size: T_BODY, ..default() },
            TextColor(branding::palette::PARCHMENT),
            Node { flex_grow: 1.0, ..default() },
        ));
        // value - R2 again: same ink, smaller size, right edge
        if let Some(v) = value {
            r.spawn((
                ValueCell,
                Text::new(v.to_string()),
                TextFont { font_size: T_DATA, ..default() },
                TextColor(branding::palette::PARCHMENT),
            ));
        }
        if let Some(k) = keycap_hint {
            keycap(r, k, false);
        }
    })
    .id()
}

/// The value half of a row, tagged so a caller can find it without a
/// second component on the name node - the settings refresh query is
/// unscoped, and a label on both would write the same string into both.
#[derive(Component)]
pub struct ValueCell;

/// A row of option pills: a fixed label gutter followed by N pills that
/// each grow, so a row of two options and a row of six both fill the
/// plate exactly.
///
/// Replaces the old `pick_row`, whose `w: f32` width argument every call
/// site had to guess at, and whose spawn-time colour was a cool blue-grey
/// overwritten on frame 1 - misleading anyone who read the spawn site to
/// find the row's real colour.
///
/// The label is PARCHMENT_DIM, not BRONZE: bronze ink measures 3.3:1 on a
/// plate, below AA (R3). That was the old colour for every one of these.
pub fn pill_row<C: Component + Copy>(
    p: &mut ChildBuilder,
    label: &str,
    items: &[(&str, C)],
    page_tag: impl Bundle,
) {
    p.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(U2),
            margin: UiRect::bottom(Val::Px(U2)),
            width: Val::Percent(100.0),
            ..default()
        },
        page_tag,
    ))
    .with_children(|r| {
        r.spawn((
            Text::new(label.to_string()),
            TextFont {
                font_size: T_MICRO,
                ..default()
            },
            TextColor(branding::palette::PARCHMENT_DIM),
            Node {
                width: Val::Px(ROW_LABEL_W),
                flex_shrink: 0.0,
                ..default()
            },
        ));
        for (name, comp) in items {
            r.spawn((
                Button,
                *comp,
                PlateRow { kind: RowKind::Normal },
                Node {
                    flex_grow: 1.0,
                    // flex_basis 0 so every pill in the row is the same
                    // width regardless of its label's length
                    flex_basis: Val::Px(0.0),
                    height: Val::Px(PILL_H),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(U2),
                    border: UiRect::left(Val::Px(RULE_KEEL_PX)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor(branding::palette::BRONZE),
                BorderRadius::ZERO,
            ))
            .with_children(|b| {
                b.spawn((
                    Node {
                        width: Val::Px(BOSS_PIP_PX),
                        height: Val::Px(BOSS_PIP_PX),
                        border: UiRect::all(Val::Px(RULE_HAIR_PX)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    BorderColor(branding::palette::BRONZE),
                    BackgroundColor(Color::NONE),
                    BorderRadius::ZERO,
                    RowBoss,
                ));
                b.spawn((
                    Text::new(name.to_string()),
                    TextFont {
                        font_size: T_DATA,
                        ..default()
                    },
                    TextColor(branding::palette::PARCHMENT),
                ));
            });
        }
    });
}

/// The page pips: the row boss again, smaller, filled on the current
/// page. A progress indicator for free, in a vocabulary the player has
/// already been taught by the heading rule.
#[derive(Component)]
pub struct PagePip(pub u8);

pub fn page_pips(p: &mut ChildBuilder, count: u8) {
    p.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(U2),
        margin: UiRect::top(Val::Px(U4)),
        ..default()
    })
    .with_children(|r| {
        for i in 0..count {
            r.spawn((
                Node {
                    width: Val::Px(BOSS_PIP_PX),
                    height: Val::Px(BOSS_PIP_PX),
                    border: UiRect::all(Val::Px(RULE_HAIR_PX)),
                    ..default()
                },
                BorderColor(branding::palette::BRONZE),
                BackgroundColor(Color::NONE),
                BorderRadius::ZERO,
                PagePip(i),
            ));
        }
    });
}

// ---- 5. the seal ---------------------------------------------------------

/// The seal at the foot of a plate, plus the surface's exit hint. One
/// placement across all six surfaces, so they read as one issued family.
///
/// The emblem is sized and tinted from `EmblemPlacement::MenuHeading`
/// rather than hand-typed, and carries a LOCAL `ZIndex` - a
/// `GlobalZIndex` on a child would lift it out of the surface root and
/// break the single-`despawn_recursive` teardown.
pub fn seal_footer(
    p: &mut ChildBuilder,
    brand: Option<&branding::BrandAssets>,
    hint: Option<(&str, &str)>,
) {
    let place = branding::EmblemPlacement::MenuHeading;
    debug_assert!(
        place.size_px() >= EMBLEM_MIN_PX,
        "emblem below the no-fringe floor"
    );
    p.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            margin: UiRect::top(Val::Px(U6)),
            padding: UiRect::top(Val::Px(U3)),
            border: UiRect::top(Val::Px(RULE_HAIR_PX)),
            ..default()
        },
        BorderColor(bronze_a(RULE_SECTION_A)),
    ))
    .with_children(|f| {
        if let Some(b) = brand {
            f.spawn((
                Node {
                    width: Val::Px(place.size_px()),
                    height: Val::Px(place.size_px()),
                    ..default()
                },
                ImageNode {
                    image: b.emblem.clone(),
                    // white tint: alpha only, hue untouched
                    color: Color::srgba(1.0, 1.0, 1.0, place.alpha()),
                    ..default()
                },
                ZIndex(ZL_MARK),
            ));
        } else {
            f.spawn(Node::default());
        }
        if let Some((key, what)) = hint {
            f.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(U2),
                ..default()
            })
            .with_children(|h| {
                keycap(h, key, false);
                h.spawn((
                    Text::new(what.to_string()),
                    TextFont {
                        font_size: T_MICRO,
                        ..default()
                    },
                    TextColor(branding::palette::PARCHMENT_DIM),
                ));
            });
        }
    });
}

// ---- 6. the painter ------------------------------------------------------

/// Repaint one row from its state. THE one place a row's appearance is
/// decided, so five separate `Changed<Interaction>` handlers cannot drift
/// into five different hover treatments.
pub fn paint_row(
    kind: RowKind,
    selected: bool,
    interaction: Interaction,
    bg: &mut BackgroundColor,
    border: &mut BorderColor,
    children: Option<&Children>,
    bosses: &mut Query<&mut BackgroundColor, (With<RowBoss>, Without<PlateRow>)>,
) {
    let (fill, keel, boss) = row_colors(kind, row_state(selected, interaction));
    *bg = BackgroundColor(fill);
    *border = BorderColor(keel);
    if let Some(kids) = children {
        for c in kids.iter() {
            if let Ok(mut b) = bosses.get_mut(*c) {
                *b = BackgroundColor(boss);
                break;
            }
        }
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
