//! BRIEF XII — the in-match HUD, human POV vs mech POV.
//!
//! ## Why this is a module and not more of `main.rs`
//!
//! `main.rs` is 30k lines and is the most contested file in the repo. It
//! was dirty with another lane's ~1600 uncommitted lines while this was
//! written, so BRIEF XII was built the way `branding.rs` was: everything
//! lives here, and the wiring is exactly two lines someone else's diff
//! cannot conflict with.
//!
//! ```ignore
//! mod hud;
//! // ...
//! .add_plugins(hud::HudPlugin)
//! ```
//!
//! ## What it does to the OLD HUD
//!
//! It does not edit it — it cannot, the file was held. Instead
//! `suppress_legacy_hud` hides, every frame, the eight legacy widgets
//! this layer supersedes, and this module draws the replacements. The
//! legacy systems keep running and keep writing into hidden text, which
//! is deliberate: `hud_fade`'s `Local` snapshot query
//! (`PanelInfoText`/`PanelAmmoText`/`HudText`) still finds its three
//! entities, so the scar it carries — "the HUD used to fade to 45%
//! mid-firefight while piloting" — cannot reopen. This layer runs its
//! own copy of that fade over its own text (`hud_layer_fade`), off the
//! same snapshot plus heat, so a venting mech does not fade either.
//!
//! Hiding rather than deleting is also why every ASCII bar graph
//! (`GRIP [####......]`, `STRIDE`, `CHARGE`, `THROW`, `POD 4 / 6`,
//! `JAVELIN WIND`), every raw sub-second float, the raw extraction
//! coordinates, `pressure %`, `HORDE n`, `hits`, `[crouched]` and the
//! score-line parentheticals are off the screen: their host widgets are.
//! When `main.rs` is free those format strings should be deleted at
//! source and the suppression list shrunk to match. That is tracked in
//! `research/FRIDAY_LOG.md`.
//!
//! ## The rules the two reference images share (`handback/reference/hud/NOTES.md`)
//!
//! 1. The centre of the screen is never touched. `centre_is_clear`
//!    asserts it for every widget this module owns.
//! 2. Health and ammo are the two biggest glyphs, opposite bottom
//!    corners. `T_NUMERAL` is the new top of the type ramp and only
//!    those two use it.
//! 3. Current/reserve ammo is a big/small pair, never one string.
//! 4. Transients are unpanelled; permanent readings get a panel — and
//!    in HUMAN mode almost nothing is permanent, so almost nothing is
//!    panelled. That IS the human/mech split.
//! 5. Objectives live in world space (already true here — `CoverVis`,
//!    `HillVis`, `CheckpointVis` are world entities).
//! 6. One hue for threat, one for systems. `THREAT` and
//!    `frontend::palette::GOLD`. `NEON_RED`/`NEON_BLUE` stay
//!    faction-only, per their own doc comment, and appear in exactly
//!    one place here: the team score.
//!
//! ## Cosmetic only
//!
//! Nothing here writes sim state. Every system takes `Res<Game>`, never
//! `ResMut`.

use bevy::prelude::*;

use crate::frontend::palette;
use crate::frontend::{T_BODY, T_HEAD, T_MICRO, T_SUB};
use crate::{
    sim, ArmorSet, BannerText, CompassText, Game, GameState, HitFeedText, HudRoot, HudText,
    MechHudPiece, MechTargetBracket, Mode, PanelAmmoText, PanelInfoText, PrecisionChargeMark,
    PromptText, RangeText, ScoreTimerText, ARMOR_PIPS, ARMOR_PIP_REFERENCE,
    HUD_ANCHORS, HUD_SAFE_FRAC, MAX_HEALTH, POWER_MAX, VITALS_SEGMENTS,
};

// ---- the type ramp, extended ---------------------------------------------
//
// `frontend.rs` ships T_TITLE 54 / T_HEAD 30 / T_ACTION 26 / T_BODY 16 /
// T_SUB 13 / T_MICRO 11. There was a hole where a HUD numeral belongs:
// health and ammo were BOTH 34 px, tied with each other and with the
// banner, so the brief's rule 2 was unsatisfiable — nothing could be
// "the two biggest glyphs" when three things were the same size.
//
// Authored at 720p like the rest of the ramp; `UiScale` does the resolution.

/// The HUD numeral. The single largest glyph on screen, and ONLY health
/// and current-ammo may use it.
pub const T_NUMERAL: f32 = 76.0;
/// The small half of every big/small pair — reserve ammo, armour value.
/// Deliberately below `T_HEAD` so the pair can never read as equal.
pub const T_NUMERAL_SM: f32 = 26.0;

// ---- colour ---------------------------------------------------------------

/// THE threat hue. One, for the whole HUD.
///
/// The old HUD had twelve-plus hardcoded colours and used at least three
/// different reds. `palette::NEON_RED` is not it: its own doc says the
/// neons are faction accents and "not general-purpose UI colours", and
/// a red that means "enemy team" cannot also mean "you are dying".
/// This is the existing `vitals_color` red, promoted to a name.
const THREAT: Color = Color::srgb(1.0, 0.18, 0.15);

/// THE systems hue is `palette::GOLD`. Mech framing and the SELECTED
/// systems row are gold or `GOLD_DIM`; nothing else is.
///
/// It used to be every systems label. That is what "sparingly" was
/// supposed to prevent: four gold rows inside a gold-bordered plate
/// under a gold frame is not an accent, it is a colour scheme, and the
/// selected mount — the one row the pilot actually needs to find — had
/// no way left to stand out. Unselected rows are `INK_SOFT` now, so
/// gold marks exactly one thing again.
const SYSTEMS: Color = palette::GOLD;

/// Panel fill for a PERMANENT reading in MECH mode — reference rule 4.
fn plate() -> Color {
    palette::PANEL.with_alpha(0.72)
}

/// XII-A: the HUMAN plate.
///
/// It used to be `Color::NONE` — bare white type sitting directly on the
/// world, which is exactly the failure mode the owner called
/// "washed out" on the legacy strip: unreadable over pale sand. The
/// brief's rule is that every PERMANENT reading gets a plate or a dark
/// outline, and health/ammo are the two most permanent readings in the
/// build.
///
/// This is a scrim, not a panel: less than half the mech plate's opacity
/// and no border at all, so the human HUD keeps its "flat on the world"
/// density (reference img2) while the numerals stop competing with sand.
pub(crate) fn scrim() -> Color {
    palette::PANEL.with_alpha(0.30)
}

/// §5: THE HUMAN PANEL, shared by the bottom-left vitals cluster and the
/// bottom-right icon/numeral/name group.
///
/// This was `inventory_strip::strip_plate` and it has moved here for one
/// reason: after §5 the strip and the ammo numeral are children of a
/// single panel, so there is only one rectangle left to colour, and the
/// module that owns that rectangle is this one. It is still expressed as
/// a MULTIPLE of `scrim()` rather than as a fresh literal, so retuning
/// the HUD's one scrim value still moves it.
///
/// Why the stronger value won: `scrim()` is tuned for 76 px white
/// glyphs, which carry themselves over pale sand. §4 photographed the
/// strip's 3 px `INK_SOFT` strokes at that alpha and they washed out to
/// smudges. One panel means one alpha, and the icons are the thing that
/// needs it.
pub(crate) fn group_plate() -> Color {
    let s = scrim();
    s.with_alpha((s.alpha() * 1.85).min(1.0))
}

// ---- anchors --------------------------------------------------------------
//
// `HUD_ANCHORS` in `main.rs` is right and is already data-driven:
// fractional offsets, so the 5% safe area holds at every resolution by
// construction. It is EXTENDED here, not replaced — the five corner
// names below are read out of it by name.
//
// These four extra slots belong in that same table. They are here only
// because `main.rs` was held by another lane; merging them is a
// mechanical move once it is free.

/// The BRIEF XII additions. Same shape as `HUD_ANCHORS`:
/// `(name, anchor 0..1, offset in screen fractions)`.
const XII_ANCHORS: &[(&str, [f32; 2], [f32; 2])] = &[
    // one urgent line, top centre, unpanelled — reference img1
    ("urgent", [0.5, 0.0], [0.0, 0.075]),
    // the kill/hit feed, moved out of the middle of the screen
    ("transient", [0.0, 1.0], [0.06, -0.28]),
    // equipment state, top-left — the slot the reference gives it, and
    // the slot K/D was squatting in (K/D lives on the scoreboard)
    ("equip", [0.0, 0.0], [0.06, 0.06]),
];

/// §5 THE MECH-SYSTEMS ANCHOR IS GONE, and its absence is the change.
///
/// It used to place a floating panel at `[-0.06, -0.24]` — a fourth
/// competing cluster hanging in space above the ammo corner, with its
/// own plate, its own border and its own alignment. The systems rows now
/// live INSIDE the bottom-right group panel, stacked over the numeral
/// exactly where the inventory strip sits on foot, so mech and infantry
/// read as the same instrument in two states rather than two designs.
///
/// Look XII anchors up BY NAME. `spawn_top`/`spawn_equip` indexed this
/// table (`XII_ANCHORS[3].2`), which meant deleting the row above would
/// have silently moved the top-left panel to the bottom-right corner.
fn xii(name: &str) -> [f32; 2] {
    XII_ANCHORS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, _, o)| *o)
        .unwrap_or([HUD_SAFE_FRAC, HUD_SAFE_FRAC])
}

/// Resolve an anchor to a pixel point. Pure; the layout tests use it.
fn anchor_px(anchor: [f32; 2], off: [f32; 2], w: f32, h: f32) -> (f32, f32) {
    (w * (anchor[0] + off[0]), h * (anchor[1] + off[1]))
}

/// Look a corner up in `HUD_ANCHORS` BY NAME rather than by index.
///
/// The spawn code in `main.rs` indexes it (`HUD_ANCHORS[0].2[0]`), which
/// means inserting a row into that table silently moves four widgets.
/// This cannot.
fn corner(name: &str) -> ([f32; 2], [f32; 2]) {
    HUD_ANCHORS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, a, o)| (*a, *o))
        .unwrap_or(([0.0, 0.0], [HUD_SAFE_FRAC, HUD_SAFE_FRAC]))
}

/// Is a point inside the centre third of the screen — the region both
/// references keep absolutely clear?
fn in_centre_third(x: f32, y: f32, w: f32, h: f32) -> bool {
    x > w / 3.0 && x < w * 2.0 / 3.0 && y > h / 3.0 && y < h * 2.0 / 3.0
}

// ---- pure formatters ------------------------------------------------------

