//! THE EDGE LOADOUT CARD — a transient, right-edge-docked summary of the
//! five loadout slots that is invisible until the loadout CHANGES.
//!
//! ## Why this is its own module
//!
//! Same argument `branding.rs` and `inventory_strip.rs` make: `main.rs`
//! is ~28k lines and is the most contended file in the repo. The wiring
//! over there is two lines nobody else's diff can collide with:
//!
//! ```ignore
//! mod loadout_edge;
//! // ...
//! .add_plugins(loadout_edge::LoadoutEdgePlugin)
//! ```
//!
//! ## This is NOT the inventory strip
//!
//! `inventory_strip.rs` owns the permanent bottom-centre/right row inside
//! `hud.rs`'s ammo panel. Nothing here touches it, reads its components,
//! or spawns into its host. The two answer different questions:
//!
//! * the strip: "what am I carrying, all match long" (always on screen)
//! * this card: "what just changed, and what is the rest of my kit" —
//!   on screen for ~1.5 s after a loadout event and hidden otherwise.
//!
//! Both read the SAME sim fields (`p.inventory`, `p.active`,
//! `p.slot_ammo`, `p.shield_up`, `p.grenades`, `p.throw_sel`). There is
//! no second inventory model in this file, and no `ResMut<Game>` in it
//! either — every system here takes `Res<Game>`.
//!
//! ## The change signal is a SIGNATURE, not an event
//!
//! There is no `LoadoutChanged` event in the sim, and adding one would
//! be `sim.rs`, which is another lane's file. So the trigger is a
//! `Sig` — a small `Copy` tuple of exactly the fields the spec names as
//! loadout changes (equipped guns, active slot, shield stance, selected
//! throwable, throwable counts). Comparing it is ~20 bytes of `==` per
//! frame; a rebuild only happens on the frames where it differs.
//!
//! Note what is deliberately NOT in the signature: the magazine and
//! reserve counts. Firing a rifle would otherwise pop this card up on
//! every single bullet, which is the opposite of "auto-hiding". The
//! ammo numbers are still PRINTED, they are just not a trigger. Grenade
//! counts ARE a trigger, because the spec asks for that explicitly and
//! a throw is a discrete event rather than a stream.
//!
//! ## Cosmetic only
//!
//! Real delta-time, frame-rate dependent easing, no RNG, nothing fed
//! back into the sim.

use bevy::prelude::*;

use crate::frontend::palette;
use crate::{sim, Game, GameState, GunKind};

// ---- timing ---------------------------------------------------------------

/// Fade-in duration, seconds. Spec: 100-150 ms.
pub(crate) const FADE_IN_S: f32 = 0.12;
/// How long the card stays lit after the last change. Spec: ~1.5 s.
pub(crate) const HOLD_S: f32 = 1.5;
/// Fade-out duration, seconds. Spec: 200-300 ms.
pub(crate) const FADE_OUT_S: f32 = 0.25;
/// How far off the right edge the card starts, in px. Small on purpose —
/// the spec forbids a "large sliding panel", so this is a nudge that
/// reads as motion, not a drawer.
pub(crate) const SLIDE_PX: f32 = 14.0;

// ---- geometry -------------------------------------------------------------

/// Gap between the card and the right screen edge. Spec: 0-5 px.
const EDGE_MARGIN: f32 = 4.0;
// WIDE ENOUGH FOR THE WIDEST COUNT. This was 66, and the first
// capture caught it: `30/120` is six glyphs of a 10px monospace face,
// centred in a 66px card with 4px of padding, and it printed past both
// sides of the plate. `inventory_strip.rs` carries the same scar for
// the same reason - a count string is not the width you guessed.
const CARD_W: f32 = 84.0;
// TALL ENOUGH THAT THE LABEL IS NOT INSIDE THE ICON. At 44 the three
// stacked rows (22 icon + label + count) summed to more than the box
// once line-height was applied, so `SHD` and `NADE` were printing over
// the foot of their own glyphs.
const SLOT_H: f32 = 50.0;
const ICON: f32 = 22.0;
/// Icons are authored in this box and drawn at exactly this size, so
/// every literal in `icon_parts` is a real pixel.
const ICON_BOX: f32 = ICON;
const RULE_H: f32 = 1.0;
const MAX_PARTS: usize = 4;
pub(crate) const SLOTS: usize = 5;

