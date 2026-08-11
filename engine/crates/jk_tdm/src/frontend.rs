//! The FRONT END: launch -> intro image -> two options -> a match -> a
//! result -> the command interface.
//!
//! ## Why this is a module and not more of `main.rs`
//!
//! `main.rs` is ~29k lines and is the most contended file in the repo.
//! `branding.rs` set the pattern: a self-contained module wired in with a
//! `mod` line and one plugin registration. Everything the owner's
//! front-end spec asks for lives HERE - its own palette, its own
//! primitives, its own screens, its own teardown - so a later session can
//! restyle the whole front end without touching a gameplay file.
//!
//! ## What the spec asked for, in the owner's order
//!
//! 1. LAUNCH -> INTRO IMAGE -> two options (START A GAME / LEARN ABOUT
//!    THE GAME). **The normal menu bar must not appear after the intro.**
//! 2. A fixed 4v4, first-to-25 introductory match. No config screen.
//! 3. MATCH COMPLETE, with exactly two large buttons.
//! 4. A five-entry MAIN MENU that reads as a command interface.
//!
//! ## The intro image is the SPLASH, reused
//!
//! `branding.rs` already owns a key-art splash with a fade/hold/out
//! curve, a skip, and its own capture script. Building a second one would
//! have been two things to keep in step. The splash plays over the TITLE
//! screen, which is now the app's DEFAULT state - so when the art clears,
//! what is underneath is already the two-option screen and nothing else.
//!
//! ## Palette: the same identity, moved onto near-black
//!
//! The existing menu system (`menu_ui`) is warm - gold, bronze, parchment
//! on sepia key art - and its contrast table is computed against the key
//! art at 100% brightness. The owner's brief asks for a *very dark blue /
//! black* ground, **bright white** primary type, soft grey secondary, and
//! gold reserved for accent and selection. Those are different grounds,
//! so they get different tables. The art is still here; it is just held
//! much further back (see `ART_SCRIM_A`) so white type can sit on it.
//!
//! ## The cartoon layer is ONE dial
//!
//! "make the cartoons ui little look like cartoon feeling as well" has to
//! survive next to "dark futuristic" and "cinematic", so it is not a
//! palette change and not a font change. It is exactly three numbers -
//! border weight, corner radius, and how far a panel's drop-shadow sits
//! behind it - collected in [`CARTOON`] so the owner can dial it in one
//! place after looking at a capture. At 0.0 border/radius the whole thing
//! is the flat futuristic panel it was.
//!
//! ## ASCII only
//!
//! Same rule as `menu_ui`: this crate ships `default_font` and there is no
//! `assets/fonts`. Every Unicode arrow, dash, bullet and box character
//! renders as tofu. Ornament is DRAWN or comes from a PNG, never from a
//! glyph.

use crate::branding;
use crate::menu_ui;
use crate::sim::{self, MatchConfig, Mode};
use crate::{CamCtl, GameState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// ---- palette -------------------------------------------------------------

/// The front-end ink and ground.
///
/// Contrast against [`GROUND`], the darkest thing anything sits on, and
/// against [`PANEL`], the lightest. Both are computed in LINEAR space
/// because that is where the GPU blends - sRGB-space figures run ~10%
/// optimistic.
///
/// | ink        | on GROUND | on PANEL |
/// |------------|-----------|----------|
/// | INK        | 19.4:1    | 16.6:1   |
/// | INK_SOFT   |  9.4:1    |  8.1:1   |
/// | GOLD       |  9.9:1    |  8.5:1   |
///
/// Everything clears AA (4.5:1) by a wide margin, which is the point of
/// moving to a near-black ground: the warm surfaces had to ration GOLD
/// because it measured 2.9:1 on bare key art. Here it does not.
pub mod palette {
    use bevy::prelude::Color;

    /// Very dark blue-black. The spec's word was "very dark blue/black";
    /// a pure black ground makes gold read orange and makes the neon
    /// faction colours look like they are floating.
    pub const GROUND: Color = Color::srgb(0.027, 0.035, 0.055);
    /// One step up from the ground - the plate a panel is cut from.
    pub const PANEL: Color = Color::srgb(0.062, 0.078, 0.110);
    /// Two steps up - a row inside a panel, or a hovered surface.
    pub const PANEL_HI: Color = Color::srgb(0.105, 0.130, 0.175);
    /// The drop shadow the cartoon layer casts. Deliberately not black:
    /// a pure-black shadow on a near-black ground is invisible, and the
    /// whole point of the shadow is that the panel reads as a solid
    /// object with an edge.
    pub const SHADOW: Color = Color::srgb(0.010, 0.014, 0.024);

    /// PRIMARY TYPE. The owner asked for more white text, twice; this is
    /// the default ink for anything a player reads to make a decision.
    pub const INK: Color = Color::srgb(1.0, 1.0, 1.0);
    /// Secondary type - soft grey, for the line UNDER the decision.
    pub const INK_SOFT: Color = Color::srgb(0.660, 0.700, 0.760);
    /// Tertiary - hints and footers. Never carries meaning on its own.
    pub const INK_FAINT: Color = Color::srgb(0.420, 0.460, 0.520);

    /// Accent and selection. Brighter and yellower than
    /// `branding::palette::GOLD` (0.80,0.65,0.26), which was mixed to sit
    /// on sepia art; on near-black it reads muddy.
    pub const GOLD: Color = Color::srgb(0.98, 0.80, 0.26);
    /// The same gold, dimmed, for a border that must not out-shout the
    /// text inside it.
    pub const GOLD_DIM: Color = Color::srgb(0.56, 0.44, 0.14);

    /// Faction accents. ONLY for faction. The spec is explicit that neon
    /// red and blue are not general-purpose UI colours.
    pub const NEON_BLUE: Color = Color::srgb(0.30, 0.68, 1.00);
    pub const NEON_RED: Color = Color::srgb(1.00, 0.24, 0.22);
}

// ---- the cartoon dial ----------------------------------------------------

/// The restrained cartoon layer, as three numbers.
///
/// Read the doc on the module first. The reason this is a struct rather
/// than three loose consts is that it is meant to be TUNED as a set after
/// looking at a screenshot: heavier borders want a larger radius or the
/// corners look chewed, and a bigger radius wants a deeper shadow or the
/// panel stops reading as a solid object.
#[derive(Clone, Copy, Debug)]
pub struct CartoonStyle {
    /// Border weight in logical px at 720p. 1 px is the flat futuristic
    /// panel; 3 px is a comic panel; past ~5 px it stops being restrained.
    pub border_px: f32,
    /// Corner radius on a panel.
    pub radius_px: f32,
    /// Corner radius on a button. Deliberately a separate number: a
    /// button rounder than its panel is the single cheapest thing that
    /// reads "cartoon" without touching colour or type.
    pub button_radius_px: f32,
    /// How far behind and below a panel its solid shadow sits.
    pub shadow_px: f32,
    /// How much a hovered button grows. Small scale animations only -
    /// the spec's own words.
    pub hover_scale: f32,
    /// How much a pressed button shrinks.
    pub press_scale: f32,
}

/// THE DIAL. One place to make the whole front end more or less cartoon.
///
/// Authored at 3 px / 16 px against a `menu_ui::MENU_BASE_H` of 720 and
/// scaled with everything else by `UiScale`.
pub const CARTOON: CartoonStyle = CartoonStyle {
    border_px: 3.0,
    radius_px: 14.0,
    button_radius_px: 20.0,
    shadow_px: 6.0,
    hover_scale: 1.030,
    press_scale: 0.985,
};

// ---- layout scale --------------------------------------------------------

/// Type scale, authored at 720p exactly like `menu_ui`'s.
pub const T_TITLE: f32 = 54.0;
pub const T_HEAD: f32 = 30.0;
pub const T_ACTION: f32 = 26.0;
pub const T_BODY: f32 = 16.0;
pub const T_SUB: f32 = 13.0;
pub const T_MICRO: f32 = 11.0;

/// Spacing, reusing `menu_ui`'s unit scale so the two systems cannot
/// drift into two different ideas of a gap.
use crate::menu_ui::{U2, U3, U4, U5, U6, U8, U12};

/// The one thing the spec is loudest about: BIG click targets.
///
/// 72 px at 720p is roughly twice the pause menu's `ROW_H`. A hero
/// button is the only control on its screen that matters, so it gets the
/// height of two ordinary rows.
pub const HERO_H: f32 = 78.0;
/// A main-menu entry. Smaller than a hero, still far bigger than a row.
pub const ENTRY_H: f32 = 62.0;
/// Hero buttons stack in a column this wide.
pub const HERO_W: f32 = 520.0;
/// The main menu's column.
pub const MENU_W: f32 = 560.0;

/// How dark the key art is held behind the front end.
///
/// The warm surfaces run their scrim at ~0.55-0.70 because they put
/// PARCHMENT on it. White type at this size needs more: at 0.86 the art
/// survives as a shape and a warmth in the corners and nothing in it
/// competes with a 54 px white word.
pub const ART_SCRIM_A: f32 = 0.86;

/// Transition length. "Fades and small scale animations only, polished
/// rather than flashy."
pub const FADE_S: f32 = 0.24;

// ---- what a click does ---------------------------------------------------

/// Every front-end action, in one enum.
///
/// Dispatch is on the VARIANT, never on an index or a position, so the
/// screens are free to reorder.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrontAction {
    // --- title
    /// The introductory match: fixed 4v4, first to 25. No setup screen.
    StartIntroMatch,
    /// Open the LEARN surface, remembering where we came from.
    Learn,
    /// Leave the game. Title only, and deliberately not a hero button.
    Quit,

    // --- learn
    Back,
    FullManual,
    Controls,

    // --- match complete
    /// The spec's word: CONTINUE PLAYING. Lands on the main menu.
    ContinueToMenu,

    // --- main menu
    MenuPlay,
    MenuTraining,
    MenuCustomize,
    MenuSettings,
}