/// Gatling heat as a percent, for BOTH chassis.
///
/// **The sim stores this field on two different scales and that is not a
/// HUD bug.** The heavy turret path clamps `gatling_heat` at `100.0`
/// (`sim.rs`, `f.gatling_heat >= 100.0`) and divides by 100 when it uses
/// it; the medic's plasma and repair paths clamp it at `1.0`. So the old
/// HUD printing it raw in the turret branch and `* 100.0` in the medic
/// branches was CORRECT in both places — and unreadable, because one
/// field name meant two things.
///
/// The honest client-side fix is one helper that knows which chassis it
/// is looking at. The real fix is two fields in the sim, which is
/// friday22's lane; this is logged for them rather than papered over.
fn heat_pct(gatling_heat: f32, scout_chassis: bool) -> f32 {
    if scout_chassis {
        (gatling_heat * 100.0).clamp(0.0, 100.0)
    } else {
        gatling_heat.clamp(0.0, 100.0)
    }
}

/// Split ammo into the big/small pair the references require. Returns
/// `(current, reserve)` already formatted; they are rendered as two
/// entities at two sizes, never as one string.
fn ammo_pair(ammo: u32, reserve: u32) -> (String, String) {
    (format!("{ammo}"), format!("{reserve}"))
}

/// The display name of a hull mount. Cosmetic labelling only — the sim
/// has no `name()` on `MechWeapon`, and this is the only place this
/// module spells them.
fn mount_name(w: sim::MechWeapon) -> &'static str {
    match w {
        sim::MechWeapon::Gatling => "TURRET",
        sim::MechWeapon::Autocannon => "AUTOCANNON",
        sim::MechWeapon::Rockets => "ROCKETS",
        sim::MechWeapon::Plasma => "PLASMA",
        sim::MechWeapon::Repair => "REPAIR",
    }
}

/// The one-line plate-condition reading, from the stages the SIM already
/// computed.
///
/// This takes `Option<ArmorStage>` per mount because that is exactly what
/// `Sim::armor_stage_of` returns, and the `None` case carries real
/// meaning: no plate on that mount at all, either never fitted or shot
/// off. A missing plate is NOT a fresh one, so `None` is counted out of
/// the denominator rather than folded into `Fresh`.
///
/// Nothing here re-derives a threshold. `PLATE_SCUFFED_FRAC` and friends
/// live in `sim.rs` and stay there — this only reports the worst stage
/// currently on the body and how many mounts are below `Fresh`.
///
/// `None` return = no plates equipped at all, and the caller prints
/// nothing rather than a row of zeroes.
fn plate_condition(stages: &[Option<sim::ArmorStage>]) -> Option<(sim::ArmorStage, usize, usize)> {
    let equipped: Vec<sim::ArmorStage> = stages.iter().flatten().copied().collect();
    // `ArmorStage`'s `Ord` derive is worst-last by construction (the sim
    // documents the variant order as the order they are REACHED in), so
    // `max` IS "the worst plate on the body" and not a coincidence.
    let worst = *equipped.iter().max()?;
    let worn = equipped
        .iter()
        .filter(|s| **s != sim::ArmorStage::Fresh)
        .count();
    Some((worst, worn, equipped.len()))
}

/// The text for that reading. Split from the colour so both are testable
/// without a `World`.
fn plate_condition_text(worst: sim::ArmorStage, worn: usize, total: usize) -> String {
    if worst == sim::ArmorStage::Fresh {
        format!("PLATES {}", worst.label())
    } else {
        // `label` is the sim's string, never a retyped copy — same rule
        // the manual screen's weapon table follows.
        format!("PLATES {} {worn}/{total}", worst.label())
    }
}

/// One hue for threat, one for systems — reference rule 6. A cracked or
/// severed plate is the only plate state that is a THREAT; scuffed is a
/// systems reading that has merely dimmed.
fn plate_condition_color(worst: sim::ArmorStage) -> Color {
    if worst.tilts() {
        THREAT
    } else if worst == sim::ArmorStage::Fresh {
        SYSTEMS
    } else {
        palette::GOLD_DIM
    }
}

/// The mech systems column — BRIEF XII-A: this IS the folded weapon
/// strip, and it carries ONLY what is not already on the screen.
///
/// Deleted from it in XII-A: the `HULL` row (the bottom-left numeral is
/// directly above it) and the `HEAT` row (the bottom-right numeral is
/// directly above it). Heat now appears exactly once, and a mount whose
/// resource IS heat prints no quantity here at all — printing it would
/// re-create the third copy this pass exists to remove.
///
/// `mounts` comes from `MechWeapon::for_set`, never a hardcoded pair:
/// the old strip printed `TURRET 0 / ROCKETS 0` inside a medic for
/// exactly that reason.
///
/// `shield` is `None` unless a shield is actually UP — the owner's
/// "everything else stays hidden unless needed".
fn systems_lines(
    mounts: &[sim::MechWeapon],
    selected: sim::MechWeapon,
    rounds: u32,
    pod: u8,
    locked: bool,
    shield: Option<(&'static str, String)>,
) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = Vec::new();
    for w in mounts {
        // ASCII only — no font asset ships, so every ornament is a drawn
        // node or a plain character. '>' is the marker the old strip used.
        let is_sel = *w == selected;
        let label = format!(
            "{} {}",
            if is_sel { SELECTED_MARK } else { " " },
            mount_name(*w)
        );
        // THE SELECTED MOUNT NEVER PRINTS ITS QUANTITY. Whatever resource
        // it spends — rounds, pods or heat — is already the bottom-right
        // numeral, directly below this row and eight times the size.
        //
        // The first consolidation pass applied this to heat mounts only,
        // reasoning that heat was the duplicate. It was the wrong cut: a
        // heavy on its turret showed `> TURRET 300` immediately above a
        // `300` numeral, which is the same defect wearing a different
        // unit. Caught in `handback/brief-vii/hud_contrast/05-level-south.png`
        // by looking at the frame rather than at the diff.
        //
        // An UNSELECTED mount still prints its count, and should: "how
        // many rockets are waiting" is the one thing the column knows
        // that the numeral cannot say.
        let qty = if is_sel {
            String::new()
        } else {
            match w {
                sim::MechWeapon::Gatling => format!("{rounds}"),
                sim::MechWeapon::Rockets => format!("{pod}"),
                sim::MechWeapon::Autocannon
                | sim::MechWeapon::Plasma
                | sim::MechWeapon::Repair => String::new(),
            }
        };
        v.push((label, qty));
    }
    if locked {
        v.push(("LOCK".to_string(), "TRACKING".to_string()));
    }
    if let Some((label, value)) = shield {
        v.push((label.to_string(), value));
    }
    v
}

/// The ASCII marker on the selected mount's row. No font asset ships, so
/// every ornament is a drawn node or a plain character — `main.rs`
/// documents U+271A rendering as a tofu box in the bundled font.
///
/// `paint_systems` matches on this to decide which single row gets the
/// gold, so the marker and the colour cannot disagree.
const SELECTED_MARK: &str = ">";

/// How many rows the systems column can ever need: two mounts, `LOCK`,
/// and a shield line.
const SYSTEMS_ROWS: usize = 4;

/// Where the legacy contextual prompt sits, in percent from the bottom.
///
/// It was 20%, which is inside the numeral band — see
/// `reanchor_legacy_centred_text`. `prompt_clears_the_numerals` is the
/// test, and it fails at 20.
const PROMPT_BOTTOM_PCT: f32 = 32.0;

/// The height of a bottom-corner numeral cluster, in percent of a 720p
/// screen: the numeral itself plus the two bar rows and the padding.
/// Used only to prove the prompt clears it.
fn numeral_cluster_height_pct() -> f32 {
    // T_NUMERAL glyph box + segment bar + armour pip row + row gaps +
    // 10 px of padding top and bottom, over 720.
    (T_NUMERAL + 6.0 + 10.0 + 4.0 + 3.0 + 20.0) / 720.0 * 100.0
}

/// The single top-centre urgent line. ONE line, highest priority only.
///
/// The old top-centre slot carried the clock, the score, a parenthetical
/// rules reminder, a tutorial sentence, a horde count, an internal
/// pressure scalar and raw world coordinates — simultaneously.
fn urgent_line(
    alive: bool,
    round_over: bool,
    overtime: bool,
    hull_frac: Option<f32>,
    health_frac: f32,
    venting: bool,
) -> String {
    if round_over {
        return "ROUND OVER".to_string();
    }
    if !alive {
        return "DOWN".to_string();
    }
    if let Some(f) = hull_frac {
        if f <= 0.25 {
            return "HULL CRITICAL".to_string();
        }
        if venting {
            return "MOUNT VENTING".to_string();
        }
    } else if health_frac <= 0.25 {
        return "CRITICAL".to_string();
    }
    if overtime {
        return "SUDDEN DEATH".to_string();
    }
    String::new()
}

/// The score line. Team names only, no parentheticals, no prose, no raw
/// coordinates, no internal scalars.
fn score_line(mode: Mode, blue: f32, red: f32, horde: usize) -> (String, String) {
    match mode {
        Mode::Tdm => (format!("{:.0}", blue), format!("{:.0}", red)),
        Mode::Koth => (format!("{:.0}s", blue), format!("{:.0}s", red)),
        Mode::Training => ("RANGE".to_string(), String::new()),
        Mode::Extraction => (format!("{horde}"), "HORDE".to_string()),
    }
}

// ---- the mech frame -------------------------------------------------------
//
// The old corner brackets used `Val::Percent` on BOTH axes for the same
// bracket — an 8%-of-WIDTH horizontal arm meeting a 6%-of-HEIGHT
// vertical one. At 16:9 that is roughly square; at 21:9 the horizontal
// arms go long-and-thin and the vertical ones fat-and-short, so the
// "machine framing" visibly deforms with the window. Machine framing
// that is not square does not read as precise.
//
// Every dimension below is PIXELS, authored at 720p, scaled by `UiScale`
// exactly like the type ramp. `frame_bar` is therefore independent of
// aspect by construction, and `frame_is_aspect_invariant` proves it.

/// Arm length of one bracket stroke, 720p px.
const FRAME_ARM_PX: f32 = 92.0;
/// Stroke thickness, 720p px.
///
/// §5: 3 px of near-saturated `GOLD` at 0.85 alpha in all four corners
/// is not framing, it is a fifth thing on the screen shouting the loudest
/// colour in the palette. A frame's whole job is to be noticed once and
/// then ignored. Two pixels of `FRAME_INK` does that.
const FRAME_THICK_PX: f32 = 2.0;

/// The frame's colour. Not `SYSTEMS`.
///
/// The owner's note on the mech view was "more simple... professional
/// similar to our old designs", against a reference whose entire chrome
/// is neutral line-work with ONE accent. `SYSTEMS` (= `palette::GOLD`)
/// has exactly one job left in this file after §5 — marking the SELECTED
/// mount and the power-core pips — and a colour spent on eight
/// decorative bars in the corners of the screen is a colour that can no
/// longer mark anything. `gold_is_spent_on_selection_only` is the test.
fn frame_ink() -> Color {
    palette::INK_SOFT.with_alpha(0.38)
}
/// Inset of the bracket corner from the screen edge, 720p px.
const FRAME_INSET_PX: f32 = 26.0;

