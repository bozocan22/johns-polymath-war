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

use crate::frontend::{palette, T_MICRO};
use crate::{sim, Game, GameState, GunKind, HudRoot, ThrowKind, HUD_ANCHORS};

// ---- geometry -------------------------------------------------------------

/// The icon box, in the 720p authoring space the rest of the type ramp
/// uses. `UiScale` handles resolution.
const ICON: f32 = 34.0;
const CELL_W: f32 = 54.0;
/// How far ABOVE the ammo cluster's own anchor the strip sits, in screen
/// fractions. The ammo corner is at `-0.09`; the big numeral plus its
/// sub-line is roughly 0.13 of a 720p screen tall, so 0.155 clears it
/// with room and still lands well below `XII_ANCHORS`' mech-systems
/// column at 0.24 — which this strip never shares a frame with anyway,
/// because it hides in a mech.
const LIFT: f32 = 0.155;

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
    /// Corner radius. Everything here is rounded — that IS the "cute".
    r: f32,
    c: Color,
}

const fn part(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) -> IconPart {
    IconPart { x, y, w, h, r, c }
}

/// The most parts any icon uses. Cells pre-spawn this many and hide the
/// tail, so a repaint never has to touch `Commands`.
const MAX_PARTS: usize = 5;

// The palette is deliberately its own, saturated and friendly, rather
// than `palette::GOLD`/`INK`. The HUD's two-hue rule (threat / systems)
// governs READINGS; these are object identities — a molotov is amber
// because molotovs are amber, and making all four throwables gold would
// destroy the one thing an icon is for, which is telling them apart at a
// glance. Every one is picked light enough to read on the dark scrim.
const GREEN: Color = Color::srgb(0.24, 0.66, 0.32);
const GREEN_HI: Color = Color::srgb(0.48, 0.86, 0.46);
const STEEL: Color = Color::srgb(0.76, 0.79, 0.82);
const CREAM: Color = Color::srgb(0.95, 0.92, 0.72);
const FLARE: Color = Color::srgb(1.00, 0.97, 0.55);
const SLATE: Color = Color::srgb(0.45, 0.56, 0.68);
const SLATE_HI: Color = Color::srgb(0.76, 0.86, 0.94);
const AMBER: Color = Color::srgb(0.92, 0.60, 0.16);
const AMBER_DK: Color = Color::srgb(0.66, 0.40, 0.10);
const FIRE: Color = Color::srgb(1.00, 0.36, 0.12);
const SPARK: Color = Color::srgb(1.00, 0.82, 0.26);
const AZURE: Color = Color::srgb(0.32, 0.56, 0.92);
const AZURE_HI: Color = Color::srgb(0.66, 0.82, 1.00);
const WOOD: Color = Color::srgb(0.64, 0.45, 0.25);
const SKIN: Color = Color::srgb(0.92, 0.74, 0.56);

/// Weapon-class tints. Three hues, so "which of my three slots is a
/// primary" is answerable without reading a word.
const TINT_PRIMARY: Color = Color::srgb(0.28, 0.76, 0.64);
const TINT_SECONDARY: Color = Color::srgb(0.44, 0.70, 1.00);
const TINT_SPECIAL: Color = Color::srgb(0.74, 0.52, 0.96);

/// Darker companion to a tint, for grips, stocks and magazines.
fn shade(c: Color) -> Color {
    let s = c.to_srgba();
    Color::srgb(s.red * 0.45, s.green * 0.45, s.blue * 0.45)
}