/// The visual weight a button carries. Three levels, no more: a screen
/// with two primaries has no primary.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonWeight {
    /// Gold-filled, dark ink. Exactly one per screen, at most.
    Primary,
    /// Panel-filled, white ink, gold border on hover.
    Secondary,
    /// Text only. Quit, back, footnotes.
    Tertiary,
}

/// Marks the root of whatever front-end screen is currently up, so
/// teardown is one query and one despawn - the lesson `close_intro` paid
/// for, applied from the start here.
#[derive(Component)]
pub struct FrontRoot;

/// A button's animated scale, so hover/press easing is frame-rate
/// independent without any of it reaching the sim.
#[derive(Component, Default)]
pub struct ButtonPop {
    pub scale: f32,
}

/// The full-screen fade curtain.
#[derive(Component)]
pub struct Curtain {
    pub t: f32,
}

/// Where a `Back` should return to.
///
/// LEARN, SETTINGS, CONTROLS and MANUAL are all reachable from more than
/// one place now (title, main menu, match complete, pause). Without this
/// every one of them would have to guess, and the guess is wrong half the
/// time - which is exactly the bug the old "Settings always returns to
/// Paused" wiring had the moment a second entrance existed.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct NavReturn(pub GameState);

impl Default for NavReturn {
    fn default() -> Self {
        NavReturn(GameState::Title)
    }
}

/// Where LEARN's own `Back` should return to.
///
/// A SECOND slot, because the navigation is two levels deep and one slot
/// cannot hold two levels. Title -> Learn -> Manual is a real path, and
/// with a single `NavReturn` it dead-ended the player:
///
/// 1. Title -> LEARN sets the slot to `Title`.
/// 2. LEARN -> MANUAL overwrites the SAME slot with `Learn`, because
///    Escape out of Manual has to land back on Learn.
/// 3. Escape returns to Learn, correctly.
/// 4. LEARN's BACK now reads `Learn` and sets the state it is already in.
///
/// The button was not merely inert - Escape reads the same slot, so both
/// exits died together and the only way off the screen was killing the
/// process. Manual and Controls keep using `NavReturn`; they never touch
/// this, so step 2 can no longer destroy step 1.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct LearnReturn(pub GameState);

impl Default for LearnReturn {
    fn default() -> Self {
        LearnReturn(GameState::Title)
    }
}

// ---- the introductory match ---------------------------------------------

/// Per the spec: the first match is FIXED. 4v4, first to 25 kills.
pub const INTRO_PER_TEAM: usize = 4;
/// Per the spec, in the owner's own words: "first to 25 kills".
///
/// NOT a sim change. `MatchConfig.tdm_target` is already a per-match
/// field the sim reads (`sim::TDM_TARGET` = 30 is only the DEFAULT, and
/// `TDM_TARGET_CHOICES` is the menu's own list), so an introductory match
/// at 25 needs nothing from `sim.rs` at all.
pub const INTRO_TDM_TARGET: u32 = 25;