const T_LABEL: f32 = 9.0;
const T_COUNT: f32 = 10.0;

/// Panel fill. Spec: dark, ~30-40% alpha.
const PLATE_A: f32 = 0.36;
/// Separator alpha — "thin subtle".
const RULE_A: f32 = 0.22;
/// The selected slot's border. "Subtle highlight border, no heavy glow".
const SEL_BORDER_A: f32 = 0.55;

// ---- slot model -----------------------------------------------------------

/// The five fixed slots, always in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Primary,
    Secondary,
    Special,
    Shield,
    Grenades,
}

impl Slot {
    pub(crate) const ALL: [Slot; SLOTS] = [
        Slot::Primary,
        Slot::Secondary,
        Slot::Special,
        Slot::Shield,
        Slot::Grenades,
    ];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Slot::Primary => "PRI",
            Slot::Secondary => "SEC",
            Slot::Special => "SPC",
            Slot::Shield => "SHD",
            Slot::Grenades => "NADE",
        }
    }
    /// The inventory index a gun slot maps to, if it is a gun slot.
    pub(crate) fn gun_index(self) -> Option<usize> {
        match self {
            Slot::Primary => Some(0),
            Slot::Secondary => Some(1),
            Slot::Special => Some(2),
            _ => None,
        }
    }
}

/// How a slot is reading. Drives one alpha multiplier for the whole
/// slot, which is why there is exactly one place that decides what
/// "selected" looks like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tier {
    /// The item in your hands (or the raised shield). Brightest.
    Active,
    /// Carried and usable. Full-ish.
    Owned,
    /// Not carried / out of stock. ~50%, still visible.
    Unowned,
}

impl Tier {
    /// Slot-level opacity BEFORE the fade multiplier.
    pub(crate) fn alpha(self) -> f32 {
        match self {
            Tier::Active => 1.0,
            Tier::Owned => 0.82,
            Tier::Unowned => 0.5,
        }
    }
}

// ---- the change signature -------------------------------------------------

/// Exactly the fields the spec calls a "loadout change". See the module
/// header for why magazine counts are absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Sig {
    pub guns: [GunKind; 3],
    pub active: usize,
    pub shield_up: bool,
    pub throw_sel: u8,
    pub nades: [u8; 4],
}

/// Which slot the transition from `old` to `new` should highlight.
///
/// `None` means "nothing the card cares about moved" — the caller keeps
/// whatever was highlighted before rather than flickering to slot 0.
///
/// Priority is weapon > shield > throwable, because a weapon swap is
/// the loudest thing a player can do and two of these can land on the
/// same frame (pressing 2 while a grenade count ticks down).
pub(crate) fn changed_slot(old: Sig, new: Sig) -> Option<usize> {
    if old.guns != new.guns || old.active != new.active {
        return Some(new.active.min(2));
    }
    if old.shield_up != new.shield_up {
        return Some(3);
    }
    if old.throw_sel != new.throw_sel || old.nades != new.nades {
        return Some(4);
    }
    None
}

// ---- the fade clock -------------------------------------------------------

/// The whole auto-hide mechanic, as a value.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct Anim {
    /// 0 = fully hidden, 1 = fully lit.
    pub alpha: f32,
    /// Seconds of hold left. While positive the card fades IN and stays.
    pub hold: f32,
}

/// A loadout change landed: reset the visibility timer.
///
/// Crucially this does NOT touch `alpha`. That is spec point 4 — a
/// second change while the card is already up must not restart the
/// fade-in, or rapid weapon cycling strobes.
pub(crate) fn trigger(a: Anim) -> Anim {
    Anim { hold: HOLD_S, ..a }
}

/// One frame of the fade. Pure, so the whole mechanic is unit-testable
/// without a `World`.
pub(crate) fn tick(a: Anim, dt: f32) -> Anim {
    if a.hold > 0.0 {
        Anim {
            alpha: (a.alpha + dt / FADE_IN_S).min(1.0),
            hold: (a.hold - dt).max(0.0),
        }
    } else {
        Anim {
            alpha: (a.alpha - dt / FADE_OUT_S).max(0.0),
            hold: 0.0,
        }
    }
}

/// The slide-in offset in px: full offset when hidden, zero when lit.
/// Eased so it decelerates into the edge — no bounce, no overshoot.
pub(crate) fn slide_px(alpha: f32) -> f32 {
    let e = 1.0 - (1.0 - alpha.clamp(0.0, 1.0)).powi(2);
    SLIDE_PX * (1.0 - e)
}