/// The class tint of a gun. `GunClass` is the SIM's classification — this
/// does not re-derive "is it a pistol", it colours what the sim already
/// says. `Minigun` is not in any loadout list and the sim calls it a
/// primary, which is the right bucket for it here too.
fn tint(kind: GunKind) -> Color {
    match sim::gun(kind).class {
        sim::GunClass::Primary => TINT_PRIMARY,
        sim::GunClass::Secondary => TINT_SECONDARY,
        sim::GunClass::Special => TINT_SPECIAL,
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
    match item {
        Item::Throw(ThrowKind::Frag) => vec![
            part(6.0, 10.0, 22.0, 21.0, 9.0, GREEN),
            part(10.0, 14.0, 8.0, 6.0, 3.0, GREEN_HI),
            part(14.0, 4.0, 5.0, 9.0, 2.0, STEEL),
            part(20.0, 3.0, 8.0, 8.0, 4.0, SPARK),
        ],
        Item::Throw(ThrowKind::Flash) => vec![
            part(8.0, 10.0, 18.0, 21.0, 7.0, CREAM),
            part(13.0, 16.0, 9.0, 9.0, 4.5, FLARE),
            part(12.0, 3.0, 9.0, 8.0, 3.0, STEEL),
        ],
        Item::Throw(ThrowKind::Smoke) => vec![
            part(9.0, 8.0, 16.0, 23.0, 5.0, SLATE),
            part(9.0, 16.0, 16.0, 5.0, 2.0, SLATE_HI),
            part(13.0, 2.0, 7.0, 7.0, 2.0, STEEL),
        ],
        Item::Throw(ThrowKind::Molotov) => vec![
            part(9.0, 14.0, 16.0, 18.0, 6.0, AMBER),
            part(13.0, 8.0, 8.0, 7.0, 2.0, AMBER_DK),
            part(12.0, 0.0, 10.0, 9.0, 4.5, FIRE),
            part(15.0, 2.0, 4.0, 4.0, 2.0, SPARK),
        ],
        Item::Shield => vec![
            part(6.0, 4.0, 22.0, 26.0, 8.0, AZURE),
            part(6.0, 4.0, 22.0, 5.0, 3.0, AZURE_HI),
            part(13.0, 14.0, 8.0, 8.0, 4.0, SPARK),
        ],
        Item::Gun(_, k) => gun_parts(k),
    }
}

fn gun_parts(k: GunKind) -> Vec<IconPart> {
    let t = tint(k);
    let d = shade(t);
    match k {
        GunKind::Fists => vec![
            part(7.0, 11.0, 19.0, 14.0, 6.0, SKIN),
            part(10.0, 14.0, 12.0, 4.0, 2.0, shade(SKIN)),
            part(23.0, 16.0, 6.0, 7.0, 3.0, SKIN),
        ],
        GunKind::Glock | GunKind::Deagle => vec![
            part(7.0, 11.0, 21.0, 7.0, 2.0, t),
            part(9.0, 16.0, 8.0, 12.0, 2.0, d),
            part(26.0, 12.0, 5.0, 5.0, 2.0, SPARK),
        ],
        GunKind::Shotgun => vec![
            part(2.0, 11.0, 29.0, 5.0, 2.0, t),
            part(10.0, 17.0, 13.0, 5.0, 2.0, d),
            part(1.0, 15.0, 8.0, 8.0, 2.0, d),
            part(24.0, 9.0, 4.0, 4.0, 2.0, SPARK),
        ],
        GunKind::Awm => vec![
            part(2.0, 14.0, 30.0, 4.0, 2.0, t),
            part(11.0, 6.0, 13.0, 6.0, 3.0, SPARK),
            part(2.0, 17.0, 10.0, 8.0, 2.0, d),
            part(16.0, 18.0, 5.0, 9.0, 2.0, d),
        ],
        GunKind::Bow => vec![
            part(13.0, 1.0, 7.0, 14.0, 3.5, t),
            part(13.0, 19.0, 7.0, 14.0, 3.5, t),
            part(12.0, 13.0, 9.0, 8.0, 3.0, d),
            part(24.0, 3.0, 3.0, 28.0, 1.5, SLATE_HI),
        ],
        GunKind::Spear => vec![
            part(7.0, 16.0, 26.0, 4.0, 2.0, WOOD),
            part(0.0, 13.0, 9.0, 9.0, 3.0, SLATE_HI),
            part(15.0, 14.0, 5.0, 8.0, 2.0, SPARK),
        ],
        // every belt/box-fed longarm shares the rifle outline
        GunKind::Mp5 | GunKind::Ak47 | GunKind::M4 | GunKind::M249 | GunKind::Minigun => vec![
            part(3.0, 12.0, 28.0, 6.0, 2.0, t),
            part(13.0, 18.0, 7.0, 11.0, 2.0, d),
            part(0.0, 14.0, 6.0, 8.0, 2.0, d),
            part(20.0, 7.0, 5.0, 5.0, 2.0, SPARK),
        ],
    }
}

// ---- pure formatters ------------------------------------------------------

/// The cells, in strip order, for a given loadout.
///
/// TWO ROWS, and the order is not arbitrary: throwables on top, guns
/// below, because the big ammo numeral sits directly under the strip and
/// describes the SELECTED GUN. Putting the gun row adjacent to the
/// numeral keeps the thing and its number together; a throwables row
/// wedged between them would separate them.
fn strip_rows(inv: &[GunKind; 3]) -> [Vec<Item>; 2] {
    [
        ThrowKind::ALL.iter().copied().map(Item::Throw).collect(),
        (0..3)
            .map(|i| Item::Gun(i, inv[i]))
            .chain(std::iter::once(Item::Shield))
            .collect(),
    ]
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
fn count_text(item: Item, mag: u32, reserve: u32, grenades: u8, shield_up: bool) -> String {
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

/// Flat cell count: 4 throwables + 3 guns + shield.
const CELLS: usize = 8;
const ROW_LEN: usize = 4;

// ---- plugin ---------------------------------------------------------------

pub struct InventoryStripPlugin;

impl Plugin for InventoryStripPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_strip).add_systems(
            Update,
            paint_strip.run_if(in_state(GameState::Playing)),
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
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(6.0),
                ..default()
            },
            // Starts hidden for the same reason the XII root does: the
            // initial state is Title and `hud_visibility` only fires on
            // Playing's enter/exit, so a root that spawned visible would
            // sit on top of the title screen until the first match.
            Visibility::Hidden,
            HudRoot,
            StripRoot,
        ))
        .with_children(|r| {
            for row in 0..2 {
                r.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|rw| {
                    for col in 0..ROW_LEN {
                        spawn_cell(rw, row * ROW_LEN + col);
                    }
                });
            }
        });
}