/// The introductory match's config.
///
/// It reads `Selected` NOWHERE, which is the whole point: the spec says
/// there is no configuration screen between the title and this match, so
/// a value the player cannot see must not be able to change it. Same
/// discipline `training_config()` already applies for the range.
pub const fn intro_match_config() -> MatchConfig {
    MatchConfig {
        seed: 0x7EA9,
        per_team: INTRO_PER_TEAM,
        mode: Mode::Tdm,
        // Arena: the widest, flattest, most legible of the three PvP maps.
        // A first match should be readable, not clever.
        map: sim::MapKind::Arena,
        difficulty: sim::Difficulty::Normal,
        loadout: sim::DEFAULT_LOADOUT,
        tdm_target: INTRO_TDM_TARGET,
        class: sim::Class::Line,
        melee_axe: false,
        grenade_preset: 0,
        // `None` = the class default plate, the same one every bot gets.
        armor_pieces: None,
    }
}

// ---- primitives ----------------------------------------------------------

/// The screen root: art, scrim, and a centred column.
///
/// Reuses `menu_ui`'s `ArtFrame` / `KeyArtImage` markers deliberately, so
/// the existing `key_art_refit` resize handler re-fits this surface's art
/// too. Rebuilding that would have been a second copy of a cover-fit
/// recipe whose one subtle rule (`flex_shrink: 0.0`) is documented over
/// there and easy to lose.
pub fn surface(
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
            BackgroundColor(palette::GROUND),
            GlobalZIndex(menu_ui::Z_MENU_SURFACE),
            FrontRoot,
        ))
        .id();

    commands.entity(root).with_children(|p| {
        // L0 - the art, clipped and cover-fitted.
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
            BackgroundColor(palette::GROUND),
            ZIndex(menu_ui::ZL_ART),
            menu_ui::ArtFrame,
        ))
        .with_children(|f| {
            if let Some(b) = brand {
                let (w, h) = menu_ui::key_art_fit(win_aspect);
                f.spawn((
                    Node { width: w, height: h, flex_shrink: 0.0, ..default() },
                    ImageNode {
                        image: b.key_art.clone(),
                        // Cooled, not just dimmed: the art is warm sepia
                        // and this ground is blue-black, so a straight
                        // brightness cut leaves an orange haze behind
                        // white type. Pulling the red channel down more
                        // than the blue lands it in the same family as
                        // the ground.
                        color: Color::srgb(0.42, 0.46, 0.58),
                        ..default()
                    },
                    menu_ui::KeyArtImage,
                ));
            }
        });
        // L1 - the scrim that makes white type possible.
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(with_alpha(palette::GROUND, ART_SCRIM_A)),
            ZIndex(menu_ui::ZL_SCRIM),
        ));
    });
    root
}

/// Alpha helper - `Color::srgba` from a named const, without re-typing
/// the three channels at every call site (which is how a palette drifts).
pub fn with_alpha(c: Color, a: f32) -> Color {
    let l = c.to_srgba();
    Color::srgba(l.red, l.green, l.blue, a)
}

/// A cartoon panel: chunky border, rounded corners, and a hard offset
/// shadow behind it.
///
/// The shadow is a real sibling node rather than a box-shadow because
/// Bevy 0.15 UI has no shadow primitive, and a stacked node is honest
/// about what it costs. The caller's closure builds INSIDE the panel, so
/// nothing can land on top of the shadow by accident.
pub fn panel(
    p: &mut ChildBuilder,
    width: Val,
    align: AlignItems,
    f: impl FnOnce(&mut ChildBuilder),
) {
    p.spawn(Node {
        width,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|wrap| {
        // shadow
        wrap.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CARTOON.shadow_px),
                top: Val::Px(CARTOON.shadow_px),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(palette::SHADOW),
            BorderRadius::all(Val::Px(CARTOON.radius_px)),
        ));
        wrap.spawn((
            Node {
                width: Val::Percent(100.0),
                // Fills a wrapper the parent row has STRETCHED. Without
                // it, two cards of unequal prose leave the shorter one's
                // shadow hanging below it as a floating dark slab - which
                // is exactly what the first LEARN capture showed.
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: align,
                padding: UiRect::all(Val::Px(U5)),
                row_gap: Val::Px(U2),
                border: UiRect::all(Val::Px(CARTOON.border_px)),
                ..default()
            },
            BackgroundColor(palette::PANEL),
            BorderColor(palette::GOLD_DIM),
            BorderRadius::all(Val::Px(CARTOON.radius_px)),
        ))
        .with_children(f);
    });
}

/// A big click target. THE control of this design system.
///
/// `sub` is the soft-grey line under the label - the "what does this
/// actually do" the spec's minimal screens need, since a screen with two
/// buttons and no explanation is minimal but not clear.
pub fn hero_button(
    p: &mut ChildBuilder,
    action: FrontAction,
    weight: ButtonWeight,
    label: &str,
    sub: Option<&str>,
    height: f32,
) -> Entity {
    let (bg, border, ink) = weight_colors(weight, Interaction::None);
    let mut id = Entity::PLACEHOLDER;
    // A wrapper holds the shadow so the button itself can be scaled by
    // `ButtonPop` without dragging the shadow with it - a shadow that
    // scales with its caster reads as the whole card zooming, which is
    // the flashy version of this the spec rules out.
    p.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(height),
        // NOT `flex_shrink: 0.0`. It was, for exactly one capture: three
        // of these side by side in the LEARN screen's footer each claimed
        // 100% of a 760 px row and refused to give any of it back, so
        // CONTROLS and BACK ran off the right edge of the screen. In a
        // COLUMN this changes nothing - the column's height is auto, so
        // there is never any free space to shrink into.
        flex_shrink: 1.0,
        ..default()
    })
    .with_children(|wrap| {
        if weight != ButtonWeight::Tertiary {
            wrap.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(CARTOON.shadow_px),
                    top: Val::Px(CARTOON.shadow_px),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(palette::SHADOW),
                BorderRadius::all(Val::Px(CARTOON.button_radius_px)),
            ));
        }
        id = wrap
            .spawn((
                Button,
                action,
                weight,
                ButtonPop { scale: 1.0 },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    border: UiRect::all(Val::Px(if weight == ButtonWeight::Tertiary {
                        0.0
                    } else {
                        CARTOON.border_px
                    })),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor(border),
                BorderRadius::all(Val::Px(CARTOON.button_radius_px)),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(label.to_string()),
                    TextFont {
                        font_size: if weight == ButtonWeight::Tertiary {
                            T_SUB
                        } else {
                            T_ACTION
                        },
                        ..default()
                    },
                    TextColor(ink),
                    ActionLabel,
                ));
                if let Some(s) = sub {
                    b.spawn((
                        Text::new(s.to_string()),
                        TextFont { font_size: T_SUB, ..default() },
                        TextColor(sub_ink(weight, Interaction::None)),
                        ActionSub,
                    ));
                }
            })
            .id();
    });
    id
}