// ---- content --------------------------------------------------------------

pub(crate) fn tier_for(slot: Slot, sig: Sig) -> Tier {
    match slot {
        Slot::Primary | Slot::Secondary | Slot::Grenades | Slot::Special => {
            if let Some(i) = slot.gun_index() {
                if sig.guns[i] == GunKind::Fists {
                    Tier::Unowned
                } else if sig.active == i && !sig.shield_up {
                    Tier::Active
                } else {
                    Tier::Owned
                }
            } else {
                // Grenades
                let n = sig.nades[(sig.throw_sel as usize).min(3)];
                if n == 0 {
                    Tier::Unowned
                } else {
                    Tier::Owned
                }
            }
        }
        Slot::Shield => {
            if sig.shield_up {
                Tier::Active
            } else {
                Tier::Owned
            }
        }
    }
}

/// The quantity/durability string for a slot.
///
/// The shield prints a STANCE, not a durability number: there is no
/// infantry shield-hp field in the sim (`shield_up: bool` is the whole
/// model; `mech_shield_hp` belongs to a chassis, not to this card), and
/// inventing an 0-100 here would be exactly the split brain this repo
/// keeps re-shipping. Stated rather than implied.
pub(crate) fn slot_text(slot: Slot, sig: Sig, slot_ammo: [(u32, u32); 3]) -> String {
    match slot.gun_index() {
        Some(i) => {
            let k = sig.guns[i];
            if k == GunKind::Fists {
                "--".to_string()
            } else if sim::gun(k).mag == 0 {
                "--".to_string()
            } else {
                let (mag, res) = slot_ammo[i];
                format!("{mag}/{res}")
            }
        }
        None => match slot {
            Slot::Shield => if sig.shield_up { "UP" } else { "READY" }.to_string(),
            _ => format!("x{}", sig.nades[(sig.throw_sel as usize).min(3)]),
        },
    }
}

// ---- flat icons -----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tone {
    Body,
    Detail,
}

#[derive(Clone, Copy, Debug)]
struct IconPart {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    t: Tone,
}

const fn part(x: f32, y: f32, w: f32, h: f32, t: Tone) -> IconPart {
    IconPart { x, y, w, h, t }
}

/// The thinnest stroke a 22px glyph may use. Two smudges, one vanishes —
/// `inventory_strip.rs` learned this the expensive way and the number is
/// carried over rather than re-discovered.
const MIN_STROKE: f32 = 3.0;

/// One silhouette per CATEGORY, not per weapon. This card's job is
/// "which of my five slots", so a category glyph is the correct
/// abstraction and also the only one that survives 22 px.
fn icon_parts(slot: Slot) -> Vec<IconPart> {
    use Tone::{Body, Detail};
    match slot {
        // Long gun: receiver, barrel, magazine, stock.
        Slot::Primary => vec![
            part(1.0, 8.0, 20.0, 4.0, Body),
            part(14.0, 5.0, 7.0, 3.0, Detail),
            part(7.0, 12.0, 4.0, 7.0, Detail),
            part(1.0, 11.0, 4.0, 5.0, Body),
        ],
        // Sidearm: slide over a grip.
        Slot::Secondary => vec![
            part(3.0, 7.0, 16.0, 5.0, Body),
            part(5.0, 12.0, 5.0, 8.0, Detail),
            part(16.0, 8.0, 4.0, 3.0, Detail),
        ],
        // Special: a scoped long barrel — bipod legs read at 22px where
        // a scope alone does not.
        Slot::Special => vec![
            part(0.0, 9.0, 22.0, 4.0, Body),
            part(6.0, 4.0, 9.0, 4.0, Detail),
            part(4.0, 13.0, 3.0, 6.0, Detail),
            part(9.0, 13.0, 3.0, 6.0, Detail),
        ],
        // Kite shield: broad shoulders, tapered foot.
        // A CREST OVER A TAPERED FOOT. The first capture read this as a
        // bucket or a funnel: the top band was as wide as the body, so
        // the outline was a rectangle that suddenly narrowed, which is
        // a vessel, not a shield. Insetting the crest gives the
        // shoulders their curve and moves the taper to the bottom third
        // where a kite shield actually has it.
        Slot::Shield => vec![
            part(5.0, 1.0, 12.0, 3.0, Detail),
            part(3.0, 4.0, 16.0, 9.0, Body),
            part(5.0, 13.0, 12.0, 4.0, Body),
            part(8.0, 17.0, 6.0, 4.0, Body),
        ],
        // Grenade: body, band, neck, spoon.
        Slot::Grenades => vec![
            part(5.0, 8.0, 12.0, 13.0, Body),
            part(5.0, 13.0, 12.0, 3.0, Detail),
            part(9.0, 3.0, 4.0, 5.0, Detail),
            part(13.0, 3.0, 6.0, 3.0, Detail),
        ],
    }
}

