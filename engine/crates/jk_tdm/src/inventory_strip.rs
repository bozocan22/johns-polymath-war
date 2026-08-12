//! THE INVENTORY STRIP — the bottom-right corner stops being a text blob.
//!
//! ## Why this is its own module
//!
//! Same reason `branding.rs` and `hud.rs` are: `main.rs` is 31k lines and
//! is the single most contended file in the repo — it was dirty with
//! another lane's uncommitted work while this was written. The wiring is
//! two lines nobody else's diff can collide with:
//!
//! ```ignore
//! mod inventory_strip;
//! // ...
//! .add_plugins(inventory_strip::InventoryStripPlugin)
//! ```
//!
//! ## What the corner USED TO say
//!
//! `handback/brief-vii/grenade_hold/01-fp-rifle-before-g.png` (captured
//! immediately before this module existed) shows it: a grey scrim with
//! `30` `120` and one sub-line reading `M4A1   FRAG x1`. That sub-line is
//! the entire carried inventory, spelled as prose. It cannot say what the
//! OTHER two guns are, what the other three throwables are, how much is
//! in them, or which key reaches any of it. The owner's word for it was
//! "drab"; the functional complaint underneath is that it is not
//! strategic — you cannot plan off it.
//!
//! This layer draws the same facts as a strip of CELLS, one per carried
//! item, each carrying four things: a small colourful icon, the KEY that
//! selects it, how many are left, and whether it is the selected one.
//!
//! ## The icons are `Node`s, not sprites and not glyphs
//!
//! There are no UI image assets in this project (`engine/assets/` is
//! audio), and glyphs are not an option either: `main.rs` documents
//! U+271A rendering as a tofu box in the bundled font, which is why this
//! codebase fell back to ASCII everywhere. So every icon here is flat
//! vector art assembled from 3-5 coloured, rounded `Node` rectangles.
//!
//! It is GEOMETRY-AS-DATA, matching the house style of `weapon_parts` in
//! `main.rs`: `icon_parts` is a pure function returning a `Vec<IconPart>`
//! in a 34x34 local box, and the spawn code just walks it. Nothing about
//! an icon is hand-inlined into `Commands`, so a new item is a new match
//! arm rather than a new node tree.
//!
//! The cells are spawned ONCE at `Startup` with a fixed part budget and
//! REPAINTED every frame from the live loadout. That is deliberate: the
//! player's guns are not known at `Startup` (they come from the loadout
//! screen), and a gun can still change mid-match by picking a minigun up
//! off the floor. Spawning per-loadout would have missed both.
//!
//! ## Cosmetic only
//!
//! Every system here takes `Res<Game>`, never `ResMut`. Nothing computes
//! a hit, a count or a threshold — `p.grenades`, `p.throw_sel`,
//! `p.inventory`, `p.slot_ammo`, `p.ammo`, `p.reserve` and `p.shield_up`
//! are all read straight out of the sim.

use bevy::prelude::*;

use crate::frontend::palette;
use crate::{sim, Game, GameState, GunKind, HudRoot, ThrowKind, HUD_ANCHORS};

// ---- geometry -------------------------------------------------------------

/// The icon AUTHORING box, in the 720p space the rest of the type ramp
/// uses. `UiScale` handles resolution. Every `IconPart` is written in
/// this box and scaled to whichever cell size it lands in, so the icon
/// table never has to know how big it is being drawn.
const ICON: f32 = 34.0;

/// The active item's icon, at the right end of the row. Nearly double
/// the secondaries — the reference's size contrast between its subject
/// and its supporting icons is roughly 2:1 and it is doing a lot of the
/// work.
const ICON_LG: f32 = 34.0;
/// A secondary item's icon.
const ICON_SM: f32 = 22.0;

const CELL_LG_W: f32 = 54.0;
/// Wide enough for the WIDEST count a secondary cell can print.
///
/// This was 28, and the capture caught it: `count_text` returns strings
/// like `17/68` and `READY`, `TextLayout::new_with_no_wrap()` refuses to
/// break them, and a centred 34px string in a 28px box overflows BOTH
/// sides into its neighbours. The result was a single illegible run
/// reading `17/68 5/10 READY x1` straight through the icons above it.
/// This is why the strip gets photographed rather than reasoned about.
///
/// 38 was the first fix and it was still too narrow — the guard-rail
/// test below caught what the capture could not, because the loadout in
/// the capture happened not to contain the widest case. An M249 holds a
/// 100 round belt, so `100/200` is SEVEN characters (~39px), not the
/// five that `17/68` suggested. That is the whole argument for having
/// the estimate test as well as the screenshot: the frame only proves
/// the loadout it photographed.
const CELL_SM_W: f32 = 44.0;

/// The count line. Smaller than `frontend::T_MICRO` (11) because eight of these
/// sit in a row and they are the least important text on the screen —
/// the numbers that matter are the 76px pair directly below.
const T_COUNT: f32 = 9.0;
/// Row height, sized off the large cell plus its count line.
const ROW_H: f32 = ICON_LG + 16.0;

/// The thin vertical rules between items. 1px at low alpha — the
/// reference separates its icon groups with hairlines, not with boxes,
/// and it is the single cheapest thing that makes a row of icons look
/// deliberate instead of scattered.
const RULE_W: f32 = 1.0;
const RULE_ALPHA: f32 = 0.16;
/// The rule immediately before the active cell separates the two GROUPS
/// rather than two items, so it is a touch stronger and full height.
const RULE_MAJOR_ALPHA: f32 = 0.34;
/// How far ABOVE the ammo cluster's own anchor the strip sits, in screen
/// fractions. The ammo corner is at `-0.09`; the big numeral plus its
/// sub-line is roughly 0.13 of a 720p screen tall, so 0.155 clears it
/// with room and still lands well below `XII_ANCHORS`' mech-systems
/// column at 0.24 — which this strip never shares a frame with anyway,
/// because it hides in a mech.
const LIFT: f32 = 0.155;