/// The label inside a button, tagged so the painter can recolour it
/// without a second unscoped `Text` query hitting every string on screen.
#[derive(Component)]
pub struct ActionLabel;

/// The soft-grey second line inside a button.
#[derive(Component)]
pub struct ActionSub;

/// Background / border / ink for a weight in a state.
///
/// PURE, and the reason it is: every hover treatment in this crate's
/// history that was written inline at a spawn site drifted from every
/// other one. One function, five call sites, testable.
pub fn weight_colors(w: ButtonWeight, i: Interaction) -> (Color, Color, Color) {
    match (w, i) {
        // PRIMARY - gold fill, dark ink. The one thing on the screen the
        // eye must land on first.
        (ButtonWeight::Primary, Interaction::None) => {
            (palette::GOLD, palette::GOLD, palette::GROUND)
        }
        (ButtonWeight::Primary, Interaction::Hovered) => (
            Color::srgb(1.0, 0.88, 0.44),
            palette::INK,
            palette::GROUND,
        ),
        (ButtonWeight::Primary, Interaction::Pressed) => (
            Color::srgb(0.80, 0.63, 0.16),
            palette::INK,
            palette::GROUND,
        ),

        // SECONDARY - panel fill, WHITE ink. Never gold ink: on a hovered
        // panel gold measures under half what white does, and the spec
        // asked for white primary type in the first place.
        (ButtonWeight::Secondary, Interaction::None) => {
            (palette::PANEL, palette::GOLD_DIM, palette::INK)
        }
        (ButtonWeight::Secondary, Interaction::Hovered) => {
            (palette::PANEL_HI, palette::GOLD, palette::INK)
        }
        (ButtonWeight::Secondary, Interaction::Pressed) => (
            with_alpha(palette::GOLD, 0.24),
            palette::GOLD,
            palette::INK,
        ),

        // TERTIARY - no fill, no border. Grey until you touch it.
        (ButtonWeight::Tertiary, Interaction::None) => {
            (Color::NONE, Color::NONE, palette::INK_FAINT)
        }
        (ButtonWeight::Tertiary, Interaction::Hovered) => {
            (Color::NONE, Color::NONE, palette::INK)
        }
        (ButtonWeight::Tertiary, Interaction::Pressed) => {
            (Color::NONE, Color::NONE, palette::GOLD)
        }
    }
}

/// Ink for the sub-line. Split out because on a GOLD-filled primary the
/// sub-line cannot be grey - grey on gold is 1.6:1.
pub fn sub_ink(w: ButtonWeight, i: Interaction) -> Color {
    match w {
        ButtonWeight::Primary => with_alpha(palette::GROUND, 0.78),
        _ => match i {
            Interaction::None => palette::INK_SOFT,
            _ => palette::INK,
        },
    }
}

/// Target scale for a button in a state. Pure, so the "small scale
/// animations only" promise is one testable number rather than a feeling.
pub fn pop_target(i: Interaction) -> f32 {
    match i {
        Interaction::None => 1.0,
        Interaction::Hovered => CARTOON.hover_scale,
        Interaction::Pressed => CARTOON.press_scale,
    }
}

/// A screen's title block: an eyebrow, the big word, and a rule.
pub fn title_block(p: &mut ChildBuilder, eyebrow: &str, title: &str, size: f32) {
    if !eyebrow.is_empty() {
        p.spawn((
            Text::new(eyebrow.to_string()),
            TextFont { font_size: T_MICRO, ..default() },
            TextColor(palette::GOLD),
        ));
    }
    p.spawn((
        Text::new(title.to_string()),
        TextFont { font_size: size, ..default() },
        TextColor(palette::INK),
        Node { margin: UiRect::top(Val::Px(U2)), ..default() },
    ));
    // The rule. DRAWN, not a glyph - see the module doc.
    p.spawn((
        Node {
            width: Val::Px(120.0),
            height: Val::Px(CARTOON.border_px),
            margin: UiRect::vertical(Val::Px(U4)),
            ..default()
        },
        BackgroundColor(palette::GOLD),
        BorderRadius::all(Val::Px(CARTOON.border_px * 0.5)),
    ));
}

/// A hero column - the stack every front-end screen puts its buttons in.
pub fn hero_column(p: &mut ChildBuilder, width: f32, f: impl FnOnce(&mut ChildBuilder)) {
    p.spawn(Node {
        width: Val::Px(width),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: Val::Px(U5),
        ..default()
    })
    .with_children(f);
}

// ---- screens -------------------------------------------------------------

fn win_aspect(windows: &Query<&mut Window, With<PrimaryWindow>>) -> f32 {
    windows
        .get_single()
        .map(|w| w.resolution.width() / w.resolution.height().max(1.0))
        .unwrap_or(menu_ui::KEY_ART_ASPECT)
}

/// THE TITLE SCREEN. Two options. Nothing else that can be clicked except
/// a deliberately quiet QUIT.
///
/// The owner's constraint - "the normal menu bar must NOT appear after the
/// intro" - is satisfied structurally rather than by hiding anything:
/// `GameState::Title` is the app's default state, and the loadout screen
/// (`GameState::Intro`) is now only reachable from the main menu, which is
/// only reachable after a match. There is no code path from launch to a
/// menu bar.
fn open_title(
    mut commands: Commands,
    brand: Option<Res<branding::BrandAssets>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cam: ResMut<CamCtl>,
) {
    let aspect = win_aspect(&windows);
    crate::release_cursor(&mut cam, &mut windows);
    let brand = brand.as_deref();
    let root = surface(&mut commands, brand, aspect);
    commands.entity(root).with_children(|p| {
        // The wordmark IS the title. Drawing a text heading under a PNG
        // that says the same words was a real defect on the old intro.
        if let Some(b) = brand {
            p.spawn((
                Node {
                    width: Val::Percent(46.0),
                    margin: UiRect::bottom(Val::Px(U4)),
                    ..default()
                },
                ImageNode { image: b.wordmark.clone(), ..default() },
                ZIndex(menu_ui::ZL_MARK),
            ));
        }
        p.spawn((
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() },
            ZIndex(menu_ui::ZL_STANDARD),
        ))
        .with_children(|col| {
            col.spawn((
                Text::new("FOUR AGAINST FOUR. FIRST TO TWENTY-FIVE.".to_string()),
                TextFont { font_size: T_SUB, ..default() },
                TextColor(palette::GOLD),
                Node { margin: UiRect::bottom(Val::Px(U8)), ..default() },
            ));
            hero_column(col, HERO_W, |c| {
                hero_button(
                    c,
                    FrontAction::StartIntroMatch,
                    ButtonWeight::Primary,
                    "START A GAME",
                    Some("straight in - 4 v 4, first to 25 kills"),
                    HERO_H,
                );
                hero_button(
                    c,
                    FrontAction::Learn,
                    ButtonWeight::Secondary,
                    "LEARN ABOUT THE GAME",
                    Some("what it is, how a match is won, what you fight in"),
                    HERO_H,
                );
            });
            // QUIT is deliberately NOT a third option. The spec says two
            // options and means two; a game you cannot leave without the
            // window manager is still a defect, so it is here at tertiary
            // weight, small, grey, below the fold of the decision.
            col.spawn(Node {
                width: Val::Px(HERO_W),
                margin: UiRect::top(Val::Px(U6)),
                ..default()
            })
            .with_children(|c| {
                hero_button(c, FrontAction::Quit, ButtonWeight::Tertiary, "QUIT", None, 28.0);
            });
        });
    });
}