fn tone_color(t: Tone) -> Color {
    match t {
        Tone::Body => palette::INK,
        Tone::Detail => palette::INK_SOFT,
    }
}

// ---- components / resources ----------------------------------------------

#[derive(Component)]
struct EdgeRoot;
#[derive(Component)]
struct EdgeSlotBox(usize);
#[derive(Component)]
struct EdgeIconPart(usize, usize);
#[derive(Component)]
struct EdgeLabel(usize);
#[derive(Component)]
struct EdgeCount(usize);
#[derive(Component)]
struct EdgeRule(usize);

/// The card's whole runtime state. One resource, no per-entity state.
#[derive(Resource, Default)]
struct EdgeState {
    anim: Anim,
    /// `None` until the first frame of a match — the first signature is
    /// adopted SILENTLY, so spawning into a match does not pop the card
    /// up before the player has done anything.
    sig: Option<Sig>,
    /// Which slot to ring. Held between events.
    highlight: Option<usize>,
    /// Per-slot base alpha, resolved on a change frame only.
    tier_alpha: [f32; SLOTS],
    /// Set on a change frame; consumed by the repaint system.
    dirty: bool,
}

fn read_sig(game: &Game) -> Option<(Sig, [(u32, u32); 3])> {
    let s = &game.sim;
    let p = s.fighters.get(s.player)?;
    Some((
        Sig {
            guns: p.inventory,
            active: p.active,
            shield_up: p.shield_up,
            throw_sel: p.throw_sel,
            nades: p.grenades,
        },
        p.slot_ammo,
    ))
}

// ---- spawn ----------------------------------------------------------------

fn spawn_card(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(EDGE_MARGIN + SLIDE_PX),
                top: Val::Percent(50.0),
                margin: UiRect::top(Val::Px(-(SLOT_H * SLOTS as f32) / 2.0)),
                width: Val::Px(CARD_W),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::vertical(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderRadius::all(Val::Px(4.0)),
            // Hidden by default — the entire mechanic in one line.
            Visibility::Hidden,
            ZIndex(30),
            EdgeRoot,
        ))
        .with_children(|r| {
            for (i, slot) in Slot::ALL.iter().enumerate() {
                if i > 0 {
                    r.spawn((
                        Node {
                            width: Val::Px(CARD_W - 14.0),
                            height: Val::Px(RULE_H),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        EdgeRule(i),
                    ));
                }
                spawn_slot(r, i, *slot);
            }
        });
}

fn spawn_slot(b: &mut ChildBuilder, index: usize, slot: Slot) {
    b.spawn((
        Node {
            width: Val::Px(CARD_W - 8.0),
            height: Val::Px(SLOT_H - RULE_H),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor(Color::NONE),
        BorderRadius::all(Val::Px(3.0)),
        EdgeSlotBox(index),
    ))
    .with_children(|c| {
        c.spawn(Node {
            width: Val::Px(ICON_BOX),
            height: Val::Px(ICON_BOX),
            position_type: PositionType::Relative,
            ..default()
        })
        .with_children(|ib| {
            let parts = icon_parts(slot);
            for p in 0..MAX_PARTS {
                let node = match parts.get(p) {
                    Some(g) => Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(g.x),
                        top: Val::Px(g.y),
                        width: Val::Px(g.w),
                        height: Val::Px(g.h),
                        ..default()
                    },
                    None => Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                };
                ib.spawn((
                    node,
                    BackgroundColor(Color::NONE),
                    BorderRadius::all(Val::Px(1.0)),
                    EdgeIconPart(index, p),
                ));
            }
        });
        c.spawn((
            Text::new(slot.label()),
            TextFont {
                font_size: T_LABEL,
                ..default()
            },
            TextColor(Color::NONE),
            TextLayout::new_with_no_wrap(),
            Node {
                margin: UiRect::vertical(Val::Px(1.0)),
                ..default()
            },
            EdgeLabel(index),
        ));
        c.spawn((
            Text::new("--"),
            TextFont {
                font_size: T_COUNT,
                ..default()
            },
            TextColor(Color::NONE),
            TextLayout::new_with_no_wrap(),
            EdgeCount(index),
        ));
    });
}