/// What a rectangle of icon art is FOR, rather than what colour it is.
///
/// ## Why the colour left this table
///
/// The first cut of this module hardcoded a colour per part: teal and
/// violet gun tints, a green frag, an azure shield, an amber molotov.
/// The brief that asked for it said "colourful and cute", and it was
/// wrong — those hues are off-palette, they are the reason the corner
/// did not look professional, and they left the strip with no way to say
/// the one thing it most needs to say, which is WHICH ITEM IS SELECTED.
/// Eight cells in eight saturated hues are eight equal shouts.
///
/// So parts now name a ROLE and `resolve` maps role x cell-state to a
/// concrete colour from `frontend::palette`. The geometry — which was
/// fine, and is the expensive part — is untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tone {
    /// The silhouette proper: receiver, body, casing. The bright tier.
    Body,
    /// Subordinate mass: grips, stocks, magazines, bands. One step down,
    /// so the shape reads as a shape and not as a solid blob.
    Detail,
    /// The ONE highlight per icon — sight, pin, tip, flame. This is the
    /// only thing that is ever allowed to be gold, and only on the
    /// selected cell. `branding.rs`'s palette doc calls GOLD "the
    /// primary accent and the colour of anything selected"; using it on
    /// every icon at once would make it the colour of nothing.
    Accent,
}

/// One rectangle of flat icon art, in the icon's own 34x34 box.
///
/// `x`/`y` are from the box's top-left. Everything is axis-aligned on
/// purpose: a UI `Node` cannot be rotated without a `Transform` fight,
/// and a diagonal that renders wrong reads as a bug while a blunt
/// silhouette merely reads as stylised.
#[derive(Clone, Copy, Debug)]
struct IconPart {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    /// Corner radius. Kept small now — the reference's weapon glyph is
    /// line art with crisp ends, and 9px radii on a 22px capsule were a
    /// big part of what read as toy-like.
    r: f32,
    t: Tone,
}

const fn part(x: f32, y: f32, w: f32, h: f32, r: f32, t: Tone) -> IconPart {
    IconPart { x, y, w, h, r, t }
}

/// The most parts any icon uses. Cells pre-spawn this many and hide the
/// tail, so a repaint never has to touch `Commands`.
const MAX_PARTS: usize = 5;

/// How a cell is currently reading. Drives every colour in the cell
/// through `resolve`, so there is exactly one place that decides what
/// "selected" or "empty" looks like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellTier {
    /// The item in your hands. Bright ink plus the gold accent.
    Selected,
    /// Carried and usable, but not selected. The resting state of most
    /// of the strip, and it must stay quiet.
    Ready,
    /// Out of stock. Still legible — you need to know you HAVE a frag
    /// slot and it is empty — but clearly demoted.
    Empty,
}

/// role x tier -> the actual colour. The whole colour system of this
/// module is these nine lines.
///
/// Three monochrome tiers off `frontend::palette` (INK / INK_SOFT /
/// INK_FAINT) plus GOLD in exactly one cell of the table. That is the
/// reference image's discipline: near-white line art, one accent used
/// sparingly, everything else carried by VALUE rather than hue.
fn resolve(t: Tone, tier: CellTier) -> Color {
    match (tier, t) {
        (CellTier::Selected, Tone::Body) => palette::INK,
        (CellTier::Selected, Tone::Detail) => palette::INK_SOFT,
        (CellTier::Selected, Tone::Accent) => palette::GOLD,

        (CellTier::Ready, Tone::Body) => palette::INK_SOFT,
        (CellTier::Ready, Tone::Detail) => palette::INK_FAINT,
        // NOT gold. An unselected cell's highlight is still a highlight
        // within its own silhouette, so it stays one step brighter than
        // the body — but it does not get to spend the accent.
        (CellTier::Ready, Tone::Accent) => palette::INK,

        (CellTier::Empty, Tone::Body) => palette::INK_FAINT.with_alpha(0.45),
        (CellTier::Empty, Tone::Detail) => palette::INK_FAINT.with_alpha(0.30),
        (CellTier::Empty, Tone::Accent) => palette::INK_FAINT.with_alpha(0.45),
    }
}

/// What a cell holds. `Gun` carries the slot index because two slots can
/// legitimately hold the same `GunKind` and the ammo lookup is per-slot.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Item {
    Gun(usize, GunKind),
    Shield,
    Throw(ThrowKind),
}

/// THE icon table. One arm per silhouette family, not one per weapon:
/// four rifles that differ only in stats should not differ in outline, or
/// the icon stops being recognisable as "your rifle".
fn icon_parts(item: Item) -> Vec<IconPart> {
    use Tone::{Accent, Body, Detail};
    match item {
        Item::Throw(ThrowKind::Frag) => vec![
            part(8.0, 11.0, 18.0, 20.0, 7.0, Body),
            part(8.0, 17.0, 18.0, 3.0, 1.0, Detail),
            part(14.0, 4.0, 5.0, 8.0, 1.5, Detail),
            part(19.0, 4.0, 8.0, 3.0, 1.5, Accent),
        ],
        Item::Throw(ThrowKind::Flash) => vec![
            part(10.0, 11.0, 15.0, 20.0, 5.0, Body),
            part(14.0, 17.0, 7.0, 8.0, 3.5, Accent),
            part(13.0, 4.0, 9.0, 7.0, 2.0, Detail),
        ],
        Item::Throw(ThrowKind::Smoke) => vec![
            part(10.0, 9.0, 15.0, 22.0, 3.0, Body),
            part(10.0, 16.0, 15.0, 3.0, 1.0, Detail),
            part(10.0, 21.0, 15.0, 3.0, 1.0, Detail),
            part(14.0, 3.0, 7.0, 6.0, 1.5, Accent),
        ],
        Item::Throw(ThrowKind::Molotov) => vec![
            part(10.0, 14.0, 15.0, 17.0, 4.0, Body),
            part(14.0, 8.0, 7.0, 7.0, 1.5, Detail),
            part(13.0, 1.0, 9.0, 8.0, 4.0, Accent),
        ],
        // A kite shield: broad shoulders, tapered foot. Built from three
        // stacked bands rather than one rounded box, because the old
        // single 8px-radius rectangle read as a phone, not a shield.
        Item::Shield => vec![
            part(6.0, 4.0, 22.0, 4.0, 1.5, Accent),
            part(6.0, 8.0, 22.0, 12.0, 1.5, Body),
            part(9.0, 20.0, 16.0, 6.0, 2.0, Body),
            part(13.0, 26.0, 8.0, 5.0, 3.0, Body),
        ],
        Item::Gun(_, k) => gun_parts(k),
    }
}