fn spawn_cell(b: &mut ChildBuilder, index: usize) {
    b.spawn((
        Node {
            width: Val::Px(CELL_W),
            height: Val::Px(CELL_W + 10.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            padding: UiRect::all(Val::Px(3.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(palette::PANEL.with_alpha(0.55)),
        BorderColor(palette::INK_FAINT.with_alpha(0.35)),
        BorderRadius::all(Val::Px(7.0)),
        InvCell(index),
    ))
    .with_children(|c| {
        // the icon box — parts are absolute inside it
        c.spawn(Node {
            width: Val::Px(ICON),
            height: Val::Px(ICON),
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
                    BorderRadius::all(Val::Px(2.0)),
                    Visibility::Hidden,
                    CellIconPart(index, p),
                ));
            }
        });
        c.spawn((
            Text::new(""),
            TextFont {
                font_size: T_MICRO,
                ..default()
            },
            TextColor(palette::INK_SOFT),
            TextLayout::new_with_no_wrap(),
            CellCount(index),
        ));
        // selected accent bar, pinned to the cell's bottom edge
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(4.0),
                right: Val::Px(4.0),
                bottom: Val::Px(2.0),
                height: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderRadius::all(Val::Px(2.0)),
            CellAccent(index),
        ));
        // the keybind badge, hanging off the top-right corner
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-7.0),
                right: Val::Px(-6.0),
                width: Val::Px(16.0),
                height: Val::Px(16.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(palette::PANEL_HI),
            BorderRadius::all(Val::Px(8.0)),
            CellKeyBadge(index),
        ))
        .with_children(|k| {
            k.spawn((
                Text::new(""),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(palette::INK),
                TextLayout::new_with_no_wrap(),
                CellKeyText(index),
            ));
        });
        // the dim veil, last so it covers icon and number
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
            BorderRadius::all(Val::Px(7.0)),
            CellVeil(index),
        ));
    });
}

/// Everything the painter needs about one cell, resolved once.
struct CellState {
    item: Item,
    selected: bool,
    empty: bool,
    count: String,
    key: Option<&'static str>,
}