// ---- detect ---------------------------------------------------------------

/// The ONLY system that reads the loadout. Cheap: one struct compare.
fn detect_change(game: Res<Game>, mut st: ResMut<EdgeState>) {
    let Some((sig, _)) = read_sig(&game) else {
        return;
    };
    match st.sig {
        None => {
            // First sighting: adopt silently, no pop.
            st.sig = Some(sig);
            st.dirty = true;
        }
        Some(old) if old != sig => {
            if let Some(h) = changed_slot(old, sig) {
                st.highlight = Some(h);
            }
            st.sig = Some(sig);
            st.anim = trigger(st.anim);
            st.dirty = true;
        }
        _ => {}
    }
}

// ---- repaint (change frames only) ----------------------------------------

#[allow(clippy::type_complexity)]
fn repaint(
    game: Res<Game>,
    mut st: ResMut<EdgeState>,
    mut counts: Query<(&EdgeCount, &mut Text)>,
) {
    if !st.dirty {
        return;
    }
    st.dirty = false;
    let Some((sig, ammo)) = read_sig(&game) else {
        return;
    };
    for (i, slot) in Slot::ALL.iter().enumerate() {
        st.tier_alpha[i] = tier_for(*slot, sig).alpha();
    }
    for (c, mut t) in &mut counts {
        if let Some(slot) = Slot::ALL.get(c.0) {
            let s = slot_text(*slot, sig, ammo);
            if t.0 != s {
                t.0 = s;
            }
        }
    }
}

// ---- fade (every frame, arithmetic only) ---------------------------------

#[allow(clippy::type_complexity)]
fn fade(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut st: ResMut<EdgeState>,
    mut root: Query<(&mut Visibility, &mut Node), With<EdgeRoot>>,
    mut q: ParamSet<(
        Query<(&EdgeIconPart, &mut BackgroundColor)>,
        Query<(&EdgeLabel, &mut TextColor)>,
        Query<(&EdgeCount, &mut TextColor)>,
        Query<(&EdgeSlotBox, &mut BorderColor)>,
        Query<&mut BackgroundColor, With<EdgeRoot>>,
        Query<(&EdgeRule, &mut BackgroundColor)>,
    )>,
) {
    let dt = time.delta_secs();
    if *state.get() == GameState::Playing {
        st.anim = tick(st.anim, dt);
    } else {
        // Off the battlefield the card is not a thing. Drop it hard so
        // it can never sit on top of a menu.
        st.anim = Anim::default();
        st.sig = None;
    }
    let a = st.anim.alpha;

    let Ok((mut vis, mut node)) = root.get_single_mut() else {
        return;
    };
    let want = if a <= 0.001 {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    if *vis != want {
        *vis = want;
    }
    if want == Visibility::Hidden {
        // Nothing to interpolate while hidden — this is the cheap path
        // the card spends nearly all of its life on.
        return;
    }
    node.right = Val::Px(EDGE_MARGIN + slide_px(a));

    let hi = st.highlight;
    let sa = st.tier_alpha;
    let mul = |i: usize| sa.get(i).copied().unwrap_or(1.0) * a;

    for (p, mut bg) in &mut q.p0() {
        let parts = icon_parts(Slot::ALL[p.0.min(SLOTS - 1)]);
        bg.0 = match parts.get(p.1) {
            Some(g) => tone_color(g.t).with_alpha(mul(p.0)),
            None => Color::NONE,
        };
    }
    for (l, mut tc) in &mut q.p1() {
        tc.0 = palette::INK_SOFT.with_alpha(mul(l.0) * 0.85);
    }
    for (c, mut tc) in &mut q.p2() {
        tc.0 = palette::INK.with_alpha(mul(c.0));
    }
    for (s, mut bc) in &mut q.p3() {
        bc.0 = if hi == Some(s.0) {
            palette::GOLD.with_alpha(SEL_BORDER_A * a)
        } else {
            Color::NONE
        };
    }
    if let Ok(mut bg) = q.p4().get_single_mut() {
        bg.0 = Color::srgba(0.0, 0.0, 0.0, PLATE_A * a);
    }
    for (_, mut bg) in &mut q.p5() {
        bg.0 = palette::INK_FAINT.with_alpha(RULE_A * a);
    }
}

// ---- plugin ---------------------------------------------------------------

pub struct LoadoutEdgePlugin;

impl Plugin for LoadoutEdgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EdgeState>()
            .add_systems(Startup, spawn_card)
            .add_systems(
                Update,
                (
                    detect_change.run_if(in_state(GameState::Playing)),
                    repaint.after(detect_change),
                    fade.after(repaint),
                ),
            );
    }
}