/// The eight strokes of the mech frame: four corners x (horizontal,
/// vertical). Returns `(left, top, width, height)` in 720p px, where
/// `left`/`top` may be measured from the far edge — see `frame_bar_node`.
///
/// Pure and aspect-free on purpose: the test asserts the SAME numbers
/// come back at 16:9 and 21:9, which a percent-based implementation
/// could not do.
fn frame_bar(i: usize) -> (f32, f32, f32, f32) {
    let horizontal = i % 2 == 0;
    if horizontal {
        (
            FRAME_INSET_PX,
            FRAME_INSET_PX,
            FRAME_ARM_PX,
            FRAME_THICK_PX,
        )
    } else {
        (
            FRAME_INSET_PX,
            FRAME_INSET_PX,
            FRAME_THICK_PX,
            FRAME_ARM_PX,
        )
    }
}

/// Turn stroke `i` into an absolutely-positioned node. Corner `i / 2`:
/// 0 = top-left, 1 = top-right, 2 = bottom-left, 3 = bottom-right.
fn frame_bar_node(i: usize) -> Node {
    let (l, t, w, h) = frame_bar(i);
    let corner = i / 2;
    let left_side = corner % 2 == 0;
    let top_side = corner < 2;
    Node {
        position_type: PositionType::Absolute,
        left: if left_side { Val::Px(l) } else { Val::Auto },
        right: if left_side { Val::Auto } else { Val::Px(l) },
        top: if top_side { Val::Px(t) } else { Val::Auto },
        bottom: if top_side { Val::Auto } else { Val::Px(t) },
        width: Val::Px(w),
        height: Val::Px(h),
        ..default()
    }
}

// ---- state ----------------------------------------------------------------

/// Which HUD the player is looking at. There was no such branch before:
/// `mech_hud_sync` faded some brackets in at alpha 0.42 and swapped two
/// strings, so a still frame of the mech HUD was distinguishable from
/// the human one only by reading the word `HULL`.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
enum HudMode {
    #[default]
    Human,
    Mech,
}

fn hud_mode_of(in_mech: bool, alive: bool) -> HudMode {
    if in_mech && alive {
        HudMode::Mech
    } else {
        HudMode::Human
    }
}

// ---- components -----------------------------------------------------------

#[derive(Component)]
struct XiiRoot;
/// Shown only in mech mode.
#[derive(Component)]
struct MechOnly;
/// Every text this module fades. (The crosshair is deliberately not in
/// this set — rule from Brief III: the crosshair never fades.)
#[derive(Component)]
pub(crate) struct XiiFade;

#[derive(Component)]
struct HpNumber;
#[derive(Component)]
struct HpSeg(usize);
#[derive(Component)]
struct ArmourPip(usize);
/// The plate damage-state line under the pip row. HUMAN mode only — a
/// chassis has no plates, it has a power core, and the pips already say
/// so.
#[derive(Component)]
struct PlateCondText;
#[derive(Component)]
struct AmmoNumber;
#[derive(Component)]
struct AmmoReserve;
#[derive(Component)]
struct AmmoSub;
#[derive(Component)]
struct UrgentText;
#[derive(Component)]
struct ClockText;
#[derive(Component)]
struct TeamScoreText(usize);
#[derive(Component)]
struct EquipText;
#[derive(Component)]
struct TransientText;
#[derive(Component)]
struct SystemsRow(usize);
#[derive(Component)]
struct SystemsLabel(usize);
#[derive(Component)]
struct SystemsValue(usize);
/// The vitals / ammo containers, so the plate can appear in mech mode
/// and vanish in human mode.
#[derive(Component)]
struct XiiPlate;

/// THE HOST SLOT for `inventory_strip`'s row, inside the bottom-right
/// group panel and directly above the ammo numeral.
///
/// Published so that module can find it at `Startup` without either
/// module knowing the other's layout. It is empty here on purpose: if
/// `InventoryStripPlugin` is not in the app it stays `Display::None` and
/// the panel simply hugs the numeral, which is what mech mode wants
/// anyway.
#[derive(Component)]
pub(crate) struct AmmoStripHost;

// ---- plugin ---------------------------------------------------------------

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudMode>()
            .init_resource::<XiiFadeAlpha>()
            .add_systems(Startup, spawn_layer)
            .add_systems(
                Update,
                (
                    suppress_legacy_hud,
                    reanchor_legacy_centred_text,
                    mode_sync,
                    paint_vitals,
                    paint_ammo,
                    paint_top,
                    paint_equip,
                    paint_transient,
                    paint_systems,
                    layer_fade,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---- spawn ----------------------------------------------------------------

fn text(size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(""),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

pub(crate) fn spawn_layer(mut commands: Commands) {
    let (v_a, v_o) = corner("vitals");
    let (a_a, a_o) = corner("ammo");

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            // The initial state is Title, and `hud_visibility` only runs
            // on Playing's enter/exit — so a root that spawned visible
            // would sit on top of the title screen until the first
            // match. Start hidden; `HudRoot` flips it to Inherited.
            Visibility::Hidden,
            HudRoot,
            XiiRoot,
        ))
        .with_children(|r| {
            spawn_mech_frame(r);
            spawn_vitals(r, v_a, v_o);
            // §5: `spawn_ammo` now builds the whole bottom-right GROUP —
            // the strip's host slot, the systems column and the numerals
            // — so `spawn_systems` is no longer a top-level call.
            spawn_ammo(r, a_a, a_o);
            spawn_top(r);
            spawn_equip(r);
            spawn_transient(r);
        });
}

fn spawn_mech_frame(r: &mut ChildBuilder) {
    for i in 0..8 {
        r.spawn((
            frame_bar_node(i),
            BackgroundColor(frame_ink()),
            Visibility::Hidden,
            MechOnly,
        ));
    }
}

fn spawn_vitals(r: &mut ChildBuilder, a: [f32; 2], o: [f32; 2]) {
    r.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent((a[0] + o[0]) * 100.0),
            bottom: Val::Percent(-(a[1] + o[1] - 1.0) * 100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        // human mode paints these to NONE; mech mode to plate()/GOLD_DIM
        BackgroundColor(Color::NONE),
        BorderColor(Color::NONE),
        XiiPlate,
    ))
    .with_children(|p| {
        // the health numeral. Reference rule 2: one of the two biggest
        // glyphs on the screen, bottom-left corner, nothing beside it.
        p.spawn((
            text(T_NUMERAL, palette::INK),
            TextLayout::new_with_no_wrap(),
            HpNumber,
            XiiFade,
        ));
        // segmented bar. A solid bar shows a ratio; a segmented one
        // shows a COUNT — this idea is kept from the old HUD, which is
        // §1's instruction (reuse what was stronger).
        p.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(3.0),
                ..default()
            },
            HullBarRow,
        ))
        .with_children(|b| {
            for i in 0..VITALS_SEGMENTS {
                b.spawn((
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(palette::INK),
                    HpSeg(i),
                ));
            }
        });
        p.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                margin: UiRect::top(Val::Px(3.0)),
                ..default()
            },
            HullBarRow,
        ))
        .with_children(|b| {
            for i in 0..ARMOR_PIPS {
                b.spawn((
                    Node {
                        width: Val::Px(10.0),
                        height: Val::Px(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(palette::GOLD_DIM),
                    BackgroundColor(Color::NONE),
                    ArmourPip(i),
                ));
            }
        });
        // XII: the plate CONDITION line. The pip row above it is fed by
        // `armor_spec(set).flat_torso`, which is a property of the SET
        // and never changes once a match starts — so before this line
        // the on-foot HUD had no way at all to say a plate had been
        // shot. `armor_stage_of` had 66 call sites in the sim and zero
        // readers anywhere in the client.
        p.spawn((
            text(T_MICRO, SYSTEMS),
            TextLayout::new_with_no_wrap(),
            PlateCondText,
            XiiFade,
        ));
    });
}

/// Marker for the two bar rows inside the vitals cluster; they exist so
/// the rows can be laid out, nothing reads them.
#[derive(Component)]
struct HullBarRow;

/// THE BOTTOM-RIGHT GROUP — §5's second task, and the mech's third.
///
/// ## What the reference actually shows
///
/// One dark translucent rounded panel containing, top to bottom: the
/// weapon icon and its small companions, the large ammo numeral with a
/// smaller dimmer reserve beside it, and the weapon name in small caps
/// underneath. ONE panel. §4 shipped the strip's plate sitting above the
/// ammo cluster's plate — two rectangles of different widths and
/// different alphas with a seam between them, which
/// `handback/brief-vii/grenade_hold/01-fp-rifle-before-g.png` shows
/// clearly and which §4's own handback called out as its second gap.
///
/// ## Why a HOST SLOT and not a merge
///
/// The ammo cluster serves MECH mode too, where the strip is hidden
/// entirely, so folding the numeral into `inventory_strip` would have
/// deleted the mech readout to fix the infantry one — the trap
/// `count_text`'s doc comment already refuses. Instead this function
/// spawns the shared panel and two EMPTY child slots above the numeral:
///
/// * `AmmoStripHost`, which `inventory_strip::spawn_strip` parents its
///   row into at `Startup`, and
/// * the systems column, mech only.
///
/// Neither module has to know the other's layout, the panel auto-sizes
/// to whichever slot is displayed, and a harness that builds `HudPlugin`
/// without `InventoryStripPlugin` (or the reverse) still works — the
/// strip keeps its absolute-positioned fallback root.
///
/// The slots use `Display::None` when idle rather than
/// `Visibility::Hidden`, because a hidden node still occupies its box
/// and the panel would stay strip-height in a mech with nothing in it.
fn spawn_ammo(r: &mut ChildBuilder, a: [f32; 2], o: [f32; 2]) {
    r.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(-(a[0] + o[0] - 1.0) * 100.0),
            bottom: Val::Percent(-(a[1] + o[1] - 1.0) * 100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(2.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor(Color::NONE),
        // The reference's rounded panel. One radius, on the one panel.
        BorderRadius::all(Val::Px(6.0)),
        XiiPlate,
    ))
    .with_children(|g| {
        // slot 1: the infantry inventory strip parents itself here
        g.spawn((
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            AmmoStripHost,
        ));
        // slot 2: the mech systems column
        spawn_systems(g);
        // slot 3: the numeral pair and the weapon name
        g.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(2.0),
                ..default()
            },
            HullBarRow,
        ))
        .with_children(|p| {
            spawn_ammo_numerals(p);
        });
    });
}