fn cell_states(p: &sim::Fighter) -> Vec<CellState> {
    let rows = strip_rows(&p.inventory);
    let mut out = Vec::with_capacity(CELLS);
    for item in rows.into_iter().flatten() {
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
        let selected = match item {
            Item::Gun(i, _) => i == p.active && !p.shield_up,
            Item::Shield => p.shield_up,
            Item::Throw(k) => ThrowKind::ALL[p.throw_sel as usize] == k,
        };
        out.push(CellState {
            item,
            selected,
            empty: cell_empty(item, mag, reserve, nades),
            count: count_text(item, mag, reserve, nades, p.shield_up),
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
    mut q: ParamSet<(
        Query<(&InvCell, &mut BorderColor, &mut BackgroundColor)>,
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
    )>,
) {
    let simr = &game.sim;
    let p = &simr.fighters[simr.player];

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

    for (cell, mut border, mut bg) in q.p0().iter_mut() {
        let Some(s) = states.get(cell.0) else { continue };
        *border = BorderColor(if s.selected {
            palette::GOLD
        } else {
            palette::INK_FAINT.with_alpha(0.35)
        });
        *bg = BackgroundColor(if s.selected {
            palette::PANEL_HI.with_alpha(0.92)
        } else {
            palette::PANEL.with_alpha(0.55)
        });
    }

    for (part_id, mut node, mut bg, mut radius, mut vis) in q.p1().iter_mut() {
        let Some(s) = states.get(part_id.0) else {
            continue;
        };
        let parts = icon_parts(s.item);
        match parts.get(part_id.1) {
            Some(ip) => {
                node.left = Val::Px(ip.x);
                node.top = Val::Px(ip.y);
                node.width = Val::Px(ip.w);
                node.height = Val::Px(ip.h);
                *bg = BackgroundColor(ip.c);
                *radius = BorderRadius::all(Val::Px(ip.r));
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }

    for (cc, mut t, mut c) in q.p2().iter_mut() {
        let Some(s) = states.get(cc.0) else { continue };
        **t = s.count.clone();
        *c = TextColor(if s.selected {
            palette::INK
        } else {
            palette::INK_SOFT
        });
    }

    for (veil, mut bg) in q.p3().iter_mut() {
        let Some(s) = states.get(veil.0) else { continue };
        *bg = BackgroundColor(if s.empty {
            palette::GROUND.with_alpha(0.62)
        } else {
            Color::NONE
        });
    }

    for (acc, mut bg) in q.p4().iter_mut() {
        let Some(s) = states.get(acc.0) else { continue };
        *bg = BackgroundColor(if s.selected { palette::GOLD } else { Color::NONE });
    }

    for (badge, mut bg, mut vis) in q.p5().iter_mut() {
        let Some(s) = states.get(badge.0) else {
            continue;
        };
        // A throwable cell that is not the selected one has no badge:
        // see `cell_key`.
        let shown = s.key.is_some() && (!matches!(s.item, Item::Throw(_)) || s.selected);
        *vis = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        *bg = BackgroundColor(if s.selected {
            palette::GOLD
        } else {
            palette::PANEL_HI
        });
    }

    for (kt, mut t, mut c) in q.p6().iter_mut() {
        let Some(s) = states.get(kt.0) else { continue };
        **t = s.key.unwrap_or("").to_string();
        *c = TextColor(if s.selected {
            palette::GROUND
        } else {
            palette::INK_SOFT
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m4_kit() -> [GunKind; 3] {
        [GunKind::M4, GunKind::Glock, GunKind::Awm]
    }

    #[test]
    fn the_strip_has_one_cell_per_carried_item() {
        let rows = strip_rows(&m4_kit());
        assert_eq!(rows[0].len(), ThrowKind::ALL.len());
        assert_eq!(rows[1].len(), 4, "three guns plus the shield");
        assert_eq!(rows[0].len() + rows[1].len(), CELLS);
        assert_eq!(rows[0].len(), ROW_LEN);
    }

    #[test]
    fn gun_row_sits_below_the_throwables_and_next_to_the_numeral() {
        // The ordering claim in `strip_rows`' doc comment, asserted:
        // row 1 (the lower one, nearest the ammo numeral) is the guns.
        let rows = strip_rows(&m4_kit());
        assert!(matches!(rows[1][0], Item::Gun(0, GunKind::M4)));
        assert!(matches!(rows[0][0], Item::Throw(_)));
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
        assert_eq!(count_text(Item::Gun(0, GunKind::M4), 30, 120, 0, false), "30/120");
        assert_eq!(count_text(Item::Gun(0, GunKind::Fists), 0, 0, 0, false), "--");
        assert_eq!(count_text(Item::Throw(ThrowKind::Frag), 0, 0, 2, false), "x2");
        assert_eq!(count_text(Item::Shield, 0, 0, 0, true), "UP");
        assert_eq!(count_text(Item::Shield, 0, 0, 0, false), "READY");
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