fn gun_parts(k: GunKind) -> Vec<IconPart> {
    use Tone::{Accent, Body, Detail};
    match k {
        GunKind::Fists => vec![
            part(7.0, 12.0, 18.0, 13.0, 3.0, Body),
            part(10.0, 15.0, 12.0, 3.0, 1.0, Detail),
            part(23.0, 16.0, 6.0, 7.0, 2.5, Body),
        ],
        GunKind::Glock | GunKind::Deagle => vec![
            part(6.0, 11.0, 22.0, 6.0, 1.5, Body),
            part(9.0, 17.0, 7.0, 11.0, 1.5, Detail),
            part(26.0, 12.0, 5.0, 4.0, 1.0, Accent),
        ],
        GunKind::Shotgun => vec![
            part(2.0, 12.0, 29.0, 4.0, 1.0, Body),
            part(4.0, 16.0, 18.0, 3.0, 1.0, Detail),
            part(1.0, 16.0, 8.0, 7.0, 1.5, Detail),
            part(24.0, 9.0, 4.0, 3.0, 1.0, Accent),
        ],
        GunKind::Awm => vec![
            part(2.0, 15.0, 30.0, 4.0, 1.0, Body),
            part(11.0, 7.0, 14.0, 5.0, 1.5, Accent),
            part(2.0, 19.0, 10.0, 7.0, 1.5, Detail),
            part(16.0, 19.0, 5.0, 8.0, 1.5, Detail),
        ],
        GunKind::Bow => vec![
            part(13.0, 1.0, 6.0, 13.0, 3.0, Body),
            part(13.0, 20.0, 6.0, 13.0, 3.0, Body),
            part(12.0, 14.0, 8.0, 6.0, 2.0, Detail),
            part(24.0, 3.0, 2.0, 28.0, 1.0, Accent),
        ],
        GunKind::Spear => vec![
            part(7.0, 16.0, 26.0, 3.0, 1.0, Body),
            part(0.0, 13.0, 9.0, 8.0, 2.0, Accent),
            part(14.0, 15.0, 4.0, 6.0, 1.0, Detail),
        ],
        // Every belt/box-fed longarm shares the rifle outline. This is
        // the strip's SCAR glyph — the reference image's centrepiece —
        // so it gets the most parts: receiver, barrel, magazine, stock,
        // optic. Five is `MAX_PARTS`, which is why nothing else here
        // grew past four.
        GunKind::Mp5 | GunKind::Ak47 | GunKind::M4 | GunKind::M249 | GunKind::Minigun => vec![
            part(4.0, 13.0, 21.0, 6.0, 1.5, Body),
            part(25.0, 15.0, 9.0, 3.0, 1.0, Body),
            part(12.0, 19.0, 6.0, 10.0, 1.5, Detail),
            part(0.0, 14.0, 5.0, 7.0, 1.5, Detail),
            part(9.0, 8.0, 9.0, 4.0, 1.0, Accent),
        ],
    }
}

// ---- pure formatters ------------------------------------------------------

/// The carried inventory in READING order, before selection moves
/// anything: the three gun slots in keybind order, the shield, then the
/// four throwables.
fn canonical_items(inv: &[GunKind; 3]) -> Vec<Item> {
    (0..3)
        .map(|i| Item::Gun(i, inv[i]))
        .chain(std::iter::once(Item::Shield))
        .chain(ThrowKind::ALL.iter().copied().map(Item::Throw))
        .collect()
}

/// THE layout rule, and the whole point of this pass.
///
/// ## What it replaced
///
/// A 2x4 grid of eight identically-sized boxes. Every cell had the same
/// weight, so the strip had no subject — and an inventory display whose
/// subject is not "the weapon in your hands" is decoration. That equal
/// weighting is most of why the corner read as toy-like.
///
/// ## The rule
///
/// One row. The SELECTED item goes LAST, at the right end, drawn large;
/// everything else keeps canonical order to its left, drawn small. The
/// reference image puts its big weapon glyph on the LEFT with the small
/// secondaries to the right — this is that layout mirrored, because our
/// cluster is anchored to the bottom-RIGHT and the reference's is
/// anchored bottom-left. Mirroring keeps the two things the reference
/// keeps together: the active weapon's silhouette sits directly above
/// its own big ammo numeral, which `hud.rs` draws immediately below.
///
/// It is a stable partition, so switching from slot 1 to slot 2 does not
/// reshuffle the other six icons — only the two that swapped roles move.
fn strip_order(inv: &[GunKind; 3], is_selected: impl Fn(Item) -> bool) -> Vec<Item> {
    let all = canonical_items(inv);
    let mut out: Vec<Item> = all.iter().copied().filter(|i| !is_selected(*i)).collect();
    out.extend(all.iter().copied().filter(|i| is_selected(*i)));
    out
}

/// The index of the large "active" cell: the last one.
const ACTIVE_SLOT: usize = CELLS - 1;

/// The strip's plate, a strengthened `hud.rs::scrim()`.
///
/// Expressed as a multiple of the shared value rather than as a fresh
/// literal: retuning the human HUD's scrim still moves this with it,
/// which is the whole reason the strip borrows it at all.
fn strip_plate() -> Color {
    let s = crate::hud::scrim();
    s.with_alpha((s.alpha() * 1.85).min(1.0))
}

/// The key that selects a cell.
///
/// `main.rs` binds Digit1/2/3 to the three gun slots, Digit4 to the
/// shield (it is inventory slot 4 now, not a verb key — see the comment
/// on that binding) and G to the throwable. All four throwables answer to
/// the same G because G EQUIPS then CYCLES, so the badge is only drawn on
/// the selected one; the other three would each be claiming a key that
/// does not reach them directly.
fn cell_key(item: Item) -> Option<&'static str> {
    match item {
        Item::Gun(0, _) => Some("1"),
        Item::Gun(1, _) => Some("2"),
        Item::Gun(2, _) => Some("3"),
        Item::Gun(_, _) => None,
        Item::Shield => Some("4"),
        Item::Throw(_) => Some("G"),
    }
}