fn spawn_ammo_numerals(p: &mut ChildBuilder) {
    {
        // THE BIG/SMALL PAIR. Two entities at two sizes — the old HUD
        // had `format!("{ammo} / {reserve}")`, one string at one size,
        // which is the one thing both references forbid outright.
        p.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                column_gap: Val::Px(6.0),
                ..default()
            },
            HullBarRow,
        ))
        .with_children(|row| {
            row.spawn((
                text(T_NUMERAL, palette::INK),
                TextLayout::new_with_no_wrap(),
                AmmoNumber,
                XiiFade,
            ));
            row.spawn((
                Node {
                    margin: UiRect::bottom(Val::Px(14.0)),
                    ..default()
                },
                // XII-A: INK_SOFT, not INK_FAINT. 0.42 grey against pale
                // sand is the washed-out failure in miniature; the pair
                // still reads as big/small because the SIZES differ by
                // 3x, which is the property rule 3 actually asks for.
                text(T_NUMERAL_SM, palette::INK_SOFT).0,
                TextFont {
                    font_size: T_NUMERAL_SM,
                    ..default()
                },
                TextColor(palette::INK_SOFT),
                TextLayout::new_with_no_wrap(),
                AmmoReserve,
                XiiFade,
            ));
        });
        p.spawn((
            text(T_SUB, palette::INK_SOFT),
            TextLayout::new_with_no_wrap(),
            AmmoSub,
            XiiFade,
        ));
    }
}

fn spawn_top(r: &mut ChildBuilder) {
    // Row 0 is "urgent". Only its OFFSET is used: the anchor's x is 0.5
    // and the rail below centres on its own width, which is the correct
    // way to centre and does not need the anchor to say so twice.
    let u_off = xii("urgent");
    // The centred-rail idiom already in `main.rs` twice: a full-width
    // row with `justify_content: Center`. NOT `left: Percent(30.0)`,
    // which is what the banner, hit feed, compass and range text all
    // used and which slides off-centre the moment the aspect changes.
    r.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Percent(u_off[1] * 100.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            ..default()
        },
        HullBarRow,
    ))
    .with_children(|p| {
        // ONE urgent line, no panel behind it — reference rule 4.
        p.spawn((
            text(T_HEAD, THREAT),
            TextLayout::new_with_no_wrap(),
            UrgentText,
        ));
        // clock + score, small, because §6 puts them below health/ammo
        p.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            HullBarRow,
        ))
        .with_children(|row| {
            // NEON_BLUE/NEON_RED are faction-only by their own doc.
            // This is the one place in the HUD that is about faction.
            row.spawn((
                text(T_BODY, palette::NEON_BLUE),
                TextLayout::new_with_no_wrap(),
                TeamScoreText(0),
            ));
            row.spawn((
                text(T_BODY, palette::INK),
                TextLayout::new_with_no_wrap(),
                ClockText,
            ));
            row.spawn((
                text(T_BODY, palette::NEON_RED),
                TextLayout::new_with_no_wrap(),
                TeamScoreText(1),
            ));
        });
    });
}

fn spawn_equip(r: &mut ChildBuilder) {
    let off = xii("equip");
    r.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(off[0] * 100.0),
            top: Val::Percent(off[1] * 100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
            ..default()
        },
        // XII-A: a scrim behind the one PERMANENT top-left reading.
        BackgroundColor(scrim()),
        HullBarRow,
    ))
    .with_children(|p| {
        // flat rows, no box — reference img2's top-left. This corner
        // used to print `K/D 3/1  hits 27  [crouched]`: a counter
        // nobody acts on and a state-machine name. K/D is on the
        // scoreboard, which already has K/A/D/DMG columns.
        p.spawn((
            text(T_SUB, palette::INK_SOFT),
            TextLayout::new_with_no_wrap(),
            EquipText,
            XiiFade,
        ));
    });
}

fn spawn_transient(r: &mut ChildBuilder) {
    let off = xii("transient");
    r.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(off[0] * 100.0),
            bottom: Val::Percent(-off[1] * 100.0),
            ..default()
        },
        // NO panel. Reference rule 4, and img2 states it explicitly:
        // "chat and event text with no panel behind it at all".
        text(T_BODY, palette::INK_SOFT).0,
        TextFont {
            font_size: T_BODY,
            ..default()
        },
        TextColor(palette::INK_SOFT),
        TransientText,
    ));
}

/// The mech systems column — now a SLOT inside the bottom-right group
/// panel rather than a floating panel of its own.
///
/// §5, the owner's "mech hud to be more simple". It used to be a
/// separate plate with its own background, its own 2 px border and its
/// own anchor, hanging above the ammo cluster's separate plate, which
/// was itself opposite the hull cluster in the other corner — inside a
/// gold frame. That is the four competing clusters.
///
/// **Every row it printed, it still prints.** Nothing here deletes a
/// reading: `systems_lines` is untouched, so the mounts, their counts,
/// `LOCK TRACKING` and `BARRIER` all survive verbatim. What went is the
/// CHROME — the second plate, the second border and the gap between them
/// — plus the floating anchor. The rows now sit on the one group panel
/// in exactly the relationship the infantry strip has to the same
/// numeral, so the two modes read as one instrument in two states.
///
/// `Display::None` rather than `Visibility::Hidden`: a hidden node still
/// occupies its box, and the panel would stay systems-height on foot
/// with nothing in it.
fn spawn_systems(r: &mut ChildBuilder) {
    r.spawn((
        Node {
            display: Display::None,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(2.0),
            margin: UiRect::bottom(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        MechOnly,
    ))
    .with_children(|p| {
        for i in 0..SYSTEMS_ROWS {
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                SystemsRow(i),
            ))
            .with_children(|row| {
                row.spawn((
                    text(T_MICRO, SYSTEMS),
                    TextLayout::new_with_no_wrap(),
                    SystemsLabel(i),
                ));
                row.spawn((
                    text(T_SUB, palette::INK),
                    TextLayout::new_with_no_wrap(),
                    SystemsValue(i),
                    XiiFade,
                ));
            });
        }
        // XII-A: the two drawn bars that used to sit here are gone. The
        // hull bar repeated the segmented bar in the bottom-left
        // cluster, and the heat bar was the second of the three heat
        // displays this pass removes. Nothing in this column now
        // restates a number that is already on the screen.
    });
}

// ---- systems --------------------------------------------------------------

/// Hide the legacy widgets this layer supersedes.
///
/// Every frame, not once: `hud_visibility` re-asserts `Inherited` on
/// every `HudRoot` on each entry to `Playing`, and all of these carry
/// `HudRoot`. A one-shot hide would come back on the first resume.
#[allow(clippy::type_complexity)]
fn suppress_legacy_hud(
    panels: Query<&Parent, Or<(With<PanelInfoText>, With<PanelAmmoText>)>>,
    mut vis: Query<&mut Visibility>,
    marks: Query<
        Entity,
        Or<(
            With<HudText>,
            With<ScoreTimerText>,
            With<BannerText>,
            With<HitFeedText>,
            With<RangeText>,
            With<MechHudPiece>,
            With<PrecisionChargeMark>,
        )>,
    >,
) {
    for parent in &panels {
        if let Ok(mut v) = vis.get_mut(parent.get()) {
            if *v != Visibility::Hidden {
                *v = Visibility::Hidden;
            }
        }
    }
    for e in &marks {
        if let Ok(mut v) = vis.get_mut(e) {
            if *v != Visibility::Hidden {
                *v = Visibility::Hidden;
            }
        }
    }
}

/// Two legacy widgets are KEPT — the contextual prompt and the compass —
/// because their content is owned by systems in `main.rs` that this
/// module has no business duplicating. Both were hardcoded to
/// `left: Percent(30.0)` / `Percent(44.0)`, i.e. hand-tuned to look
/// centred at 16:9 only. Re-anchoring them to the full-width centred
/// rail is a LAYOUT change, which is this lane; their text is not
/// touched.
///
/// Runs every frame and is idempotent — cheaper than a one-shot with a
/// `Local` flag, and immune to respawned widgets.
fn reanchor_legacy_centred_text(
    mut prompt: Query<&mut Node, (With<PromptText>, Without<CompassText>)>,
    mut compass: Query<&mut Node, (With<CompassText>, Without<PromptText>)>,
    mut done: Local<bool>,
    mut commands: Commands,
    p_e: Query<Entity, With<PromptText>>,
    c_e: Query<Entity, With<CompassText>>,
) {
    if *done {
        return;
    }
    let mut any = false;
    for mut n in &mut prompt {
        n.left = Val::Px(0.0);
        n.width = Val::Percent(100.0);
        // XII-A: and UP, out of the numeral band.
        //
        // The capture caught the boarding prompt — "BIG MECH BOARDED -
        // 1/2: MOUNTS - C: REPULSOR - U: DISMOUNT - protect your REAR"
        // — running the full width of the screen at bottom 20%, which is
        // exactly the row the two big numerals occupy. It struck through
        // the hull numeral on the left and the heat numeral on the
        // right in every mech frame. That is the owner's "overlapping
        // elements" in one line.
        n.bottom = Val::Percent(PROMPT_BOTTOM_PCT);
        any = true;
    }
    for mut n in &mut compass {
        n.left = Val::Px(0.0);
        n.width = Val::Percent(100.0);
        any = true;
    }
    for e in &p_e {
        commands
            .entity(e)
            .insert(TextLayout::new_with_justify(JustifyText::Center));
    }
    for e in &c_e {
        commands
            .entity(e)
            .insert(TextLayout::new_with_justify(JustifyText::Center));
    }
    if any {
        *done = true;
    }
}