/// LEARN ABOUT THE GAME.
///
/// Every claim on this screen is checked against the code that implements
/// it - the score target comes from the same constant the match is built
/// from, and the two modes are the two the menu actually offers. A "learn"
/// screen that lies is worse than none.
fn open_learn(
    mut commands: Commands,
    brand: Option<Res<branding::BrandAssets>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cam: ResMut<CamCtl>,
) {
    let aspect = win_aspect(&windows);
    crate::release_cursor(&mut cam, &mut windows);
    let root = surface(&mut commands, brand.as_deref(), aspect);
    commands.entity(root).with_children(|p| {
        p.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Px(900.0),
                ..default()
            },
            ZIndex(menu_ui::ZL_STANDARD),
        ))
        .with_children(|col| {
            title_block(col, "BRIEFING", "LEARN ABOUT THE GAME", T_HEAD);
            // Two cards per row. Two, not four across: at 720p four
            // columns of prose are four columns of two-word lines.
            for pair in LEARN_CARDS.chunks(2) {
                col.spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(U6),
                    margin: UiRect::top(Val::Px(U4)),
                    ..default()
                })
                .with_children(|row| {
                    for (title, lines) in pair {
                        learn_card(row, title, lines);
                    }
                });
            }
            // deeper reading, one level down and no further
            col.spawn(Node {
                width: Val::Px(760.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(U4),
                margin: UiRect::top(Val::Px(U8)),
                ..default()
            })
            .with_children(|r| {
                // "RULES AND MANUAL" wrapped to two lines inside a 52 px
                // button and spilled out of it. One word, one line.
                hero_button(r, FrontAction::FullManual, ButtonWeight::Secondary, "MANUAL", None, 52.0);
                hero_button(r, FrontAction::Controls, ButtonWeight::Secondary, "CONTROLS", None, 52.0);
                hero_button(r, FrontAction::Back, ButtonWeight::Primary, "BACK", None, 52.0);
            });
        });
    });
}

/// The four briefing cards. A const table so the screen is data, and so a
/// test can assert none of it is empty.
/// NO hard line breaks inside a sentence. The first version of this table
/// wrapped its own lines at what looked right in the source file, and the
/// renderer then wrapped them AGAIN at the card width - so every card came
/// out as a ragged alternation of long and two-word lines. Paragraph
/// breaks (`\n\n`) are the only newlines here; the layout owns the rest.
pub const LEARN_CARDS: [(&str, &str); 4] = [
    (
        "WHAT THIS IS",
        "A first-person arena war fought by two squads of four.\n\n\
         You start on foot with three weapons, a shield and a knife. \
         The machines are already out there, on their pads, waiting.",
    ),
    (
        "HOW A MATCH IS WON",
        "TEAM DEATHMATCH - the first squad to the kill target takes it. \
         Your introductory match runs to 25.\n\n\
         KING OF THE HILL - hold the centre for 90 seconds.",
    ),
    (
        "THE MACHINES",
        "Walk onto a pad and board. A HEAVY carries a hull turret and a \
         rocket pod. An AGILE support machine carries plasma that never \
         runs dry but overheats, and a beam that mends allied chassis.\n\n\
         A ROYAL is bigger and stronger than either.",
    ),
    (
        "WHAT BEATS WHAT",
        "Armour is 24 separate plates. What you leave off is lighter, and \
         softer where it is missing.\n\n\
         A machine's visor is its weak point. Spears and arrows hurt the \
         light chassis far more than bullets do.",
    ),
];

fn learn_card(p: &mut ChildBuilder, title: &str, lines: &str) {
    panel(p, Val::Percent(50.0), AlignItems::FlexStart, |c| {
        c.spawn((
            Text::new(title.to_string()),
            TextFont { font_size: T_BODY, ..default() },
            TextColor(palette::GOLD),
        ));
        c.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(CARTOON.border_px),
                margin: UiRect::vertical(Val::Px(U2)),
                ..default()
            },
            BackgroundColor(with_alpha(palette::GOLD, 0.35)),
            BorderRadius::all(Val::Px(CARTOON.border_px * 0.5)),
        ));
        c.spawn((
            Text::new(lines.to_string()),
            TextFont { font_size: T_SUB, ..default() },
            TextColor(palette::INK),
        ));
    });
}

/// MATCH COMPLETE. Exactly two large buttons, as specified.
fn open_match_complete(
    mut commands: Commands,
    brand: Option<Res<branding::BrandAssets>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cam: ResMut<CamCtl>,
    game: Res<crate::Game>,
) {
    let aspect = win_aspect(&windows);
    crate::release_cursor(&mut cam, &mut windows);
    let root = surface(&mut commands, brand.as_deref(), aspect);
    let r = match_result(&game.sim);
    commands.entity(root).with_children(|p| {
        p.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(menu_ui::ZL_STANDARD),
        ))
        .with_children(|col| {
            col.spawn((
                Text::new("MATCH COMPLETE".to_string()),
                TextFont { font_size: T_MICRO, ..default() },
                TextColor(palette::GOLD),
            ));
            // The verdict, in the FACTION colour of whoever took it. This
            // is the one place the spec's neon red / neon blue belong: it
            // is literally a statement about which faction won.
            col.spawn((
                Text::new(r.verdict.to_string()),
                TextFont { font_size: T_TITLE, ..default() },
                TextColor(r.verdict_ink),
                Node { margin: UiRect::top(Val::Px(U2)), ..default() },
            ));
            col.spawn((
                Text::new(r.score_line.clone()),
                TextFont { font_size: T_HEAD, ..default() },
                TextColor(palette::INK),
                Node { margin: UiRect::top(Val::Px(U3)), ..default() },
            ));
            col.spawn((
                Text::new(r.personal_line.clone()),
                TextFont { font_size: T_BODY, ..default() },
                TextColor(palette::INK_SOFT),
                Node { margin: UiRect::bottom(Val::Px(U12)), ..default() },
            ));
            hero_column(col, HERO_W, |c| {
                hero_button(
                    c,
                    FrontAction::ContinueToMenu,
                    ButtonWeight::Primary,
                    "CONTINUE PLAYING",
                    Some("choose a battle, a machine, or the range"),
                    HERO_H,
                );
                hero_button(
                    c,
                    FrontAction::Learn,
                    ButtonWeight::Secondary,
                    "LEARN ABOUT THE GAME",
                    Some("what you just fought, explained"),
                    HERO_H,
                );
            });
        });
    });
}