/// How many are left, as the cell prints it.
///
/// A gun shows `mag/reserve` because the strip's job is the things you
/// are NOT currently holding — the one you are holding already has a
/// 76 px numeral under the strip. `--` is for a weapon with no
/// magazine at all (fists, and anything else the sim gives `mag = 0`),
/// which is not the same reading as `0` and must not look like it.
///
/// ## `primary` prints nothing, on purpose
///
/// THE ACTIVE CELL IS DIRECTLY ABOVE `hud.rs`'s `AmmoNumber`. Printing
/// `30/120` there in 11px, four pixels above the same fact rendered in
/// 76px and 26px, is the identical defect `systems_lines` in `hud.rs`
/// carries a fourteen-line scar about: a mount printing `> TURRET 300`
/// immediately above a `300` numeral. That was caught by looking at a
/// frame rather than at a diff, and it is not being re-inherited here.
///
/// This is also the answer to "should the strip host the numeral?" — no.
/// `hud.rs`'s ammo cluster already IS the reference's big/small pair
/// plus the small-caps weapon name, it is `XiiFade`-aware, and it serves
/// MECH mode too, where this strip is hidden entirely. Moving the
/// numeral in here would have deleted the mech readout to fix a human
/// one. The strip defers to it and stays silent instead.
fn count_text(
    item: Item,
    mag: u32,
    reserve: u32,
    grenades: u8,
    shield_up: bool,
    primary: bool,
) -> String {
    if primary {
        return String::new();
    }
    match item {
        Item::Gun(_, k) => {
            if sim::gun(k).mag == 0 {
                "--".to_string()
            } else {
                format!("{mag}/{reserve}")
            }
        }
        Item::Shield => if shield_up { "UP" } else { "READY" }.to_string(),
        Item::Throw(_) => format!("x{grenades}"),
    }
}

/// Is the cell out of stock? Drives the dim veil.
///
/// A gun is empty only when the magazine AND the reserve are gone — an
/// empty mag with 120 in reserve is a RELOAD, not an empty slot, and
/// greying it out would say the wrong thing at the worst moment. A
/// magazineless weapon (fists) is never empty.
fn cell_empty(item: Item, mag: u32, reserve: u32, grenades: u8) -> bool {
    match item {
        Item::Gun(_, k) => sim::gun(k).mag > 0 && mag == 0 && reserve == 0,
        Item::Shield => false,
        Item::Throw(_) => grenades == 0,
    }
}

/// Bottom edge of the strip as a screen fraction, from the SAME anchor
/// table the ammo cluster reads. Pure, so the layout test can assert the
/// clearance without a `World`.
fn strip_bottom_frac() -> f32 {
    let ammo = HUD_ANCHORS
        .iter()
        .find(|(n, _, _)| *n == "ammo")
        .map(|(_, _, o)| -o[1])
        .unwrap_or(0.06);
    ammo + LIFT
}

fn strip_right_frac() -> f32 {
    HUD_ANCHORS
        .iter()
        .find(|(n, _, _)| *n == "ammo")
        .map(|(_, _, o)| -o[0])
        .unwrap_or(0.06)
}

// ---- components -----------------------------------------------------------

#[derive(Component)]
struct StripRoot;
/// One inventory cell. `index` is a flat index into the row-major cell
/// list, so every painter is one `Vec` lookup rather than a nested match.
#[derive(Component)]
struct InvCell(usize);
#[derive(Component)]
struct CellIconPart(usize, usize);
#[derive(Component)]
struct CellCount(usize);
/// The dim veil laid over a cell whose stock is 0. Dimming one node
/// dims the icon and the number together, which is cheaper and more
/// consistent than retinting five parts.
#[derive(Component)]
struct CellVeil(usize);
/// The selected-cell accent bar along the bottom of the cell.
#[derive(Component)]
struct CellAccent(usize);
#[derive(Component)]
struct CellKeyBadge(usize);
#[derive(Component)]
struct CellKeyText(usize);
/// A hairline rule. `usize` is the slot it sits immediately BEFORE, so
/// rule `ACTIVE_SLOT` is the major group divider.
#[derive(Component)]
struct CellRule(usize);

/// Flat cell count: 3 guns + shield + 4 throwables.
const CELLS: usize = 8;

/// The icon size drawn in a given slot.
fn icon_px(slot: usize) -> f32 {
    if slot == ACTIVE_SLOT {
        ICON_LG
    } else {
        ICON_SM
    }
}

/// The cell width for a given slot.
fn cell_px(slot: usize) -> f32 {
    if slot == ACTIVE_SLOT {
        CELL_LG_W
    } else {
        CELL_SM_W
    }
}

// ---- plugin ---------------------------------------------------------------

pub struct InventoryStripPlugin;

impl Plugin for InventoryStripPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_strip).add_systems(
            Update,
            // AFTER `layer_fade`, which is the single owner of the idle
            // clock. Reading `XiiFadeAlpha` in the same frame it is
            // written keeps the strip and the numerals dimming on the
            // same tick; scheduled before it, the strip would trail by
            // one frame and the two would visibly disagree on the exact
            // frame the fade flips.
            paint_strip
                .after(crate::hud::layer_fade)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn spawn_strip(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(strip_right_frac() * 100.0),
                bottom: Val::Percent(strip_bottom_frac() * 100.0),
                // ONE row now, not a 2x4 grid. See `strip_order`.
                flex_direction: FlexDirection::Row,
                // Bottom-aligned: the small cells hang off the same
                // baseline as the large one, so the row has a floor even
                // though its cells are two different heights.
                align_items: AlignItems::FlexEnd,
                height: Val::Px(ROW_H),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            // The reference's dark translucent rounded panel.
            //
            // DERIVED from `hud.rs::scrim()` rather than equal to it, so
            // the two still move together if that value is retuned. The
            // strip needs more backing than the numerals do and the
            // capture is why: `scrim()` is tuned for 76px white glyphs,
            // which carry themselves over pale sand. These icons are
            // 2-3px line-art strokes at INK_SOFT, and at 0.30 alpha over
            // a bright desert floor they washed out to faint smudges.
            BackgroundColor(strip_plate()),
            BorderRadius::all(Val::Px(6.0)),
            // Starts hidden for the same reason the XII root does: the
            // initial state is Title and `hud_visibility` only fires on
            // Playing's enter/exit, so a root that spawned visible would
            // sit on top of the title screen until the first match.
            Visibility::Hidden,
            HudRoot,
            StripRoot,
        ))
        .with_children(|r| {
            for slot in 0..CELLS {
                if slot > 0 {
                    spawn_rule(r, slot);
                }
                spawn_cell(r, slot);
            }
        });
}