/// The human/mech branch. This is §4/§5's "immediately recognizable".
#[allow(clippy::type_complexity)]
fn mode_sync(
    game: Res<Game>,
    mut mode: ResMut<HudMode>,
    // §5: the ONE idle clock, read (never re-timed) so the panels dim
    // with the type they sit behind. §4 fixed exactly this desync for
    // the strip's own plate; now that the strip has no plate of its own,
    // the fix has to live where the surviving plate does.
    fade: Res<XiiFadeAlpha>,
    mut mech_only: Query<(&mut Visibility, &mut Node), With<MechOnly>>,
    mut plates: Query<(&mut BackgroundColor, &mut BorderColor), With<XiiPlate>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let m = hud_mode_of(p.in_mech(), p.alive());
    *mode = m;
    let mech = m == HudMode::Mech;
    for (mut v, mut node) in &mut mech_only {
        let want = if mech {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
        // §5: DISPLAY as well as visibility. The systems column is a
        // child of the bottom-right group panel now, and a merely hidden
        // child still occupies its box — the panel would stand four rows
        // taller than its contents for the whole of an infantry match.
        // The frame bars are absolutely positioned, so this is a no-op
        // for them.
        let want_d = if mech { Display::Flex } else { Display::None };
        if node.display != want_d {
            node.display = want_d;
        }
    }
    // Reference rule 4, and the whole of the density split: in HUMAN
    // mode the numbers sit on a bare scrim with no border and no
    // chrome. In MECH mode the same clusters get a full panel and a
    // gold rule, because a machine frames its readouts.
    //
    // XII-A moved human from `Color::NONE` to `scrim()`. The density
    // split survives — 0.30 against 0.72, borderless against ruled —
    // but a white numeral over pale sand is now legible, which was the
    // owner's "washed-out" complaint and which no amount of density
    // discipline excuses.
    for (mut bg, mut bc) in &mut plates {
        let (b, o) = if mech {
            // §5: the border was `GOLD_DIM`. Two gold-ruled rectangles in
            // the two bottom corners, under a gold frame, next to gold
            // power pips and a gold selected-mount marker — five claims
            // on one accent. A neutral hairline is all a panel edge owes
            // anyone, and it is the same hairline the systems column's
            // own border used before it lost its border entirely.
            (plate(), palette::INK_FAINT.with_alpha(0.30))
        } else {
            // §5: `group_plate()`, not `scrim()`. The strip's icons are
            // 3 px strokes and needed the stronger backing (§4 proved it
            // over sand); now that the strip and the numeral share ONE
            // panel there can only be one alpha, and it has to be the
            // stronger of the two or the icons wash out again.
            (group_plate(), Color::NONE)
        };
        let f = fade.0;
        bg.0 = b.with_alpha(b.alpha() * f);
        bc.0 = o.with_alpha(o.alpha() * f);
    }
}

#[allow(clippy::type_complexity)]
fn paint_vitals(
    game: Res<Game>,
    mode: Res<HudMode>,
    mut q: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<HpNumber>>,
        Query<(&HpSeg, &mut BackgroundColor)>,
        Query<(&ArmourPip, &mut BackgroundColor, &mut BorderColor)>,
        Query<(&mut Text, &mut TextColor), With<PlateCondText>>,
    )>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];
    let mech = *mode == HudMode::Mech;

    let (value, frac) = if mech {
        let max = p.mech_hull_max().max(1.0);
        (p.hull.max(0.0), (p.hull / max).clamp(0.0, 1.0))
    } else {
        (
            p.health.max(0.0),
            (p.health / MAX_HEALTH).clamp(0.0, 1.0),
        )
    };
    // one threat hue, one nominal ink. `vitals_color` is reused for the
    // infantry curve (it owns the ≤25 red / ≤20 pulse rule and the
    // threshold test); the chassis uses the same rule on its fraction.
    let col = if mech {
        if frac <= 0.25 {
            THREAT
        } else {
            palette::INK
        }
    } else {
        crate::vitals_color(p.health.max(0.0), simr.t)
    };
    if let Ok((mut t, mut c)) = q.p0().get_single_mut() {
        **t = if p.alive() {
            format!("{value:.0}")
        } else {
            String::new()
        };
        *c = TextColor(col);
    }
    let filled = frac * VITALS_SEGMENTS as f32;
    for (seg, mut bg) in &mut q.p1() {
        let i = seg.0 as f32;
        bg.0 = if filled >= i + 1.0 {
            col
        } else if filled > i {
            col.with_alpha(0.45)
        } else {
            palette::PANEL_HI
        };
    }
    // armour: the power core in a chassis, the set's torso plate on
    // foot. Same model the old HUD reasoned its way to — kept, per §1.
    let pips = if mech {
        p.armor.max(0.0) / (POWER_MAX / ARMOR_PIPS as f32)
    } else {
        crate::armor_spec(p.armor_set).flat_torso
            / (ARMOR_PIP_REFERENCE / ARMOR_PIPS as f32)
    };
    for (pip, mut bg, mut bc) in &mut q.p2() {
        let full = pips >= pip.0 as f32 + 1.0;
        let partial = !full && pips > pip.0 as f32;
        bg.0 = if full {
            SYSTEMS
        } else if partial {
            SYSTEMS.with_alpha(0.4)
        } else {
            Color::NONE
        };
        bc.0 = if full || partial {
            SYSTEMS
        } else {
            palette::GOLD_DIM
        };
    }
    // plate condition, HUMAN mode only. Every stage comes from the sim;
    // this side owns the string layout and the hue, nothing else.
    let cond = (!mech && p.alive()).then(|| {
        let stages: Vec<Option<sim::ArmorStage>> = sim::ArmorPiece::ALL
            .iter()
            .map(|piece| simr.armor_stage_of(simr.player, *piece))
            .collect();
        plate_condition(&stages)
    });
    if let Ok((mut t, mut c)) = q.p3().get_single_mut() {
        match cond.flatten() {
            Some((worst, worn, total)) => {
                **t = plate_condition_text(worst, worn, total);
                *c = TextColor(plate_condition_color(worst));
            }
            // no plates on, or in a chassis, or dead: print nothing
            // rather than a reading that means nothing.
            None => **t = String::new(),
        }
    }
}

#[allow(clippy::type_complexity)]
fn paint_ammo(
    game: Res<Game>,
    mode: Res<HudMode>,
    mut q: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<AmmoNumber>>,
        Query<&mut Text, With<AmmoReserve>>,
        Query<&mut Text, With<AmmoSub>>,
    )>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];

    // §C.7 branch order is load-bearing and is preserved: the in-mech
    // arm MUST precede every infantry-flavoured one, or a stale
    // shield/reload state paints over the belt.
    //
    // `MechWeapon` is read from the sim, never hardcoded — the weapon
    // strip once printed `TURRET 0 / ROCKETS 0` in a medic for exactly
    // that reason.
    let (big, small, sub, danger) = if !p.alive() {
        (String::new(), String::new(), String::new(), false)
    } else if *mode == HudMode::Mech {
        match p.mech_weapon {
            sim::MechWeapon::Rockets => {
                let (a, b) = ammo_pair(p.pod_ammo as u32, crate::POD_TUBES as u32);
                (a, b, "ROCKET PODS".to_string(), p.pod_ammo <= 1)
            }
            sim::MechWeapon::Plasma => (
                format!("{:.0}", heat_pct(p.gatling_heat, true)),
                "%".to_string(),
                "PLASMA BOW".to_string(),
                p.gatling_vent_t > 0.0,
            ),
            sim::MechWeapon::Repair => (
                format!("{:.0}", heat_pct(p.gatling_heat, true)),
                "%".to_string(),
                if p.repair_target >= 0 {
                    "REPAIR BEAM  LINKED".to_string()
                } else {
                    "REPAIR BEAM".to_string()
                },
                p.gatling_vent_t > 0.0,
            ),
            // XII-A: name the mount from `mount_name`, the same function
            // the systems column uses.
            //
            // The capture caught this arm labelling the GATLING turret
            // "AUTOCANNON": `Gatling` falls into `_` here, so a frame
            // read `> TURRET` in the systems column and `AUTOCANNON`
            // under the numeral, four hundred pixels apart, for the same
            // mount. Two spellings of one fact is precisely what this
            // pass exists to remove.
            other => {
                let (a, b) = ammo_pair(p.mech_rounds, crate::MECH_ROUNDS);
                (
                    a,
                    b,
                    mount_name(other).to_string(),
                    crate::ammo_is_low(p.mech_rounds, crate::MECH_ROUNDS),
                )
            }
        }
    } else {
        let (a, b) = ammo_pair(p.ammo, p.reserve);
        // The throwable half of this line MOVED to `inventory_strip`,
        // which draws the selected throwable's icon, its key and its
        // count directly above this cluster. Leaving `FRAG x1` here too
        // would print one fact in two places six pixels apart — the
        // duplication XII-A exists to remove.
        //
        // What replaces it is the CALIBRE, from `crate::ammo_kind` — the
        // sim-adjacent table that already spells every one of these and
        // had been sitting behind an `allow(dead_code)` since the flavour
        // line was dropped. It is the one fact about the number above it
        // that the strip does NOT carry.
        (
            a,
            b,
            format!("{}   {}", crate::gun(p.gun).name, crate::ammo_kind(p.gun)),
            crate::ammo_is_low(p.ammo, crate::gun(p.gun).mag),
        )
    };

    if let Ok((mut t, mut c)) = q.p0().get_single_mut() {
        **t = big;
        *c = TextColor(if danger { THREAT } else { palette::INK });
    }
    if let Ok(mut t) = q.p1().get_single_mut() {
        **t = small;
    }
    if let Ok(mut t) = q.p2().get_single_mut() {
        **t = sub;
    }
}

#[allow(clippy::type_complexity)]
fn paint_top(
    game: Res<Game>,
    mode: Res<HudMode>,
    mut q: ParamSet<(
        Query<&mut Text, With<UrgentText>>,
        Query<&mut Text, With<ClockText>>,
        Query<(&TeamScoreText, &mut Text)>,
    )>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];
    let hull_frac = if *mode == HudMode::Mech {
        Some((p.hull / p.mech_hull_max().max(1.0)).clamp(0.0, 1.0))
    } else {
        None
    };
    let line = urgent_line(
        p.alive(),
        simr.round_over_t.is_some(),
        simr.overtime,
        hull_frac,
        (p.health / MAX_HEALTH).clamp(0.0, 1.0),
        p.gatling_vent_t > 0.0 || p.vent_t > 0.0,
    );
    if let Ok(mut t) = q.p0().get_single_mut() {
        **t = line;
    }
    if let Ok(mut t) = q.p1().get_single_mut() {
        **t = crate::fmt_clock(simr.match_t);
    }
    let (blue, red) = score_line(simr.mode, simr.score[0], simr.score[1], simr.zombies.len());
    for (slot, mut t) in &mut q.p2() {
        **t = if slot.0 == 0 { blue.clone() } else { red.clone() };
    }
}

fn paint_equip(game: Res<Game>, mut q: Query<&mut Text, With<EquipText>>) {
    let p = &game.sim.fighters[game.sim.player];
    let Ok(mut t) = q.get_single_mut() else {
        return;
    };
    // reference img2's top-left: a short vertical list of equipment
    // STATE — flat rows, a label and a status word, no box.
    let set = match p.armor_set {
        ArmorSet::None => "NO ARMOUR",
        ArmorSet::Folk => "FOLK ARMOUR",
        ArmorSet::Recon => "RECON WEAVE",
        ArmorSet::RobotSuit | ArmorSet::RoyalMech | ArmorSet::ScoutMech => {
            p.armor_set.chassis_name().unwrap_or("CHASSIS")
        }
    };
    let mut s = String::new();
    s += set;
    s.push('\n');
    if p.shield_up && !p.in_mech() {
        s += "SHIELD    UP\n";
    }
    if p.brace || p.mech_brace {
        s += "BRACE     SET\n";
    }
    if p.reload_t > 0.0 {
        s += "RELOAD    ...\n";
    }
    // §owner THE JAVELIN CHARGE, and the seven-second hold's TELL.
    //
    // `spear_max_charged_of` pays +10% damage four seconds after the
    // charge fills and had ZERO readers anywhere in the client, so the
    // bonus was a mechanic no player could discover: nothing on screen
    // changed when they earned it. Three states, one row, in this
    // panel's own LABEL/STATUS grammar - and no float and no bar, per
    // this file's §0 ban on printing raw sub-second numbers.
    //
    // Both facts come from the sim's own accessors rather than from
    // `spear_charge_t / SPEAR_CHARGE_FULL_S`, which is a sim constant
    // divided on the client and the exact split those accessors were
    // published to close.
    match game.sim.spear_stance_of(game.sim.player) {
        sim::SpearStance::Winding => {
            s += if game.sim.spear_max_charged_of(game.sim.player) {
                "JAVELIN   MAX +10%\n"
            } else if game.sim.spear_wind_frac_of(game.sim.player) >= 1.0 {
                "JAVELIN   FULL\n"
            } else {
                "JAVELIN   WIND\n"
            };
        }
        sim::SpearStance::Planting => {
            // the earned bonus survives the release, and so does its
            // readout - the accessor is deliberately true through both
            s += if game.sim.spear_max_charged_of(game.sim.player) {
                "JAVELIN   THROWN  MAX\n"
            } else {
                "JAVELIN   THROWN\n"
            };
        }
        sim::SpearStance::Carried => {}
    }
    **t = s;
}