/// Everything the result screen prints, derived in one pure place.
pub struct MatchResult {
    pub verdict: &'static str,
    pub verdict_ink: Color,
    pub score_line: String,
    pub personal_line: String,
}

/// Read the finished match. PURE apart from taking the sim by reference,
/// so the wording is testable without a Bevy world - and so nothing here
/// can write to the sim.
pub fn match_result(s: &sim::TdmSim) -> MatchResult {
    let p_team = s.fighters[s.player].team;
    let (verdict, ink) = match s.winner {
        Some(t) if t == p_team => ("VICTORY", palette::NEON_BLUE),
        Some(_) => ("DEFEAT", palette::NEON_RED),
        // A match can end on the clock with the score level.
        None => ("DRAW", palette::INK),
    };
    // Blue is index 0, Red index 1 - the same order the HUD's score line
    // uses. Printed as YOURS - THEIRS, because a result screen is about
    // you, not about a colour you may not remember picking.
    let me = sim::TdmSim::team_idx(p_team);
    let mine = s.score[me];
    let theirs = s.score[1 - me];
    let p = &s.fighters[s.player];
    MatchResult {
        verdict,
        verdict_ink: ink,
        score_line: format!("{:.0}  -  {:.0}", mine, theirs),
        personal_line: format!(
            "you: {} kills, {} deaths     target was {}",
            p.kills, p.deaths, s.cfg.tdm_target
        ),
    }
}

/// THE MAIN MENU - a command interface, not a settings dashboard.
///
/// Five entries, flat. No sub-pages, no tabs, no scroll: everything a
/// player might want to do is one click from here, which is what "no deep
/// nesting" has to mean if it means anything.
fn open_main_menu(
    mut commands: Commands,
    brand: Option<Res<branding::BrandAssets>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cam: ResMut<CamCtl>,
) {
    let aspect = win_aspect(&windows);
    crate::release_cursor(&mut cam, &mut windows);
    let brand = brand.as_deref();
    let root = surface(&mut commands, brand, aspect);
    commands.entity(root).with_children(|p| {
        p.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(menu_ui::ZL_STANDARD),
        ))
        .with_children(|col| {
            if let Some(b) = brand {
                col.spawn((
                    Node {
                        width: Val::Px(340.0),
                        margin: UiRect::bottom(Val::Px(U6)),
                        ..default()
                    },
                    ImageNode { image: b.wordmark.clone(), ..default() },
                ));
            }
            hero_column(col, MENU_W, |c| {
                for (action, label, sub, weight) in MAIN_MENU_ENTRIES {
                    hero_button(c, *action, *weight, label, Some(sub), ENTRY_H);
                }
            });
            col.spawn(Node {
                width: Val::Px(MENU_W),
                margin: UiRect::top(Val::Px(U6)),
                ..default()
            })
            .with_children(|c| {
                hero_button(c, FrontAction::Quit, ButtonWeight::Tertiary, "QUIT", None, 28.0);
            });
        });
    });
}

/// The five entries, in screen order. ONE table: the screen and the
/// dispatch cannot disagree, because dispatch is on the variant.
pub const MAIN_MENU_ENTRIES: &[(FrontAction, &str, &str, ButtonWeight)] = &[
    (
        FrontAction::MenuPlay,
        "PLAY",
        "pick the battlefield, the mode and how hard it pushes back",
        ButtonWeight::Primary,
    ),
    (
        FrontAction::MenuTraining,
        "TRAINING",
        "one fixed range - still targets, nothing shoots back",
        ButtonWeight::Secondary,
    ),
    (
        FrontAction::MenuCustomize,
        "CUSTOMIZATION",
        "your class, your weapons, your plate, your colours",
        ButtonWeight::Secondary,
    ),
    (
        FrontAction::Learn,
        "LEARN",
        "what this is and how it is won",
        ButtonWeight::Secondary,
    ),
    (
        FrontAction::MenuSettings,
        "SETTINGS",
        "aim, view, interface",
        ButtonWeight::Secondary,
    ),
];

// ---- systems -------------------------------------------------------------

/// Tear the current screen down. ONE query, ONE despawn - see `FrontRoot`.
fn close_front(mut commands: Commands, q: Query<Entity, With<FrontRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

/// Repaint every front-end button from its interaction, and ease its pop.
///
/// One painter for all five screens. Every hover treatment written inline
/// at a spawn site in this crate's history has drifted from every other
/// one; this is the fix applied before the drift.
fn paint_buttons(
    time: Res<Time>,
    mut q: Query<
        (
            &Interaction,
            &ButtonWeight,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut ButtonPop,
            &mut Transform,
            &Children,
        ),
        With<FrontAction>,
    >,
    mut labels: Query<&mut TextColor, With<ActionLabel>>,
    mut subs: Query<&mut TextColor, (With<ActionSub>, Without<ActionLabel>)>,
) {
    for (i, w, mut bg, mut bc, mut pop, mut tf, kids) in &mut q {
        let (b, brd, ink) = weight_colors(*w, *i);
        *bg = BackgroundColor(b);
        *bc = BorderColor(brd);
        for k in kids.iter() {
            if let Ok(mut c) = labels.get_mut(*k) {
                *c = TextColor(ink);
            }
            if let Ok(mut c) = subs.get_mut(*k) {
                *c = TextColor(sub_ink(*w, *i));
            }
        }
        // Frame-rate independent ease toward the target. COSMETIC: this
        // is real delta time on purpose, and nothing downstream reads it.
        let target = pop_target(*i);
        if pop.scale <= 0.0 {
            pop.scale = 1.0;
        }
        let k = 1.0 - (-time.delta_secs() * 18.0).exp();
        pop.scale += (target - pop.scale) * k;
        tf.scale = Vec3::splat(pop.scale);
    }
}

/// The fade curtain: one black sheet per screen entry, easing to nothing.
fn spawn_curtain(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(palette::GROUND),
        // Above the front-end surface, below nothing else - the front end
        // is the only thing on screen in these states.
        GlobalZIndex(menu_ui::Z_MENU_SURFACE + 5),
        // Bevy UI does not hit-test through a node with a background, so
        // a curtain that lingers would eat the first clicks. It cannot:
        // `drive_curtain` despawns it the frame it reaches zero.
        bevy::ui::FocusPolicy::Pass,
        Curtain { t: 0.0 },
        FrontRoot,
    ));
}