/// A hairline between two cells. Height and alpha are fixed at spawn —
/// which slot is which never changes, because `strip_order` moves ITEMS
/// between slots and never moves the slots themselves.
fn spawn_rule(b: &mut ChildBuilder, slot: usize) {
    let major = slot == ACTIVE_SLOT;
    b.spawn((
        Node {
            width: Val::Px(RULE_W),
            height: Val::Px(if major { ICON_LG } else { ICON_SM }),
            margin: UiRect::horizontal(Val::Px(if major { 6.0 } else { 3.0 })),
            ..default()
        },
        BackgroundColor(palette::INK_SOFT.with_alpha(if major {
            RULE_MAJOR_ALPHA
        } else {
            RULE_ALPHA
        })),
        CellRule(slot),
    ));
}

fn spawn_cell(b: &mut ChildBuilder, index: usize) {
    let icon = icon_px(index);
    b.spawn((
        Node {
            width: Val::Px(cell_px(index)),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        // EIGHT BORDERED, FILLED BOXES ARE GONE. They were the other
        // half of the toy-like reading: a panel inside a panel, eight
        // times, each with its own 2px border and 7px radius. The
        // reference has no cell chrome at all — its icons sit directly
        // on the one dark plate, separated by hairlines. The strip's
        // own root now carries the only plate, `spawn_rule` carries the
        // only separators, and a cell is just a layout box.
        BackgroundColor(Color::NONE),
        InvCell(index),
    ))
    .with_children(|c| {
        // the icon box — parts are absolute inside it
        c.spawn(Node {
            width: Val::Px(icon),
            height: Val::Px(icon),
            position_type: PositionType::Relative,
            ..default()
        })
        .with_children(|ib| {
            for p in 0..MAX_PARTS {
                ib.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderRadius::all(Val::Px(1.0)),
                    Visibility::Hidden,
                    CellIconPart(index, p),
                ));
            }
        });
        c.spawn((
            Text::new(""),
            TextFont {
                font_size: T_COUNT,
                ..default()
            },
            TextColor(palette::INK_SOFT),
            TextLayout::new_with_no_wrap(),
            CellCount(index),
        ));
        // The selected accent: a 2px gold underline under the ACTIVE
        // cell. This is the only gold rectangle in the strip, and with
        // the cell borders gone it is now the sole selection signal
        // besides size — which is what "one accent, used sparingly"
        // actually costs you.
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(2.0),
                right: Val::Px(2.0),
                bottom: Val::Px(0.0),
                height: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            CellAccent(index),
        ));
        // The keybind, top-right. The filled `PANEL_HI` pill it used to
        // sit in is gone for the same reason the cell borders are: eight
        // little discs is chrome, and the reference has none. It is bare
        // type now, INK_FAINT normally and GOLD on the active cell.
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            CellKeyBadge(index),
        ))
        .with_children(|k| {
            k.spawn((
                Text::new(""),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(palette::INK_FAINT),
                TextLayout::new_with_no_wrap(),
                CellKeyText(index),
            ));
        });
        // The dim veil, last so it covers icon and number.
        //
        // Retained but much weaker than the old 0.62 blackout: the icon
        // parts themselves now drop to the `Empty` tier via `resolve`,
        // so the veil is a second, gentler pass rather than the only
        // signal. Two full-strength dimmers stacked made an empty slot
        // nearly invisible, and you need to SEE that a slot is empty.
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            CellVeil(index),
        ));
    });
}

/// Everything the painter needs about one cell, resolved once.
struct CellState {
    item: Item,
    /// The item actually in your hands. Exactly ONE cell per frame has
    /// this, and it is the one that lands in `ACTIVE_SLOT`.
    primary: bool,
    /// The throwable G would throw next. NOT the same thing as
    /// `primary`, and conflating them was a real bug waiting to happen:
    /// a throwable is always "selected" in the sim's sense — `throw_sel`
    /// is never null — so if that counted as primary, TWO cells would
    /// claim the big slot and the partition in `strip_order` would put
    /// the frag grenade where the rifle belongs. It gets the quieter
    /// mark instead.
    queued: bool,
    empty: bool,
    count: String,
    key: Option<&'static str>,
}

impl CellState {
    fn tier(&self) -> CellTier {
        if self.primary {
            CellTier::Selected
        } else if self.empty {
            CellTier::Empty
        } else {
            CellTier::Ready
        }
    }
}

/// Which single item is in the player's hands.
///
/// Read straight off the sim's `shield_up` / `active`; nothing here
/// re-derives what is equipped.
fn is_primary(item: Item, active: usize, shield_up: bool) -> bool {
    match item {
        Item::Gun(i, _) => i == active && !shield_up,
        Item::Shield => shield_up,
        Item::Throw(_) => false,
    }
}

fn cell_states(p: &sim::Fighter) -> Vec<CellState> {
    let order = strip_order(&p.inventory, |i| is_primary(i, p.active, p.shield_up));
    let mut out = Vec::with_capacity(CELLS);
    for item in order {
        let (mag, reserve) = match item {
            // the sim mirrors the ACTIVE slot into `ammo`/`reserve` and
            // only writes `slot_ammo` on a swap, so the live pair has to
            // come from the mirror for the slot you are holding.
            Item::Gun(i, _) if i == p.active => (p.ammo, p.reserve),
            Item::Gun(i, _) => p.slot_ammo[i],
            _ => (0, 0),
        };
        let nades = match item {
            Item::Throw(k) => p.grenades[ThrowKind::ALL.iter().position(|a| *a == k).unwrap_or(0)],
            _ => 0,
        };
        let primary = is_primary(item, p.active, p.shield_up);
        let queued = matches!(item, Item::Throw(k) if ThrowKind::ALL[p.throw_sel as usize] == k);
        out.push(CellState {
            item,
            primary,
            queued,
            empty: cell_empty(item, mag, reserve, nades),
            count: count_text(item, mag, reserve, nades, p.shield_up, primary),
            key: cell_key(item),
        });
    }
    out
}