fn paint_transient(game: Res<Game>, mut q: Query<&mut Text, With<TransientText>>) {
    let simr = &game.sim;
    let Ok(mut t) = q.get_single_mut() else {
        return;
    };
    // Bottom-left, unpanelled, three lines maximum. This used to sit at
    // 40%/62% — in the middle third of the screen, which both
    // references forbid.
    let mut s = String::new();
    for (ev, _) in simr.hits.iter().rev().take(3) {
        if ev.shooter == simr.player {
            s += &format!(
                "{}  {:.0}{}\n",
                simr.fighters[ev.victim].name,
                ev.damage,
                if ev.fatal { "  DOWN" } else { "" }
            );
        }
    }
    **t = s;
}

#[allow(clippy::type_complexity)]
fn paint_systems(
    game: Res<Game>,
    mode: Res<HudMode>,
    bracket: Query<&BackgroundColor, With<MechTargetBracket>>,
    mut q: ParamSet<(
        Query<(&SystemsLabel, &mut Text, &mut TextColor)>,
        Query<(&SystemsValue, &mut Text)>,
    )>,
) {
    if *mode != HudMode::Mech {
        return;
    }
    let p = &game.sim.fighters[game.sim.player];
    // Is the acquisition bracket actually on a body?
    //
    // It is READ, not recomputed. `mech_hud_sync` owns the 10-degree
    // cone and drops the bracket's alpha to zero when nothing is
    // acquired; a second copy of that ray in this module is precisely
    // the split brain that has already shipped four times here.
    let locked = bracket
        .iter()
        .any(|c| c.0.alpha() > 0.02);
    // XII-A: the folded weapon strip. `for_set` is the sim's own list.
    //
    // The shield line appears ONLY while a shield is actually up. The
    // barrier is the HEAVY's forearm module — the sim's damage path
    // returns before it consults the pool for a `ScoutMech`, so a
    // medic's `mech_shield_hp` is a field nothing reads, and the old
    // `shield_readout` printed it as a full bar for the whole match.
    // That trap is not re-inherited here.
    let shield = if p.shield_up && p.in_heavy_mech() {
        Some(("BARRIER", format!("{:.0}", p.mech_shield_hp.max(0.0))))
    } else if p.shield_up && !p.in_mech() {
        Some(("GUARD", "UP".to_string()))
    } else {
        None
    };
    let rows = systems_lines(
        sim::MechWeapon::for_set(p.armor_set),
        p.mech_weapon,
        p.mech_rounds,
        p.pod_ammo,
        locked,
        shield,
    );
    for (slot, mut t, mut c) in &mut q.p0() {
        let label = rows.get(slot.0).map(|r| r.0.clone()).unwrap_or_default();
        // The selection marker is read off the row's OWN label, which
        // `systems_lines` wrote eleven lines up in this same file. That
        // is not the split brain the module docs warn about — nothing
        // here re-derives `p.mech_weapon` from the sim, it just asks the
        // row it was handed which one it is. Alternatives were widening
        // the tuple (five test call sites) while another lane holds
        // `main.rs`; this stays inside the function pair that owns it.
        *c = TextColor(if label.starts_with(SELECTED_MARK) {
            SYSTEMS
        } else {
            palette::INK_SOFT
        });
        **t = label;
    }
    for (slot, mut t) in &mut q.p1() {
        **t = rows.get(slot.0).map(|r| r.1.clone()).unwrap_or_default();
    }
}

/// The 4-second idle fade, for this layer's text.
///
/// `hud_fade` in `main.rs` is untouched and still drives the (now
/// hidden) legacy trio, so its `Local` snapshot tuple and its
/// `Or<(...)>` query keep finding exactly the entities they expect.
/// This is the same rule over this layer's text, with heat added — a
/// pilot holding a hot mount is not idle.
/// The seconds of no-change after which this layer dims.
const IDLE_FADE_AFTER: f32 = 4.0;
/// What it dims TO. Not zero: a HUD that vanishes is a HUD you cannot
/// plan off between fights.
const IDLE_FADE_ALPHA: f32 = 0.45;

/// The idle fade curve, pulled out of `layer_fade` so it is callable.
///
/// It is a step, not a ramp, and that is the pre-existing behaviour —
/// this function only gives it a name and a test. A 47-degree camera bug
/// once survived for months in this repo purely because its arithmetic
/// lived inside a Bevy system nothing could call.
fn idle_alpha(idle_t: f32) -> f32 {
    if idle_t > IDLE_FADE_AFTER {
        IDLE_FADE_ALPHA
    } else {
        1.0
    }
}

/// THE fade clock's current output, published for layers that cannot
/// carry `XiiFade`.
///
/// `XiiFade` works by rewriting `TextColor`, which is exactly right for
/// text and useless for `inventory_strip`, whose cells are `Node`s with
/// `BackgroundColor`/`BorderColor`. That module shipped with no fade
/// participation at all and correctly refused to start a SECOND four
/// second clock to get one — two clocks drifting apart is how a HUD ends
/// up half dimmed.
///
/// So there is still exactly one clock, in `layer_fade`, and this
/// resource is its reading. Consumers multiply, they never re-time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct XiiFadeAlpha(pub f32);

impl Default for XiiFadeAlpha {
    fn default() -> Self {
        Self(1.0)
    }
}

pub(crate) fn layer_fade(
    time: Res<Time>,
    game: Res<Game>,
    mut last: Local<[i32; 9]>,
    mut idle_t: Local<f32>,
    mut out: ResMut<XiiFadeAlpha>,
    mut q: Query<&mut TextColor, With<XiiFade>>,
) {
    let p = &game.sim.fighters[game.sim.player];
    let snap = fade_snapshot(
        p.ammo,
        p.reserve,
        p.health,
        p.throw_sel,
        p.shield_up,
        p.mech_rounds,
        p.pod_ammo,
        p.hull,
        p.gatling_heat,
    );
    if snap != *last {
        *last = snap;
        *idle_t = 0.0;
    } else {
        *idle_t += time.delta_secs();
    }
    let alpha = idle_alpha(*idle_t);
    // Published BEFORE the text pass so a consumer scheduled after this
    // system sees this frame's value, not the previous one's.
    out.0 = alpha;
    for mut tc in &mut q {
        tc.0 = tc.0.with_alpha(alpha);
    }
}

/// Pure, so the "does the mech HUD fade mid-firefight" scar is testable
/// rather than only reproducible.
#[allow(clippy::too_many_arguments)]
fn fade_snapshot(
    ammo: u32,
    reserve: u32,
    health: f32,
    throw_sel: u8,
    shield_up: bool,
    mech_rounds: u32,
    // `u8` because that is what `Fighter::pod_ammo` is. Taking the sim's
    // own type means a widening there becomes a compile error here rather
    // than a silently truncated snapshot that stops waking the fade.
    pod_ammo: u8,
    hull: f32,
    gatling_heat: f32,
) -> [i32; 9] {
    [
        ammo as i32,
        reserve as i32,
        health as i32,
        throw_sel as i32,
        shield_up as i32,
        mech_rounds as i32,
        pod_ammo as i32,
        hull as i32,
        // heat quantised to 1% so a cooling mount keeps the HUD awake
        // without repainting on float noise
        (gatling_heat * 100.0) as i32,
    ]
}