// ---- capture --------------------------------------------------------------

/// The card's own instrument. It has to exist, because NO script in
/// `CAPTURE_SCRIPTS` could photograph this feature: the card is hidden
/// unless a loadout change landed within the last ~1.75 s, and a frame
/// is only meaningful if you know how long ago the last key went down.
/// Every other script's snaps are timed for a camera, not for a fade.
///
/// So the beats below are timed AGAINST the clock in this file:
///
/// * `01-hidden` at t=1.0 — a full second with no input. Nothing on the
///   right edge. This is the default state and is half the evidence.
/// * a `Digit2` press at 1.4, snapped at 1.6 — 200 ms after the event,
///   i.e. past `FADE_IN_S` and deep inside `HOLD_S`. The card is lit and
///   the SECONDARY slot is ringed.
/// * a `Digit3` press at 2.2, snapped at 2.4 — the reset case. If the
///   timer stacked instead of resetting, this frame would be wrong.
/// * `04-hidden-again` at 4.6 — 2.4 s after the last event, which is
///   past `HOLD_S + FADE_OUT_S` (1.75 s). Back to nothing.
///
/// Timings are held one beat apart from any camera move so a snap is
/// never taken on the same frame as a teleport.
pub mod capture {
    use crate::{beat, CapBeat, CapKey};
    use bevy::prelude::KeyCode;

    pub const SCRIPT: &str = "loadout_edge";

    pub const BEATS: &[CapBeat] = &[
        // Settle the camera first; nothing here touches the loadout, so
        // the card must still be down when the first frame is taken.
        CapBeat {
            look: Some((0.0, 0.02)),
            boom: Some(2.6),
            ..beat(0.3)
        },
        CapBeat {
            snap: Some("01-hidden"),
            ..beat(1.0)
        },
        // ---- event 1: swap to the secondary ----
        CapBeat {
            press: &[CapKey::K(KeyCode::Digit2)],
            ..beat(1.4)
        },
        CapBeat {
            release: &[CapKey::K(KeyCode::Digit2)],
            ..beat(1.5)
        },
        CapBeat {
            snap: Some("02-shown-secondary"),
            ..beat(1.6)
        },
        // ---- event 2: swap to the special, WHILE still visible ----
        CapBeat {
            press: &[CapKey::K(KeyCode::Digit3)],
            ..beat(2.2)
        },
        CapBeat {
            release: &[CapKey::K(KeyCode::Digit3)],
            ..beat(2.3)
        },
        CapBeat {
            snap: Some("03-shown-special-timer-reset"),
            ..beat(2.4)
        },
        // MID-FADE-OUT. This beat was at 4.0 s and photographed nothing:
        // the last event lands at 2.3, the hold runs to 3.8 and the
        // fade-out is over by 4.05, so 4.0 was already past the end of
        // the animation. The window is 250 ms wide and this is the
        // middle of it - the frame that proves the fade is a fade and
        // not a pop.
        CapBeat {
            snap: Some("04-fading-out"),
            ..beat(3.92)
        },
        // 2.4 s after the last event — past HOLD + FADE_OUT.
        CapBeat {
            snap: Some("05-hidden-again"),
            ..beat(4.7)
        },
        CapBeat { end: true, ..beat(5.2) },
    ];
}