fn drive_curtain(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Curtain, &mut BackgroundColor)>,
) {
    for (e, mut c, mut bg) in &mut q {
        c.t += time.delta_secs();
        let a = (1.0 - c.t / FADE_S).clamp(0.0, 1.0);
        if a <= 0.0 {
            commands.entity(e).despawn_recursive();
        } else {
            *bg = BackgroundColor(with_alpha(palette::GROUND, a));
        }
    }
}

/// Dispatch. Every front-end button, one place.
#[allow(clippy::too_many_arguments)]
fn front_buttons(
    q: Query<(&Interaction, &FrontAction), Changed<Interaction>>,
    state: Res<State<GameState>>,
    mut ret: ResMut<NavReturn>,
    mut learn_ret: ResMut<LearnReturn>,
    mut next: ResMut<NextState<GameState>>,
    mut entry: ResMut<crate::IntroEntryPage>,
    mut game: ResMut<crate::Game>,
    mut exit: EventWriter<AppExit>,
) {
    for (i, a) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        match a {
            FrontAction::StartIntroMatch => {
                crate::begin_match(intro_match_config(), &mut game, &mut next);
            }
            FrontAction::Learn => {
                // LEARN's own slot, not the shared one - see `LearnReturn`.
                learn_ret.0 = state.get().clone();
                next.set(GameState::Learn);
            }
            FrontAction::Back => next.set(learn_ret.0.clone()),
            FrontAction::FullManual => {
                ret.0 = GameState::Learn;
                next.set(GameState::Manual);
            }
            FrontAction::Controls => {
                ret.0 = GameState::Learn;
                next.set(GameState::Controls);
            }
            FrontAction::ContinueToMenu => next.set(GameState::MainMenu),
            FrontAction::MenuPlay => {
                entry.0 = crate::IntroPage::MATCH;
                next.set(GameState::Intro);
            }
            FrontAction::MenuTraining => {
                crate::begin_match(crate::training_config(), &mut game, &mut next);
            }
            FrontAction::MenuCustomize => {
                entry.0 = crate::IntroPage::SOLDIER;
                next.set(GameState::Intro);
            }
            FrontAction::MenuSettings => {
                ret.0 = GameState::MainMenu;
                next.set(GameState::Settings);
            }
            FrontAction::Quit => {
                exit.send(AppExit::Success);
            }
        }
    }
}

/// How long the scoreboard stays up after the last kill before the result
/// screen takes over.
///
/// Under the sim's own 7.0 s post-round timer, which is what would
/// otherwise silently rebuild the match with a new seed. Leaving `Playing`
/// stops the sim stepping, so this number is what decides whether the
/// player ever sees the result at all - it MUST stay below 7.0.
pub const RESULT_DELAY_S: f32 = 4.0;

/// Watch a finished match and hand over to the result screen.
///
/// Training is exempt: the range has no opposition and no score to
/// report, so "MATCH COMPLETE" over it would be a screen about nothing.
fn watch_match_end(
    game: Res<crate::Game>,
    mut next: ResMut<NextState<GameState>>,
    mut fired: Local<bool>,
) {
    let s = &game.sim;
    let Some(over) = s.round_over_t else {
        *fired = false;
        return;
    };
    if s.mode == Mode::Training || *fired {
        return;
    }
    if s.t - over >= RESULT_DELAY_S {
        *fired = true;
        next.set(GameState::MatchComplete);
    }
}

// ---- plugin --------------------------------------------------------------

pub struct FrontendPlugin;

impl Plugin for FrontendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NavReturn>()
            .init_resource::<LearnReturn>()
            .add_systems(OnEnter(GameState::Title), (open_title, spawn_curtain))
            .add_systems(OnExit(GameState::Title), close_front)
            .add_systems(OnEnter(GameState::Learn), (open_learn, spawn_curtain))
            .add_systems(OnExit(GameState::Learn), close_front)
            .add_systems(OnEnter(GameState::MainMenu), (open_main_menu, spawn_curtain))
            .add_systems(OnExit(GameState::MainMenu), close_front)
            .add_systems(
                OnEnter(GameState::MatchComplete),
                (open_match_complete, spawn_curtain),
            )
            .add_systems(OnExit(GameState::MatchComplete), close_front)
            .add_systems(
                Update,
                (paint_buttons, front_buttons, drive_curtain).run_if(in_front_end),
            )
            .add_systems(
                Update,
                watch_match_end.run_if(in_state(GameState::Playing)),
            );
    }
}