#[allow(clippy::type_complexity)]
fn paint_strip(
    game: Res<Game>,
    // B0001: this `&mut Visibility` is OUTSIDE the ParamSet, so Bevy has
    // to be shown it cannot overlap the two members that also take
    // `&mut Visibility` (the icon parts and the key badges). A ParamSet
    // only proves its own members disjoint from each other. Without
    // these filters the app panics on the first frame - it did, in
    // every capture script, which is how this was found.
    // (Filter added by the hands/graphics lane to unblock captures; the
    // system's behaviour is unchanged, since no StripRoot entity has
    // ever carried CellIconPart or CellKeyBadge.)
    mut root: Query<
        &mut Visibility,
        (With<StripRoot>, Without<CellIconPart>, Without<CellKeyBadge>),
    >,
    // The ONE fade clock, owned by `hud.rs::layer_fade`. `Option` so
    // this module still runs in any harness that builds the strip
    // without `HudPlugin`; the fallback is "fully lit", never a second
    // timer.
    fade: Option<Res<crate::hud::XiiFadeAlpha>>,
    // EIGHT members is Bevy's ceiling for a `ParamSet`, and this is
    // exactly eight. The ninth was a `Query<(&InvCell, &mut
    // BackgroundColor)>` that painted every cell `Color::NONE` — dead
    // weight the moment the cell chrome was deleted. `InvCell` survives
    // as a layout marker only.
    mut q: ParamSet<(
        Query<(
            &CellIconPart,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderRadius,
            &mut Visibility,
        )>,
        Query<(&CellCount, &mut Text, &mut TextColor)>,
        Query<(&CellVeil, &mut BackgroundColor)>,
        Query<(&CellAccent, &mut BackgroundColor)>,
        Query<(&CellKeyBadge, &mut BackgroundColor, &mut Visibility)>,
        Query<(&CellKeyText, &mut Text, &mut TextColor)>,
        Query<(&CellRule, &mut BackgroundColor)>,
        Query<&mut BackgroundColor, With<StripRoot>>,
    )>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];
    // THE FADE DESYNC FIX. The strip used to sit at full opacity while
    // every numeral beside it dropped to 0.45, which read as the strip
    // being a different, louder piece of software. One multiplier,
    // applied to every colour this function writes.
    let fade = fade.map(|f| f.0).unwrap_or(1.0);
    let dim = |c: Color| c.with_alpha(c.alpha() * fade);

    // A chassis has no pockets. The strip is INFANTRY inventory, and in a
    // mech the same corner belongs to the systems column — showing both
    // would put two readings in one place, the exact fault BRIEF XII-A
    // was pulled together to remove.
    let show = p.alive() && !p.in_mech();
    if let Ok(mut v) = root.get_single_mut() {
        *v = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if !show {
        return;
    }

    let states = cell_states(p);

    for (part_id, mut node, mut bg, mut radius, mut vis) in q.p0().iter_mut() {
        let Some(s) = states.get(part_id.0) else {
            continue;
        };
        let parts = icon_parts(s.item);
        // The icon table is authored in a 34x34 box; a secondary cell
        // draws it at 20. Scaling here rather than in the table is what
        // lets one set of coordinates serve both sizes.
        let k = icon_px(part_id.0) / ICON;
        match parts.get(part_id.1) {
            Some(ip) => {
                node.left = Val::Px(ip.x * k);
                node.top = Val::Px(ip.y * k);
                node.width = Val::Px(ip.w * k);
                node.height = Val::Px(ip.h * k);
                *bg = BackgroundColor(dim(resolve(ip.t, s.tier())));
                *radius = BorderRadius::all(Val::Px(ip.r * k));
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }

    for (cc, mut t, mut c) in q.p1().iter_mut() {
        let Some(s) = states.get(cc.0) else { continue };
        **t = s.count.clone();
        *c = TextColor(dim(match s.tier() {
            CellTier::Selected => palette::INK,
            CellTier::Ready => palette::INK_SOFT,
            CellTier::Empty => palette::INK_FAINT,
        }));
    }

    for (veil, mut bg) in q.p2().iter_mut() {
        let Some(s) = states.get(veil.0) else { continue };
        *bg = BackgroundColor(if s.empty {
            dim(palette::GROUND.with_alpha(0.30))
        } else {
            Color::NONE
        });
    }

    for (acc, mut bg) in q.p3().iter_mut() {
        let Some(s) = states.get(acc.0) else { continue };
        // Gold and full width under the item in your hands; a shorter,
        // dimmer gold tick under the throwable G would throw next. Two
        // strengths of one accent, rather than two accent colours.
        *bg = BackgroundColor(if s.primary {
            dim(palette::GOLD)
        } else if s.queued {
            dim(palette::GOLD_DIM.with_alpha(0.75))
        } else {
            Color::NONE
        });
    }

    for (badge, mut bg, mut vis) in q.p4().iter_mut() {
        let Some(s) = states.get(badge.0) else {
            continue;
        };
        // A throwable cell that is not the queued one has no badge:
        // see `cell_key`.
        let shown = s.key.is_some() && (!matches!(s.item, Item::Throw(_)) || s.queued);
        *vis = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        // The pill is gone; the badge node is a bare positioning box.
        *bg = BackgroundColor(Color::NONE);
    }

    for (kt, mut t, mut c) in q.p5().iter_mut() {
        let Some(s) = states.get(kt.0) else { continue };
        **t = s.key.unwrap_or("").to_string();
        *c = TextColor(dim(if s.primary {
            palette::GOLD
        } else {
            palette::INK_FAINT
        }));
    }

    for (rule, mut bg) in q.p6().iter_mut() {
        let major = rule.0 == ACTIVE_SLOT;
        *bg = BackgroundColor(dim(palette::INK_SOFT.with_alpha(if major {
            RULE_MAJOR_ALPHA
        } else {
            RULE_ALPHA
        })));
    }

    // The plate fades with everything else. Without this the strip's
    // scrim stayed at full strength around dimmed icons, which is the
    // desync in miniature.
    for mut bg in q.p7().iter_mut() {
        *bg = BackgroundColor(dim(strip_plate()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m4_kit() -> [GunKind; 3] {
        [GunKind::M4, GunKind::Glock, GunKind::Awm]
    }

    /// Selection predicate for the tests: gun slot `active`, no shield.
    fn holding(active: usize) -> impl Fn(Item) -> bool {
        move |i| is_primary(i, active, false)
    }

    #[test]
    fn the_strip_has_one_cell_per_carried_item() {
        let all = canonical_items(&m4_kit());
        assert_eq!(all.len(), CELLS);
        assert_eq!(
            all.iter().filter(|i| matches!(i, Item::Throw(_))).count(),
            ThrowKind::ALL.len()
        );
        assert_eq!(all.iter().filter(|i| matches!(i, Item::Gun(..))).count(), 3);
        assert_eq!(all.iter().filter(|i| **i == Item::Shield).count(), 1);
    }

    #[test]
    fn the_item_in_your_hands_takes_the_large_slot_next_to_the_numeral() {
        // THE layout rule. On the pre-change `strip_rows` the order was
        // fixed at [throwables..., guns..., shield] regardless of what
        // was equipped, so this fails there for every `active` — the
        // last cell was always the shield.
        for active in 0..3 {
            let order = strip_order(&m4_kit(), holding(active));
            assert_eq!(order.len(), CELLS, "the partition dropped an item");
            assert!(
                matches!(order[ACTIVE_SLOT], Item::Gun(i, _) if i == active),
                "slot {ACTIVE_SLOT} should hold gun {active}, got {:?}",
                order[ACTIVE_SLOT]
            );
        }
        // and the shield, when it is what is raised
        let order = strip_order(&m4_kit(), |i| is_primary(i, 0, true));
        assert_eq!(order[ACTIVE_SLOT], Item::Shield);
    }

    #[test]
    fn switching_weapons_does_not_reshuffle_the_other_icons() {
        // The stable-partition claim. Only the two items that swapped
        // roles may move; a strip that re-sorts itself on every weapon
        // switch is unreadable under fire.
        let a = strip_order(&m4_kit(), holding(0));
        let b = strip_order(&m4_kit(), holding(1));
        let moved = a
            .iter()
            .zip(b.iter())
            .filter(|(x, y)| x != y)
            .count();
        assert!(
            moved <= 2,
            "{moved} icons moved on a single weapon switch; expected at most 2"
        );
    }

    #[test]
    fn exactly_one_cell_is_ever_primary() {
        // The trap `queued` exists to avoid: `throw_sel` always names a
        // throwable, so a predicate that counted the selected throwable
        // as primary would put TWO items in the large slot and silently
        // drop one from the strip.
        for shield_up in [false, true] {
            for active in 0..3 {
                let n = canonical_items(&m4_kit())
                    .into_iter()
                    .filter(|i| is_primary(*i, active, shield_up))
                    .count();
                assert_eq!(n, 1, "active={active} shield_up={shield_up}");
            }
        }
        // a throwable is NEVER primary, however `throw_sel` is set
        for k in ThrowKind::ALL {
            assert!(!is_primary(Item::Throw(k), 0, false));
        }
    }

    #[test]
    fn every_item_draws_something_and_nothing_overflows_its_box() {
        let mut items: Vec<Item> = ThrowKind::ALL.iter().copied().map(Item::Throw).collect();
        items.push(Item::Shield);
        for k in [
            GunKind::Fists,
            GunKind::Glock,
            GunKind::Deagle,
            GunKind::Mp5,
            GunKind::Shotgun,
            GunKind::Ak47,
            GunKind::M4,
            GunKind::Awm,
            GunKind::M249,
            GunKind::Bow,
            GunKind::Spear,
            GunKind::Minigun,
        ] {
            items.push(Item::Gun(0, k));
        }
        for it in items {
            let parts = icon_parts(it);
            assert!(!parts.is_empty(), "{it:?} draws nothing");
            assert!(
                parts.len() <= MAX_PARTS,
                "{it:?} needs {} parts, cells only spawn {MAX_PARTS}",
                parts.len()
            );
            for p in parts {
                assert!(p.w > 0.0 && p.h > 0.0, "{it:?} has a zero-area part");
                assert!(
                    p.x >= 0.0 && p.y >= 0.0 && p.x + p.w <= ICON && p.y + p.h <= ICON,
                    "{it:?} part {p:?} leaves the {ICON}px icon box"
                );
            }
        }
    }

    #[test]
    fn an_empty_mag_with_reserve_left_is_a_reload_not_an_empty_slot() {
        let g = Item::Gun(0, GunKind::M4);
        assert!(!cell_empty(g, 0, 120, 0), "reloadable must not grey out");
        assert!(cell_empty(g, 0, 0, 0));
        assert!(!cell_empty(g, 1, 0, 0));
        // fists have no magazine and can never run dry
        assert!(!cell_empty(Item::Gun(0, GunKind::Fists), 0, 0, 0));
    }

    #[test]
    fn a_spent_throwable_greys_out() {
        assert!(cell_empty(Item::Throw(ThrowKind::Frag), 0, 0, 0));
        assert!(!cell_empty(Item::Throw(ThrowKind::Frag), 0, 0, 1));
        assert!(!cell_empty(Item::Shield, 0, 0, 0));
    }

    #[test]
    fn counts_read_the_way_the_corner_needs() {
        let c = |i, m, r, g, s| count_text(i, m, r, g, s, false);
        assert_eq!(c(Item::Gun(0, GunKind::M4), 30, 120, 0, false), "30/120");
        assert_eq!(c(Item::Gun(0, GunKind::Fists), 0, 0, 0, false), "--");
        assert_eq!(c(Item::Throw(ThrowKind::Frag), 0, 0, 2, false), "x2");
        assert_eq!(c(Item::Shield, 0, 0, 0, true), "UP");
        assert_eq!(c(Item::Shield, 0, 0, 0, false), "READY");
    }

    #[test]
    fn the_active_cell_never_restates_the_numeral_below_it() {
        // `hud.rs` draws the held weapon's mag at 76px and its reserve
        // at 26px directly beneath this cell. Printing "30/120" here in
        // 11px is the `> TURRET 300` defect that `systems_lines` carries
        // a scar comment about. Fails on the pre-change `count_text`,
        // which had no `primary` argument and always printed.
        assert_eq!(
            count_text(Item::Gun(0, GunKind::M4), 30, 120, 0, false, true),
            "",
            "the active cell must stay silent"
        );
        // ...but ONLY the active one.
        assert_eq!(
            count_text(Item::Gun(1, GunKind::M4), 30, 120, 0, false, false),
            "30/120"
        );
        // the rule is about position, not item type
        assert_eq!(count_text(Item::Shield, 0, 0, 0, true, true), "");
    }

    #[test]
    fn the_icons_are_monochrome_with_gold_on_the_selected_cell_only() {
        // THE design correction, asserted as a relationship rather than
        // as hex values. Fails on the pre-change `icon_parts`, whose
        // parts carried teal/violet/azure/green fills of their own.
        let lum = |c: Color| {
            let s = c.to_srgba();
            0.2126 * s.red + 0.7152 * s.green + 0.0722 * s.blue
        };
        let sat = |c: Color| {
            let s = c.to_srgba();
            let hi = s.red.max(s.green).max(s.blue);
            let lo = s.red.min(s.green).min(s.blue);
            hi - lo
        };

        // 1. GOLD appears in exactly one (tier, tone) pair.
        let tiers = [CellTier::Selected, CellTier::Ready, CellTier::Empty];
        let tones = [Tone::Body, Tone::Detail, Tone::Accent];
        let golds: Vec<_> = tiers
            .iter()
            .flat_map(|t| tones.iter().map(move |n| (*t, *n)))
            .filter(|(t, n)| resolve(*n, *t) == palette::GOLD)
            .collect();
        assert_eq!(
            golds,
            vec![(CellTier::Selected, Tone::Accent)],
            "gold must mark the selected item's accent and nothing else"
        );

        // 2. Everything that is not that one accent is near-neutral.
        //    `INK`/`INK_SOFT`/`INK_FAINT` are all within 0.1 of grey;
        //    the old TINT_PRIMARY teal was 0.48 of chroma.
        for t in tiers {
            for n in tones {
                if (t, n) == (CellTier::Selected, Tone::Accent) {
                    continue;
                }
                let c = resolve(n, t);
                assert!(
                    sat(c) < 0.12,
                    "{t:?}/{n:?} is a saturated hue ({:.2} chroma) - the strip is monochrome",
                    sat(c)
                );
            }
        }

        // 3. The tiers are actually a hierarchy: selected is brighter
        //    than ready, which is brighter than empty. This is what
        //    carries selection now that the cell borders are gone.
        let body = |t| resolve(Tone::Body, t);
        let eff = |t| lum(body(t)) * body(t).alpha();
        assert!(
            eff(CellTier::Selected) > eff(CellTier::Ready),
            "the held weapon must be the brightest thing in the strip"
        );
        assert!(
            eff(CellTier::Ready) > eff(CellTier::Empty),
            "an empty slot must read as demoted"
        );
        // ...but an empty slot is still VISIBLE. Two stacked dimmers
        // once made it nearly invisible, and you need to see that you
        // have a frag slot and it is spent.
        assert!(
            eff(CellTier::Empty) > 0.05,
            "an empty slot has been dimmed out of existence"
        );
    }

    #[test]
    fn the_hierarchy_is_carried_by_size_as_well_as_value() {
        // The reference's active weapon is roughly double its
        // secondaries. On the pre-change 2x4 grid every cell was
        // `CELL_W`, so this ratio was exactly 1.0 and the strip had no
        // subject.
        assert!(
            icon_px(ACTIVE_SLOT) / icon_px(0) >= 1.5,
            "the active icon is not meaningfully larger than a secondary"
        );
        assert!(cell_px(ACTIVE_SLOT) > cell_px(0));
        for s in 0..ACTIVE_SLOT {
            assert_eq!(icon_px(s), ICON_SM, "slot {s} should be a secondary");
        }
    }

    /// THE OVERFLOW BUG, caught by looking at a capture.
    ///
    /// `TextLayout::new_with_no_wrap()` will not break these strings, so
    /// a count wider than its cell overflows both sides and runs
    /// straight through its neighbours. At `CELL_SM_W = 28` the frame
    /// read `17/68 5/10 READY x1` as one illegible line.
    ///
    /// The glyph metric is an ESTIMATE, deliberately generous: this is a
    /// guard rail against a future `count_text` arm returning something
    /// long (`"RELOADING"`), not a substitute for the capture.
    #[test]
    fn every_count_a_cell_can_print_fits_inside_that_cell() {
        // The bundled font is monospace; ~0.62 em per glyph is a safe
        // upper bound for it at these sizes.
        let width = |s: &str| s.chars().count() as f32 * T_COUNT * 0.62;

        let mut samples: Vec<String> = Vec::new();
        for k in [GunKind::M4, GunKind::Awm, GunKind::Fists, GunKind::M249] {
            // the widest a mag/reserve pair gets in this build
            samples.push(count_text(Item::Gun(0, k), 200, 999, 0, false, false));
        }
        samples.push(count_text(Item::Shield, 0, 0, 0, false, false));
        samples.push(count_text(Item::Shield, 0, 0, 0, true, false));
        for k in ThrowKind::ALL {
            samples.push(count_text(Item::Throw(k), 0, 0, 9, false, false));
        }

        for s in samples {
            assert!(
                width(&s) <= CELL_SM_W,
                "count {s:?} is ~{:.0}px wide and will overflow a {CELL_SM_W}px cell \
                 into its neighbours - no_wrap does not clip",
                width(&s)
            );
        }
    }

    #[test]
    fn the_dividers_are_hairlines_and_the_group_rule_is_the_stronger_one() {
        assert!(RULE_W <= 2.0, "a divider that wide is a box, not a rule");
        assert!(RULE_ALPHA < RULE_MAJOR_ALPHA);
        assert!(
            RULE_MAJOR_ALPHA < 0.5,
            "even the group divider must stay subordinate to the icons"
        );
    }

    #[test]
    fn the_keys_are_the_ones_main_actually_binds() {
        assert_eq!(cell_key(Item::Gun(0, GunKind::M4)), Some("1"));
        assert_eq!(cell_key(Item::Gun(1, GunKind::M4)), Some("2"));
        assert_eq!(cell_key(Item::Gun(2, GunKind::M4)), Some("3"));
        assert_eq!(cell_key(Item::Shield), Some("4"));
        assert_eq!(cell_key(Item::Throw(ThrowKind::Smoke)), Some("G"));
    }

    #[test]
    fn the_strip_clears_the_ammo_cluster_and_stays_out_of_the_centre() {
        let b = strip_bottom_frac();
        let ammo_b = 0.09_f32; // HUD_ANCHORS "ammo" offset, by construction
        assert!(
            b >= ammo_b + 0.12,
            "strip bottom {b} does not clear the ammo numeral"
        );
        // bottom-right quadrant: never crosses the centre third, in
        // either axis, at any resolution (the fractions are resolution
        // independent by construction).
        assert!(1.0 - b > 2.0 / 3.0, "strip has climbed into the centre band");
        assert!(strip_right_frac() < 1.0 / 3.0);
    }
}