// ---- tests ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sig() -> Sig {
        Sig {
            guns: [GunKind::M4, GunKind::Glock, GunKind::Awm],
            active: 0,
            shield_up: false,
            throw_sel: 0,
            nades: [2, 1, 1, 0],
        }
    }

    #[test]
    fn a_fresh_card_is_fully_hidden() {
        assert_eq!(Anim::default().alpha, 0.0);
        assert_eq!(Anim::default().hold, 0.0);
    }

    #[test]
    fn fade_in_reaches_full_within_the_spec_window() {
        // 100-150 ms. Step at 60 Hz from hidden.
        let mut a = trigger(Anim::default());
        let mut t = 0.0;
        while a.alpha < 1.0 && t < 1.0 {
            a = tick(a, 1.0 / 60.0);
            t += 1.0 / 60.0;
        }
        assert!(a.alpha >= 1.0, "never reached full: {a:?}");
        assert!((0.09..=0.16).contains(&t), "fade-in took {t}s");
    }

    #[test]
    fn it_hides_again_about_one_point_seven_five_seconds_after_one_event() {
        let mut a = trigger(Anim::default());
        let mut t = 0.0;
        // Run past the hold, then the fade-out. `t == 0` is excluded
        // because the card legitimately starts at alpha 0 on the frame
        // the event lands — the loop must not exit before it lights up.
        while (t == 0.0 || a.alpha > 0.0) && t < 10.0 {
            a = tick(a, 1.0 / 240.0);
            t += 1.0 / 240.0;
        }
        assert_eq!(a.alpha, 0.0);
        // A LITERAL, not `HOLD_S + FADE_OUT_S`. Deriving the expected
        // value from the constants under test is the defect OPERATION.md
        // rule 12 names: the mutation run proved it, because this test
        // still PASSED with HOLD_S moved to five seconds. 1.75 is the
        // spec's number (~1.5 s visible + a 200-300 ms fade), and the
        // test is now allowed to disagree with the code.
        //
        // HOLD starts ticking during the fade-in, so the total is
        // HOLD + FADE_OUT, not HOLD + FADE_IN + FADE_OUT.
        let want: f32 = 1.75;
        assert!((t - want).abs() < 0.05, "hidden after {t}s, wanted ~{want}");
    }

    #[test]
    fn a_second_event_resets_the_timer_without_restarting_the_fade() {
        let mut a = trigger(Anim::default());
        for _ in 0..60 {
            a = tick(a, 1.0 / 60.0);
        }
        assert_eq!(a.alpha, 1.0);
        let before = a.alpha;
        a = trigger(a);
        // Point 4: content updates and the timer resets; the fade does
        // NOT restart from zero.
        assert_eq!(a.alpha, before, "second event re-ran the fade-in");
        assert_eq!(a.hold, HOLD_S, "second event did not reset the timer");
    }

    #[test]
    fn an_event_during_the_fade_out_recovers_instead_of_popping() {
        let mut a = Anim { alpha: 0.4, hold: 0.0 };
        a = tick(a, 1.0 / 60.0);
        assert!(a.alpha < 0.4);
        let dipped = a.alpha;
        a = trigger(a);
        a = tick(a, 1.0 / 60.0);
        assert!(a.alpha > dipped, "did not recover: {a:?}");
        assert!(a.alpha < 1.0, "snapped to full instead of fading");
    }

    #[test]
    fn slide_is_full_when_hidden_and_zero_when_lit() {
        assert!((slide_px(0.0) - SLIDE_PX).abs() < 1e-6);
        assert!(slide_px(1.0).abs() < 1e-6);
        // Monotonic, no overshoot past the edge (spec: no bouncing).
        let mut prev = slide_px(0.0);
        for i in 1..=20 {
            let s = slide_px(i as f32 / 20.0);
            assert!(s <= prev + 1e-6, "slide went backwards at {i}");
            assert!(s >= -1e-6, "slide overshot the edge: {s}");
            prev = s;
        }
    }

    #[test]
    fn a_weapon_swap_highlights_the_newly_active_gun_slot() {
        let old = sig();
        let new = Sig { active: 2, ..old };
        assert_eq!(changed_slot(old, new), Some(2));
    }

    #[test]
    fn toggling_the_shield_highlights_the_shield_slot() {
        let old = sig();
        let new = Sig { shield_up: true, ..old };
        assert_eq!(changed_slot(old, new), Some(3));
    }

    #[test]
    fn selecting_or_spending_a_grenade_highlights_the_grenade_slot() {
        let old = sig();
        assert_eq!(changed_slot(old, Sig { throw_sel: 2, ..old }), Some(4));
        assert_eq!(changed_slot(old, Sig { nades: [1, 1, 1, 0], ..old }), Some(4));
    }

    #[test]
    fn a_weapon_swap_wins_over_a_simultaneous_grenade_tick() {
        let old = sig();
        let new = Sig {
            active: 1,
            nades: [1, 1, 1, 0],
            ..old
        };
        assert_eq!(changed_slot(old, new), Some(1));
    }

    #[test]
    fn nothing_changing_highlights_nothing() {
        assert_eq!(changed_slot(sig(), sig()), None);
    }

    #[test]
    fn an_empty_gun_slot_is_dim_and_the_held_one_is_brightest() {
        let s = Sig {
            guns: [GunKind::M4, GunKind::Fists, GunKind::Awm],
            ..sig()
        };
        assert_eq!(tier_for(Slot::Primary, s), Tier::Active);
        assert_eq!(tier_for(Slot::Secondary, s), Tier::Unowned);
        assert_eq!(tier_for(Slot::Special, s), Tier::Owned);
        assert!(Tier::Unowned.alpha() < Tier::Owned.alpha());
        assert!(Tier::Owned.alpha() < Tier::Active.alpha());
        // Spec: unowned is ~50% and STILL VISIBLE.
        assert!(Tier::Unowned.alpha() >= 0.4 && Tier::Unowned.alpha() <= 0.6);
    }

    #[test]
    fn raising_the_shield_takes_the_active_read_off_the_gun() {
        let s = Sig { shield_up: true, ..sig() };
        assert_eq!(tier_for(Slot::Shield, s), Tier::Active);
        assert_eq!(tier_for(Slot::Primary, s), Tier::Owned);
    }

    #[test]
    fn an_exhausted_grenade_slot_is_dim() {
        let s = Sig { throw_sel: 3, ..sig() };
        assert_eq!(tier_for(Slot::Grenades, s), Tier::Unowned);
        assert_eq!(tier_for(Slot::Grenades, sig()), Tier::Owned);
    }

    #[test]
    fn slot_text_prints_ammo_stance_and_quantity() {
        let s = sig();
        let ammo = [(30, 120), (12, 60), (5, 15)];
        assert_eq!(slot_text(Slot::Primary, s, ammo), "30/120");
        assert_eq!(slot_text(Slot::Secondary, s, ammo), "12/60");
        assert_eq!(slot_text(Slot::Shield, s, ammo), "READY");
        assert_eq!(
            slot_text(Slot::Shield, Sig { shield_up: true, ..s }, ammo),
            "UP"
        );
        assert_eq!(slot_text(Slot::Grenades, s, ammo), "x2");
        let empty = Sig {
            guns: [GunKind::Fists, GunKind::Glock, GunKind::Awm],
            ..s
        };
        assert_eq!(slot_text(Slot::Primary, empty, ammo), "--");
    }

    #[test]
    fn every_slot_has_a_short_label_and_a_readable_icon() {
        for slot in Slot::ALL {
            let l = slot.label();
            assert!(!l.is_empty() && l.len() <= 4, "{l} is not a short label");
            let parts = icon_parts(slot);
            assert!(
                (2..=MAX_PARTS).contains(&parts.len()),
                "{l}: {} parts",
                parts.len()
            );
            for p in parts {
                assert!(
                    p.w >= MIN_STROKE && p.h >= MIN_STROKE,
                    "{l}: {}x{} smudges at {ICON}px",
                    p.w,
                    p.h
                );
                assert!(
                    p.x >= 0.0 && p.y >= 0.0 && p.x + p.w <= ICON_BOX && p.y + p.h <= ICON_BOX,
                    "{l}: part escapes the icon box"
                );
            }
        }
    }

    #[test]
    fn the_card_is_narrow_and_flush_to_the_right_edge() {
        assert!(EDGE_MARGIN <= 5.0, "spec asks for a 0-5px margin");
        // "Narrow" against the 1280-wide space the type ramp is
        // authored in, rather than against a bare literal: the card
        // legitimately grew from 66 to 84 to stop `30/120` printing off
        // the plate, and a test that fights a legibility fix is worse
        // than no test. A twelfth of the screen is still an edge dock.
        assert!(CARD_W <= 1280.0 / 12.0, "not a narrow card: {CARD_W}");
        // Taller than wide: it is a vertical strip, not a bar.
        assert!(SLOT_H * SLOTS as f32 > CARD_W * 2.0);
    }
}