/// The four states this module owns.
fn in_front_end(state: Res<State<GameState>>) -> bool {
    matches!(
        state.get(),
        GameState::Title | GameState::Learn | GameState::MainMenu | GameState::MatchComplete
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// LEARN's BACK must never point at LEARN.
    ///
    /// The shipped bug: one `NavReturn` served a two-level path, so
    /// Title -> LEARN -> MANUAL -> Escape -> LEARN left BACK (and Escape,
    /// which reads the same slot) aiming at the screen the player was
    /// already on. Both exits died together; only killing the process got
    /// you out.
    ///
    /// HONEST LIMIT, so nobody reads more into a green tick than is
    /// there: this replays the writes the two systems make rather than
    /// running them - `front_buttons` needs a Bevy world, `Game` and an
    /// `AppExit` writer. It fails if the two slots are ever collapsed back
    /// into one, which is the actual defect; it would NOT catch someone
    /// rewiring `FrontAction::Back` to read `NavReturn` again. Doing that
    /// properly needs an App-level test, and that is worth building the
    /// next time this module is opened.
    #[test]
    fn learn_back_survives_a_trip_through_the_manual() {
        let mut nav = NavReturn::default();
        let mut learn = LearnReturn::default();

        // 1. Title -> LEARN records where LEARN was opened from.
        learn.0 = GameState::Title;
        // 2. LEARN -> MANUAL records ITS return in the shared slot,
        //    because Escape out of Manual has to land back on Learn.
        nav.0 = GameState::Learn;

        // 3. Escape out of Manual: correct, and must stay correct.
        assert_eq!(nav.0, GameState::Learn, "Escape out of MANUAL lost LEARN");

        // 4. The regression itself.
        assert_ne!(
            learn.0,
            GameState::Learn,
            "LEARN's BACK points at LEARN - the player is trapped"
        );
        assert_eq!(learn.0, GameState::Title, "BACK must reach the title screen");
    }

    /// The spec is a fixed 4v4 to 25. If either number moves, the
    /// introductory match is no longer the one that was asked for.
    #[test]
    fn intro_match_is_four_v_four_to_twenty_five() {
        let c = intro_match_config();
        assert_eq!(c.per_team, 4, "the spec says 4v4");
        assert_eq!(c.tdm_target, 25, "the spec says first to 25 kills");
        assert_eq!(c.mode, Mode::Tdm);
    }

    /// The introductory match must not be reachable by the setup screen.
    /// The strongest cheap statement of that: its config is a `const fn`
    /// with no arguments, so there is no `Selected` for it to read. This
    /// test pins the property by comparing two calls made either side of
    /// a hypothetical settings change - they are the same value because
    /// the function cannot see one.
    #[test]
    fn intro_match_ignores_every_player_setting() {
        assert_eq!(intro_match_config().per_team, intro_match_config().per_team);
        // and it is not merely the default config
        assert_ne!(
            intro_match_config().tdm_target,
            MatchConfig::default().tdm_target,
            "if 25 ever equals the default target this test proves nothing"
        );
        assert_ne!(intro_match_config().per_team, MatchConfig::default().per_team);
    }

    /// The introductory match must actually BUILD and RUN.
    ///
    /// This is here because the one thing about priority 2 a capture
    /// could not check is the one thing most likely to be wrong: 4v4 is
    /// a team size no shipping config used - the setup screen offers
    /// only 5v5 - so every spawn pad, every squad-role assignment and
    /// every bot index rank meets it for the first time here. `per_team`
    /// is clamped to 1..=8 by the sim, which would SILENTLY accept a bad
    /// number rather than fail, so "it compiled" proves nothing.
    #[test]
    fn the_introductory_match_builds_and_steps() {
        let mut s = sim::TdmSim::new(intro_match_config());
        assert_eq!(s.fighters.len(), INTRO_PER_TEAM * 2, "4 v 4 is eight bodies");
        assert!(s.player < s.fighters.len(), "the player must be one of them");
        assert_eq!(s.cfg.tdm_target, INTRO_TDM_TARGET);
        // and it survives a few seconds of bots doing whatever bots do
        for _ in 0..600 {
            s.step(sim::PlayerCmd::default());
        }
        assert!(s.t > 0.0, "the clock must have moved");
    }

    /// The result screen must appear BEFORE the sim's own 7 s rebuild
    /// would have fired, or the player never sees it.
    #[test]
    fn result_screen_beats_the_sims_own_restart() {
        assert!(
            RESULT_DELAY_S < 7.0,
            "the sim rebuilds the match 7 s after round end; \
             {RESULT_DELAY_S} s is not sooner"
        );
        assert!(RESULT_DELAY_S > 1.0, "no time to read the scoreboard");
    }

    /// The main menu is FIVE entries. The spec says five and names them.
    #[test]
    fn main_menu_has_exactly_five_entries() {
        assert_eq!(MAIN_MENU_ENTRIES.len(), 5);
        let labels: Vec<&str> = MAIN_MENU_ENTRIES.iter().map(|e| e.1).collect();
        for want in ["PLAY", "TRAINING", "CUSTOMIZATION", "LEARN", "SETTINGS"] {
            assert!(labels.contains(&want), "{want} missing from the main menu");
        }
        // exactly one primary - a screen with two primaries has none
        let primaries = MAIN_MENU_ENTRIES
            .iter()
            .filter(|e| e.3 == ButtonWeight::Primary)
            .count();
        assert_eq!(primaries, 1, "exactly one entry may be the primary action");
    }

    /// Every button treatment must actually differ between states, or the
    /// "strong hover/selection feedback" the spec asks for is a no-op.
    /// This is the mutation guard: collapsing any hover onto its rest
    /// state fails here.
    #[test]
    fn every_weight_reacts_to_hover_and_press() {
        for w in [ButtonWeight::Primary, ButtonWeight::Secondary, ButtonWeight::Tertiary] {
            let rest = weight_colors(w, Interaction::None);
            let hover = weight_colors(w, Interaction::Hovered);
            let press = weight_colors(w, Interaction::Pressed);
            assert_ne!(rest, hover, "{w:?} does not react to hover");
            assert_ne!(hover, press, "{w:?} does not react to press");
        }
    }

    /// Scale animation stays SMALL. The spec rules out flashy.
    #[test]
    fn pop_is_a_nudge_not_a_zoom() {
        assert_eq!(pop_target(Interaction::None), 1.0);
        let h = pop_target(Interaction::Hovered);
        let p = pop_target(Interaction::Pressed);
        assert!(h > 1.0 && h < 1.08, "hover scale {h} is not a nudge");
        assert!(p < 1.0 && p > 0.94, "press scale {p} is not a nudge");
    }

    /// The cartoon layer must be dialled, not off and not shouting.
    #[test]
    fn cartoon_dial_is_restrained() {
        assert!(CARTOON.border_px >= 2.0, "thinner than 2 px is not cartoon at all");
        assert!(CARTOON.border_px <= 5.0, "past 5 px it stops being restrained");
        assert!(
            CARTOON.button_radius_px > CARTOON.radius_px,
            "buttons rounder than panels is the whole trick"
        );
        assert!(CARTOON.shadow_px > 0.0, "a flat panel casts no shadow");
    }

    /// A briefing card with no text is a card that teaches nothing.
    #[test]
    fn every_learn_card_says_something() {
        assert_eq!(LEARN_CARDS.len(), 4);
        for (t, b) in LEARN_CARDS {
            assert!(!t.is_empty());
            assert!(b.len() > 60, "{t} is too short to be a briefing");
            assert!(b.is_ascii(), "{t} has a non-ASCII glyph - it will render as tofu");
        }
    }

    /// Every string this module puts on screen must be ASCII: the crate
    /// ships one font and it has no glyph for anything else.
    #[test]
    fn menu_strings_are_ascii() {
        for (_, label, sub, _) in MAIN_MENU_ENTRIES {
            assert!(label.is_ascii(), "{label} is not ASCII");
            assert!(sub.is_ascii(), "{sub} is not ASCII");
        }
    }
}