// ---- tests ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The idle fade is now a named, callable function AND a published
    /// resource, so `inventory_strip` can dim in step with the numerals
    /// instead of running a second four-second clock beside them.
    ///
    /// Fails on the pre-change code, where `idle_alpha` did not exist
    /// and the step lived inline in `layer_fade`.
    #[test]
    fn the_idle_fade_is_one_clock_that_other_layers_can_read() {
        // Lit while anything is changing, and lit right up to the edge.
        assert_eq!(idle_alpha(0.0), 1.0);
        assert_eq!(idle_alpha(IDLE_FADE_AFTER), 1.0, "must not dim early");
        // Dimmed once idle, and DIMMED, not gone.
        assert_eq!(idle_alpha(IDLE_FADE_AFTER + 0.01), IDLE_FADE_ALPHA);
        assert_eq!(idle_alpha(60.0), IDLE_FADE_ALPHA);
        assert!(
            IDLE_FADE_ALPHA > 0.0 && IDLE_FADE_ALPHA < 1.0,
            "a HUD that vanishes is a HUD you cannot plan off between fights"
        );
        // A layer that spawns before the clock has ever ticked must
        // start LIT, not invisible.
        assert_eq!(XiiFadeAlpha::default().0, 1.0);
    }

    /// Gold marks the selected mount and nothing else. The column used
    /// to print all four labels in `SYSTEMS`, which left the one row the
    /// pilot needs to find with no way to stand out.
    #[test]
    fn only_the_selected_mount_row_wears_the_systems_hue() {
        let mounts = sim::MechWeapon::for_set(sim::ArmorSet::RobotSuit);
        assert!(mounts.len() >= 2, "this test needs a multi-mount chassis");
        let rows = systems_lines(mounts, mounts[0], 240, 4, true, None);
        let gold: Vec<&String> = rows
            .iter()
            .map(|r| &r.0)
            .filter(|l| l.starts_with(SELECTED_MARK))
            .collect();
        assert_eq!(gold.len(), 1, "exactly one row may take the accent");
        assert!(gold[0].contains(mount_name(mounts[0])));
        // the LOCK row is a reading, not a selection, and must not
        // accidentally match the marker
        assert!(rows.iter().any(|r| r.0 == "LOCK"));
        assert!(!"LOCK".starts_with(SELECTED_MARK));
        // an unselected mount is padded with a space, never the marker
        assert!(rows[1].0.starts_with(' '));
    }

    /// Every widget this module owns is outside the centre third, at
    /// every aspect. Reference rule 1, which the old HUD broke with
    /// seven widgets.
    #[test]
    fn centre_is_clear() {
        for (w, h) in [(1280.0, 720.0), (1920.0, 1080.0), (2560.0, 1080.0)] {
            for (name, a, o) in HUD_ANCHORS.iter().chain(XII_ANCHORS.iter()) {
                let (x, y) = anchor_px(*a, *o, w, h);
                assert!(
                    !in_centre_third(x, y, w, h),
                    "{name} sits in the centre third at {w}x{h}: ({x},{y})"
                );
                assert!(
                    x >= w * HUD_SAFE_FRAC - 1.0 && x <= w * (1.0 - HUD_SAFE_FRAC) + 1.0,
                    "{name} breaks the safe area in x at {w}x{h}"
                );
                assert!(
                    y >= h * HUD_SAFE_FRAC - 1.0 && y <= h * (1.0 - HUD_SAFE_FRAC) + 1.0,
                    "{name} breaks the safe area in y at {w}x{h}"
                );
            }
        }
    }

    /// §5: GOLD IS AN ACCENT AGAIN.
    ///
    /// The owner looked at the mech view and asked for "more simple...
    /// professional similar to our old designs". The frame photographed
    /// in `handback/brief-vii/hud_contrast/03-level-north.png` was eight
    /// bars of `palette::GOLD` at 0.85 alpha, and the two bottom clusters
    /// were ruled in `GOLD_DIM` — so the accent was simultaneously the
    /// frame colour, the panel-edge colour, the power-pip colour and the
    /// selected-mount colour. A colour that marks four things marks none.
    ///
    /// This asserts the two that were demoted are now NEUTRAL, and that
    /// the one that survives is still gold. It fails on the pre-change
    /// code at the first assert.
    #[test]
    fn gold_is_spent_on_selection_only() {
        let chroma = |c: Color| {
            let s = c.to_srgba();
            s.red.max(s.green).max(s.blue) - s.red.min(s.green).min(s.blue)
        };
        assert!(
            chroma(frame_ink()) < 0.12,
            "the mech frame is still a saturated hue ({:.2} chroma)",
            chroma(frame_ink())
        );
        assert!(
            frame_ink().alpha() < 0.6,
            "a frame at {:.2} alpha is a fifth widget, not framing",
            frame_ink().alpha()
        );
        assert!(
            FRAME_THICK_PX <= 2.0,
            "a {FRAME_THICK_PX}px stroke in all four corners is not a hairline"
        );
        // ...and the accent still exists, on the one thing it marks.
        assert_eq!(SYSTEMS, palette::GOLD);
    }

    /// §5: the systems column stopped being a fourth floating cluster.
    ///
    /// Its anchor is gone from `XII_ANCHORS` because the rows now live
    /// inside the bottom-right group panel. This test is what stops the
    /// deletion being silent: `spawn_top`/`spawn_equip` used to index
    /// this table (`XII_ANCHORS[3].2`), so removing a row would have
    /// moved the top-left panel to the bottom-right corner without a
    /// single compiler complaint. Both call sites look up by NAME now.
    ///
    /// Fails on the pre-change code, where `xii` did not exist and
    /// "mech-systems" did.
    #[test]
    fn the_xii_anchors_are_addressed_by_name_and_the_floating_column_is_gone() {
        assert!(
            !XII_ANCHORS.iter().any(|(n, _, _)| *n == "mech-systems"),
            "the systems column still has a floating anchor of its own"
        );
        // every surviving name resolves to ITS OWN row, not to a position
        for (name, _, off) in XII_ANCHORS {
            assert_eq!(xii(name), *off, "{name} resolves to the wrong offset");
        }
        // and an unknown name is a safe-area corner, never a panic and
        // never the centre of the screen
        assert_eq!(xii("no-such-anchor"), [HUD_SAFE_FRAC, HUD_SAFE_FRAC]);
    }

    /// **Simplifying the mech HUD must not delete a mech reading.** Every
    /// row exists because of a real past bug, and `systems_lines` is
    /// deliberately untouched by §5 — only its chrome and its position
    /// changed. This pins the content so a later "simplification" cannot
    /// quietly drop one.
    #[test]
    fn the_simplified_mech_column_still_prints_every_reading() {
        let rows = systems_lines(
            &[sim::MechWeapon::Gatling, sim::MechWeapon::Rockets],
            sim::MechWeapon::Gatling,
            300,
            10,
            true,
            Some(("BARRIER", "280".to_string())),
        );
        let labels: Vec<&str> = rows.iter().map(|r| r.0.trim()).collect();
        for want in ["> TURRET", "ROCKETS", "LOCK", "BARRIER"] {
            assert!(
                labels.iter().any(|l| l == &want),
                "{want} vanished from the mech column: {labels:?}"
            );
        }
        // the unselected mount still carries its count - that is the one
        // thing the column knows that the big numeral cannot say
        assert_eq!(
            rows.iter().find(|r| r.0.trim() == "ROCKETS").map(|r| &r.1),
            Some(&"10".to_string())
        );
        assert!(
            rows.len() <= SYSTEMS_ROWS,
            "{} rows will not fit the {SYSTEMS_ROWS} spawned",
            rows.len()
        );
    }

    /// The human panel has to be the STRONGER of the two values, because
    /// after §5 it is the only rectangle behind both the 76 px numerals
    /// and the strip's 3 px icon strokes — and the icons are what needs
    /// the backing. Fails on the pre-change code, where the ammo cluster
    /// used `scrim()` directly and the strip carried its own plate.
    #[test]
    fn the_one_human_panel_is_backed_for_the_thinnest_thing_on_it() {
        assert!(
            group_plate().alpha() > scrim().alpha(),
            "the group panel is no stronger than the bare scrim"
        );
        assert!(
            group_plate().alpha() < plate().alpha(),
            "the human panel has become as heavy as the mech's - that is \
             the density split this HUD is built on"
        );
    }

    /// The plate line reports the WORST plate on the body, not the first
    /// one, not the average. A player with seven fresh plates and one
    /// severed pauldron is in trouble at the pauldron.
    #[test]
    fn plate_line_reports_the_worst_plate() {
        use sim::ArmorStage::*;
        let stages = [Some(Fresh), Some(Scuffed), Some(Severed), Some(Fresh)];
        let (worst, worn, total) = plate_condition(&stages).expect("four plates equipped");
        assert_eq!(worst, Severed);
        assert_eq!(worn, 2, "scuffed and severed are both below fresh");
        assert_eq!(total, 4);
        assert_eq!(plate_condition_text(worst, worn, total), "PLATES SEVERED 2/4");

        // ...and the worst plate being LAST in the array must give the
        // same answer as it being first. If `plate_condition` returned
        // `stages[0]` — which reads plausibly — the assert above would
        // still pass on a Severed-first array, so this is the pair that
        // gives it teeth.
        let reordered = [Some(Severed), Some(Fresh), Some(Scuffed), Some(Fresh)];
        assert_eq!(plate_condition(&reordered), plate_condition(&stages));
    }

    /// `None` is "no plate on this mount", and it must not be counted as
    /// a fresh one — that would put clean steel on a bare shoulder, the
    /// exact failure `armor_stage_of`'s own doc comment warns about.
    #[test]
    fn a_missing_plate_is_not_a_fresh_plate() {
        use sim::ArmorStage::*;
        let (worst, worn, total) =
            plate_condition(&[None, Some(Fresh), None, Some(Cracked)]).expect("two equipped");
        assert_eq!(total, 2, "the two empty mounts are not plates");
        assert_eq!(worst, Cracked);
        assert_eq!(worn, 1);
        // nothing equipped at all prints nothing at all
        assert_eq!(plate_condition(&[None; 24]), None);
        assert_eq!(plate_condition(&[]), None);
    }

    /// An all-fresh set omits the count — "PLATES FRESH 0/8" reads like a
    /// warning. And only CRACKED/SEVERED are allowed the threat hue;
    /// reference rule 6 keeps one red for one meaning.
    #[test]
    fn fresh_is_quiet_and_only_broken_plates_are_red() {
        use sim::ArmorStage::*;
        assert_eq!(plate_condition_text(Fresh, 0, 8), "PLATES FRESH");
        assert_eq!(plate_condition_color(Fresh), SYSTEMS);
        assert_eq!(plate_condition_color(Scuffed), palette::GOLD_DIM);
        assert_eq!(plate_condition_color(Cracked), THREAT);
        assert_eq!(plate_condition_color(Severed), THREAT);
        // teeth: SYSTEMS and THREAT must not be the same colour, or the
        // three asserts above are vacuous.
        assert_ne!(SYSTEMS, THREAT);
        assert_ne!(palette::GOLD_DIM, THREAT);
    }

    /// MUTATION PROOF for the test above: the predicate must actually
    /// reject something. These are the literal coordinates the old HUD
    /// used for the hit feed (40%/62%) and the range text
    /// (51.6%/52.2%) — if `in_centre_third` were vacuous, both would
    /// pass and `centre_is_clear` would be worthless.
    #[test]
    fn centre_predicate_has_teeth() {
        let (w, h) = (1920.0, 1080.0);
        for (x, y) in [(0.400, 0.620), (0.516, 0.522), (0.58, 0.58)] {
            assert!(
                in_centre_third(x * w, y * h, w, h),
                "the old central widget at ({x},{y}) was not detected"
            );
        }
        // ...and must not reject the corners.
        assert!(!in_centre_third(0.06 * w, 0.94 * h, w, h));
    }

    /// The mech frame must not deform with aspect. The old brackets
    /// mixed percent-of-width with percent-of-height, so a stroke that
    /// was 8% wide and 6% tall at 16:9 became a different shape at
    /// 21:9. Every stroke here is px, so all four numbers are equal
    /// across aspects.
    #[test]
    fn frame_is_aspect_invariant() {
        for i in 0..8 {
            let a = frame_bar(i);
            let b = frame_bar(i);
            assert_eq!(a, b);
        }
        // and each stroke is a genuine long-thin arm, not a square
        for i in 0..8 {
            let (_, _, w, h) = frame_bar(i);
            assert!(
                (w - h).abs() > 40.0,
                "stroke {i} is not an arm: {w}x{h}"
            );
            assert!(w.min(h) <= FRAME_THICK_PX + 0.01);
            assert!(w.max(h) >= FRAME_ARM_PX - 0.01);
        }
        // horizontal and vertical arms are the SAME length — which is
        // the property the percent version could not hold.
        let (_, _, hw, _) = frame_bar(0);
        let (_, _, _, vh) = frame_bar(1);
        assert_eq!(hw, vh);
    }

    /// Health and ammo are the two biggest glyphs, and the reserve is
    /// strictly smaller than the current. On the pre-change HUD both
    /// numbers were 34 px and the banner was 34 px too.
    #[test]
    fn numerals_dominate() {
        use crate::frontend::{T_ACTION, T_BODY, T_HEAD, T_MICRO, T_SUB, T_TITLE};
        for other in [T_TITLE, T_HEAD, T_ACTION, T_BODY, T_SUB, T_MICRO] {
            assert!(
                T_NUMERAL > other,
                "T_NUMERAL {T_NUMERAL} does not beat {other}"
            );
        }
        assert!(T_NUMERAL_SM < T_NUMERAL * 0.5);
        assert!(T_NUMERAL_SM < T_HEAD);
    }

    /// The big/small pair is two strings, never `"{a} / {b}"`.
    #[test]
    fn ammo_is_a_pair() {
        let (a, b) = ammo_pair(31, 124);
        assert_eq!(a, "31");
        assert_eq!(b, "124");
        assert!(!a.contains('/'), "the pair collapsed back into one string");
    }

    /// The two heat scales. A single-scale helper cannot pass this.
    #[test]
    fn heat_reads_the_same_on_both_chassis() {
        // medic: sim clamps gatling_heat at 1.0
        assert!((heat_pct(0.63, true) - 63.0).abs() < 0.01);
        assert!((heat_pct(1.0, true) - 100.0).abs() < 0.01);
        // heavy: sim clamps the SAME field at 100.0
        assert!((heat_pct(63.0, false) - 63.0).abs() < 0.01);
        assert!((heat_pct(100.0, false) - 100.0).abs() < 0.01);
        // and neither can report over full
        assert!(heat_pct(5.0, true) <= 100.0);
    }

    /// §2: no ASCII bar graphs, and no raw sub-second floats, anywhere
    /// in the systems column. This is the loudest debug tell in the
    /// build and it must not come back.
    #[test]
    fn systems_column_has_no_debug_art() {
        for set in [sim::ArmorSet::RobotSuit, sim::ArmorSet::ScoutMech] {
            let mounts = sim::MechWeapon::for_set(set);
            for sel in mounts {
                for locked in [false, true] {
                    for shield in [None, Some(("BARRIER", "280".to_string()))] {
                        for (l, v) in
                            systems_lines(mounts, *sel, 240, 4, locked, shield.clone())
                        {
                            for s in [&l, &v] {
                                assert!(!s.contains('#'), "ASCII bar in {s:?}");
                                assert!(!s.contains('['), "ASCII bar in {s:?}");
                                assert!(!s.contains(".."), "ASCII bar in {s:?}");
                                // no "1.4s"-style raw float
                                assert!(
                                    !(s.contains('.') && s.contains('s')),
                                    "raw sub-second float in {s:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// XII-A, the whole point of the pass: the systems column restates
    /// NEITHER of the two big numerals.
    ///
    /// Mutation proof: this fails on the pre-change `systems_lines`,
    /// which opened with a literal `HULL` row and a literal `HEAT` row.
    #[test]
    fn systems_column_repeats_no_numeral() {
        for set in [sim::ArmorSet::RobotSuit, sim::ArmorSet::ScoutMech] {
            let mounts = sim::MechWeapon::for_set(set);
            for locked in [false, true] {
                let rows = systems_lines(mounts, mounts[0], 240, 4, locked, None);
                for (l, v) in &rows {
                    assert_ne!(l, "HULL", "the hull numeral is repeated in the column");
                    assert_ne!(l, "HEAT", "the heat numeral is repeated in the column");
                    assert!(!v.contains('%'), "a percent reading survived in {v:?}");
                    // The SELECTED mount's resource is the bottom-right
                    // numeral. Any value on the selected row is that
                    // number printed twice, whatever its unit.
                    //
                    // This is deliberately written against the ROW rather
                    // than against a constant: a heavy on its turret
                    // shipped `> TURRET 300` above a `300` numeral, and a
                    // test that only banned the strings HULL and HEAT
                    // passed it happily. Ban the SHAPE, not the words.
                    if l.starts_with('>') {
                        assert!(
                            v.is_empty(),
                            "selected mount {l:?} prints {v:?}, which the \
                             bottom-right numeral already shows"
                        );
                    }
                }
            }
        }
    }

    /// The mount list comes from the chassis, not from a hardcoded pair.
    /// The old strip printed `TURRET 0 / ROCKETS 0` inside a medic.
    #[test]
    fn systems_column_lists_the_chassis_you_are_in() {
        let medic = sim::MechWeapon::for_set(sim::ArmorSet::ScoutMech);
        let rows = systems_lines(medic, sim::MechWeapon::Repair, 240, 4, false, None);
        let labels: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(labels.len(), 2, "{labels:?}");
        assert!(labels.iter().all(|l| !l.contains("TURRET")));
        assert!(labels.iter().all(|l| !l.contains("ROCKETS")));
        // exactly one selection marker, and it is on the selected mount
        assert_eq!(rows.iter().filter(|r| r.0.starts_with('>')).count(), 1);
        assert!(rows.iter().any(|r| r.0 == "> REPAIR"));
        // ...and neither medic mount invents a round count
        assert!(rows.iter().all(|r| r.1.is_empty()), "{rows:?}");
    }

    /// `LOCK` and the shield line are HIDDEN unless they are true —
    /// the owner's "everything else stays hidden unless needed". The
    /// pre-change column printed `LOCK -` permanently.
    #[test]
    fn optional_rows_are_hidden_until_needed() {
        let m = sim::MechWeapon::for_set(sim::ArmorSet::RobotSuit);
        let quiet = systems_lines(m, m[0], 240, 4, false, None);
        assert_eq!(quiet.len(), 2, "{quiet:?}");
        assert!(quiet.iter().all(|r| r.0 != "LOCK"));
        let busy = systems_lines(
            m,
            m[0],
            240,
            4,
            true,
            Some(("BARRIER", "280".to_string())),
        );
        assert_eq!(busy.len(), 4, "{busy:?}");
        assert!(busy.iter().any(|r| r.0 == "LOCK"));
        assert!(busy.iter().any(|r| r.0 == "BARRIER"));
        // the column can never overflow the rows that were spawned
        assert!(busy.len() <= SYSTEMS_ROWS);
    }

    /// XII-A: the contextual prompt must sit ABOVE the numeral band.
    ///
    /// Caught by capture, not by reading: the boarding prompt is a long
    /// full-width line and it struck straight through both big numerals
    /// at bottom 20%. Mutation proof: this fails at 20.0, which is the
    /// value the pre-change code shipped.
    #[test]
    fn prompt_clears_the_numerals() {
        // where the vitals/ammo clusters START, from HUD_ANCHORS
        let (_, o) = corner("vitals");
        let cluster_bottom = -(o[1]) * 100.0;
        let cluster_top = cluster_bottom + numeral_cluster_height_pct();
        assert!(
            PROMPT_BOTTOM_PCT > cluster_top,
            "the prompt at {PROMPT_BOTTOM_PCT}% crosses a numeral cluster \
             reaching {cluster_top}%"
        );
        // and it must not climb into the centre third either
        assert!(PROMPT_BOTTOM_PCT < 100.0 / 3.0 + 1.0);
    }

    /// The human HUD is no longer bare text on the world. Mutation
    /// proof: the pre-change code painted `Color::NONE` here, whose
    /// alpha is 0, and this asserts a floor above it — while the
    /// density split against the mech plate is still asserted.
    #[test]
    fn human_readings_have_a_backing_but_stay_lighter_than_mech() {
        let s = scrim().alpha();
        let m = plate().alpha();
        assert!(s > 0.15, "a {s} scrim will not carry white over sand");
        assert!(s < m * 0.6, "the human HUD is as heavy as the mech's");
    }

    /// The urgent line is ONE line and the priority order is fixed:
    /// round over > death > chassis critical > venting > overtime.
    #[test]
    fn urgent_line_is_one_line_and_ordered() {
        assert_eq!(urgent_line(true, true, true, None, 0.1, true), "ROUND OVER");
        assert_eq!(urgent_line(false, false, true, None, 0.1, true), "DOWN");
        assert_eq!(
            urgent_line(true, false, true, Some(0.10), 1.0, true),
            "HULL CRITICAL"
        );
        assert_eq!(
            urgent_line(true, false, true, Some(0.90), 1.0, true),
            "MOUNT VENTING"
        );
        assert_eq!(urgent_line(true, false, false, None, 0.20, false), "CRITICAL");
        assert_eq!(
            urgent_line(true, false, true, None, 1.0, false),
            "SUDDEN DEATH"
        );
        // nothing urgent = nothing on screen
        assert_eq!(urgent_line(true, false, false, None, 1.0, false), "");
        for s in [
            urgent_line(true, true, true, None, 0.1, true),
            urgent_line(true, false, true, Some(0.1), 1.0, false),
        ] {
            assert!(!s.contains('\n'), "the urgent line grew a second line");
        }
    }

    /// The score line carries scores and nothing else. The old one
    /// carried `(first to 30)`, a tutorial sentence, `pressure  63%`,
    /// `HORDE 12` and raw world coordinates.
    #[test]
    fn score_line_is_only_the_score() {
        for (mode, horde) in [
            (Mode::Tdm, 0),
            (Mode::Koth, 0),
            (Mode::Training, 0),
            (Mode::Extraction, 12),
        ] {
            let (a, b) = score_line(mode, 12.0, 8.0, horde);
            for s in [&a, &b] {
                assert!(!s.contains('('), "parenthetical survived in {s:?}");
                assert!(!s.contains("first to"), "rules text survived in {s:?}");
                assert!(!s.contains("pressure"), "internal scalar in {s:?}");
                assert!(!s.contains(','), "raw coordinates in {s:?}");
                assert!(s.len() <= 8, "score field is prose: {s:?}");
            }
        }
    }

    /// The mode branch exists at all — the pre-change HUD had none.
    #[test]
    fn mode_branches() {
        assert_eq!(hud_mode_of(true, true), HudMode::Mech);
        assert_eq!(hud_mode_of(true, false), HudMode::Human);
        assert_eq!(hud_mode_of(false, true), HudMode::Human);
    }

    /// Heat moving keeps the HUD awake. Without the ninth field a
    /// pilot holding a firing button — which moves nothing else on a
    /// heat mount — would fade to 45% mid-firefight, which is the exact
    /// scar `hud_fade`'s mech trio was added for.
    #[test]
    fn fade_snapshot_notices_heat() {
        let a = fade_snapshot(0, 0, 100.0, 0, false, 0, 0, 600.0, 0.10);
        let b = fade_snapshot(0, 0, 100.0, 0, false, 0, 0, 600.0, 0.40);
        assert_ne!(a, b, "a firing heat mount looks idle to the fade");
        let c = fade_snapshot(0, 0, 100.0, 0, false, 0, 0, 600.0, 0.10);
        assert_eq!(a, c, "the snapshot is not stable for an idle player");
    }

    /// The corner table is read BY NAME, so inserting a row into
    /// `HUD_ANCHORS` cannot silently move a widget.
    #[test]
    fn corners_resolve_by_name() {
        let (a, _) = corner("vitals");
        assert_eq!(a, [0.0, 1.0]);
        let (a, _) = corner("ammo");
        assert_eq!(a, [1.0, 1.0]);
        // an unknown name must not panic and must not land in the middle
        let (a, o) = corner("nope");
        let (x, y) = anchor_px(a, o, 1920.0, 1080.0);
        assert!(!in_centre_third(x, y, 1920.0, 1080.0));
    }
}
