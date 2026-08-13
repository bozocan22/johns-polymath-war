//! §20 THE COCKPIT: what it is like to be INSIDE one of these machines.
//!
//! Before this, boarding a chassis raised your eye height and put a hull
//! mount in the lower right. Everything else about the frame was a
//! soldier's frame: an empty sky, an empty floor, and a HUD floating in
//! the middle of it. The pilot was a tall man with a big gun, not a
//! person sealed into a machine.
//!
//! This module builds the machine AROUND the camera - structure, lit
//! edges, instruments, screen glow and vibration - and it builds a
//! DIFFERENT one per chassis, because the note already written into
//! `mech_hud_sync` applies twice as hard here: the heavy and the medic
//! must not read as one cockpit with different numbers in it.
//!
//!   HEAVY   an armoured cab. Thick square pillars, a chamfered canopy,
//!           a deep console lip you look over, bolt heads, amber light.
//!           It is a box, and it feels small.
//!   MEDIC   a bubble canopy. Thin posts set right at the edge, a
//!           stepped arch instead of a beam, no console at all - just a
//!           slim dash bar floating low and left - and cool white light.
//!           It is open, and it feels light.
//!
//! # Why the geometry is declared in SCREEN space
//!
//! The one hard requirement on a first-person cockpit is that it must
//! not eat the middle of the frame: the crosshair, the charge marks and
//! `mech_hud_sync`'s target bracket all live there. That is a constraint
//! about SCREEN COVERAGE, so the panels are declared as screen-fraction
//! rectangles (`Panel`) and turned into local transforms by
//! `panel_transform` - rather than as metre offsets whose screen extent
//! nobody could check without launching the game.
//!
//! `centre_stays_clear` then reads the same tables the spawner reads and
//! asserts every panel clears the centre with room for the worst
//! vibration the shake function can produce. That is the one part of a
//! visual feature a unit test can honestly hold, so it holds it.
//!
//! Each `Panel` is a box whose declared rectangle is its FAR face and
//! which extrudes TOWARD the camera. That direction is deliberate: a box
//! extruded away from the eye projects its far face closer to the screen
//! centre than its near face, so the declared rect would stop bounding
//! it. Extruded forward, the declared rect is the innermost extent and
//! the test means what it says.
//!
//! # Cosmetic, and only cosmetic
//!
//! Nothing here is read by the sim, writes to the sim, or draws from the
//! sim's RNG. The vibration is wall-clock and per-client by design; the
//! gauges are pure functions of fighter state; the damage flicker is an
//! edge detector on a value that is already on screen.

use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::{sim, CamCtl, Game, GameState, ModelKit, COCKPIT_LAYER, VM_FOV_DEG};

/// The aspect the screen-space tables are authored against. The
/// viewmodel camera takes its real aspect from the window, so on a
/// non-16:9 window the pillars sit slightly in or out of the true edge -
/// which is the correct failure (a frame that pulls AWAY from the edge
/// on a narrow window is a gap; one that pulls in is still clear of the
/// centre by a wide margin).
pub const DESIGN_ASPECT: f32 = 16.0 / 9.0;

/// The half-extent of the box at screen centre that no cockpit part may
/// enter, in the same -1..1 screen fractions the panels use. 0.40 is the
/// central 40% of the screen in each axis.
pub const CENTRE_SAFE_U: f32 = 0.40;
pub const CENTRE_SAFE_V: f32 = 0.40;

/// The largest translation `cockpit_shake` may ever return, in metres.
/// `centre_stays_clear` converts this into screen fractions and demands
/// every panel clears the safe box by at least that much, so a shake can
/// never push structure over the crosshair.
pub const MAX_SHAKE_M: f32 = 0.016;

/// The largest the pitch lean may lift or drop the shell, in metres.
/// Vertical only, so it widens the centre-clearance margin in v alone.
pub const MAX_LEAN_M: f32 = 0.034;

/// Where the shell sits. Nearer than every hull-mount viewmodel
/// (0.46-0.62 m) on purpose: the mounts hang OUTSIDE the machine, so the
/// canopy has to be able to occlude them, not the other way round.
const Z_SHELL: f32 = 0.44;
const Z_EDGE: f32 = 0.415;

/// How DEEP a lit trim strip is, and the number that made the whole cab
/// read as one flat orange.
///
/// A `Panel` is a BOX declared at its far face and extruded toward the
/// eye, so its near face projects `z / (z - d)` times further from the
/// screen centre than its far face - and the side wall between them is
/// drawn too. The trim strips run at |u| ~ 0.86-0.93, right out at the
/// screen edge, where that magnification bites hardest.
///
/// At the old `d = 0.02` the pillar's 0.020-wide accent line covered
/// 0.064 of the screen - THREE TIMES its authored width, 51 px of the
/// most saturated colour in the palette, and four of them ringing the
/// view in a closed rectangle. Measured off
/// `handback/brief-vii/cockpit/01-cockpit-level.png`, scanline y=300:
/// a solid `(0.803, 0.502, 0.207)` run from x=77 to x=127.
///
/// This is the same failure the palette note below describes - "you
/// could not tell the lamp from the wall" - arriving through geometry
/// rather than through colour, which is why fixing the palette did not
/// fix it. `no_lit_trim_renders_far_wider_than_it_was_authored` holds
/// the bound now.
///
/// 4 mm and not zero: a zero-depth box has no thickness to catch the
/// canopy lamp, and these are meant to be a seam in a machine rather
/// than a decal on it.
const EDGE_D: f32 = 0.004;

/// The most a lit trim strip may cover, as a multiple of the width it
/// was authored with. See `EDGE_D` - the old geometry ran at 3.2x.
pub const EDGE_MAX_SPREAD: f32 = 1.7;

/// How hard the lit trim burns, against a gauge segment reading full.
///
/// Not 1.0, and that is the other half of "tell the lamp from the wall".
/// The trim and the gauges were the SAME material value, so a seam in
/// the structure and a readout at maximum were the same colour on the
/// screen - and there is far more seam than readout. A trim line is
/// evidence that the frame is powered; a gauge is a number. The number
/// gets the top of the range.
///
/// `lit_trim_never_burns_as_hard_as_a_gauge` is the test.
const EDGE_LEVEL: f32 = 0.5;

/// What a gauge with NOTHING TO SAY burns at.
///
/// `EDGE_LEVEL` put a seam below a readout and that was right, but it
/// left the readouts themselves at the top of the range permanently -
/// and a gauge is at the top of the range nearly all the time, because
/// most of the time your hull is fine, your power is fine and your
/// barrier is full. The barrier bar is the worst case: ten cells of the
/// most saturated colour in the palette, in a row, along the bottom of
/// the frame directly under the crosshair. Measured off
/// `handback/brief-vii/cockpit/01-cockpit-level.png`, it was the single
/// loudest thing in the cab.
///
/// So the level is now a FUNCTION OF THE ALARM (`gauge_level`): a calm
/// gauge sits here, an alarming one climbs back to 1.0. That is not just
/// a dimmer. Before this, a gauge at full and a gauge screaming were the
/// same VALUE and differed only in hue - which is the exact failure
/// `ALARM_TINT`'s own note records ("an alarm that reads as the amber
/// one, but slightly more orange is not an alarm"), surviving in the one
/// channel that note did not cover. An alarm now gains value as well as
/// hue, and the quiet state pays for it.
///
/// Not lower, and the number is pinned from both sides:
/// `lit_trim_never_burns_as_hard_as_a_gauge` demands the trim stay
/// clearly under a CALM gauge (0.5 vs 0.75), and
/// `a_calm_gauge_is_still_lit_instrumentation` demands a calm gauge stay
/// clearly over the trim. A gauge dimmed into its own backing is not a
/// quieter cockpit, it is a broken one.
const GAUGE_CALM: f32 = 0.75;

/// The most lit ink a gauge mounted on the CANOPY may cover, as a
/// fraction of the screen.
///
/// Canopy-mounted only, and that asymmetry is the whole point. The
/// instrument stack is outboard-left at |u| ~ 0.6-0.9, in peripheral
/// vision, where a big readout costs nothing; the barrier bar is sunk
/// into the console lip at the bottom CENTRE, which is where the eye
/// already is because that is where the crosshair is. The same area of
/// amber is not the same amount of loud in those two places.
///
/// `a_canopy_gauge_stays_out_of_the_sight_line` is the test, and it
/// fails on the geometry this file shipped last pass: ten 0.050 x 0.060
/// cells came to 0.43% of the screen against this 0.30% cap.
pub const CANOPY_GAUGE_INK_MAX: f32 = 0.0030;

/// THE INSTRUMENT STACK, front to back. Five layers, each one thin, and
/// each one's NEAR face behind the next one's FAR face.
///
/// The first cut got this wrong and it cost a capture: the bezel was
/// declared at 0.385 but 0.05 m DEEP, which put its near face at 0.335 -
/// in front of gauges sitting at 0.37. Two of the three ladders were
/// perfectly correct and completely invisible, buried inside the box
/// that was supposed to be their surround. Nothing about the code looked
/// wrong; you could only see it in a photograph.
///
/// `panel_stack_layers_do_not_bury_each_other` now holds the ordering.
const Z_BEZEL: f32 = 0.410;
const Z_BEZEL_D: f32 = 0.020;
const Z_FASCIA: f32 = 0.388;
const Z_FASCIA_D: f32 = 0.012;
const Z_RUNG: f32 = 0.374;
const Z_RUNG_D: f32 = 0.008;
const Z_GAUGE: f32 = 0.364;
const Z_GAUGE_D: f32 = 0.006;
const Z_SCAN: f32 = 0.356;

/// The two chassis tints, as sRGB 0..1. Amber for the heavy, cool
/// white-blue for the medic - the same pair `mech_hud_sync` tints its
/// housing frame with, so the flat overlay and the 3D shell agree about
/// which machine you are in.
///
/// BASE COLOUR, not emissive, and that is not a style choice.
/// `StandardMaterial { unlit: true }` skips `apply_pbr_lighting`
/// entirely, and `emissive` is added INSIDE that function - so on an
/// unlit material the emissive channel is simply not read. The first
/// cut of this module set `unlit: true` and then drove `emissive` every
/// frame: the flicker, the alarm ramp, the gauge pulse and both warning
/// lamps were all computing correctly and none of them reached a pixel.
/// The panel photographed as flat brown paint. (The rest of the game's
/// glow materials set both, which is why nobody had hit it: they are
/// authored with the base colour already bright.)
///
/// There is no bloom and no HDR on these cameras, so there would be no
/// headroom above 1.0 to spend anyway.
const HEAVY_TINT: [f32; 3] = [1.00, 0.58, 0.14];
const MEDIC_TINT: [f32; 3] = [0.42, 0.86, 1.00];
/// What a gauge burns when the thing it measures is in trouble. Red,
/// and nothing like either chassis tint - an alarm that reads as "the
/// amber one, but slightly more orange" is not an alarm, and the first
/// pass shipped exactly that: at 16% hull the ladder photographed the
/// same brown as the two beside it reading full.
const ALARM_TINT: [f32; 3] = [1.00, 0.17, 0.10];
/// A lamp with nothing to say. Not black - a dead black rectangle reads
/// as a hole in the console rather than a bulb that is not lit.
const LAMP_OFF: [f32; 3] = [0.075, 0.072, 0.068];
const LAMP_CAUTION: [f32; 3] = [1.00, 0.14, 0.08];
const LAMP_VENT: [f32; 3] = [1.00, 0.66, 0.06];

fn srgb(c: [f32; 3], k: f32) -> Color {
    Color::srgb(
        (c[0] * k).clamp(0.0, 1.0),
        (c[1] * k).clamp(0.0, 1.0),
        (c[2] * k).clamp(0.0, 1.0),
    )
}

// ---------------------------------------------------------------------
// tables
// ---------------------------------------------------------------------

/// What a piece of the shell is made of. Each role is one material per
/// chassis, so a chassis is a palette swap over a layout rather than a
/// second copy of a material list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    /// Load-bearing structure - pillars, beams, the console body.
    Frame,
    /// Shadowed recesses and the backs of things.
    Deep,
    /// A lit accent line in the chassis tint. Thin, and always proud of
    /// the frame so it never z-fights with it.
    Edge,
    /// Near-black instrument backing. The thing gauges are printed on.
    Fascia,
    /// A bolt head or a rivet. Heavy only - the medic is welded.
    Stud,
}

/// One box of the cockpit shell, in screen fractions (-1..1, v up).
#[derive(Clone, Copy, Debug)]
pub struct Panel {
    pub u0: f32,
    pub u1: f32,
    pub v0: f32,
    pub v1: f32,
    /// Depth of the FAR face, metres ahead of the eye.
    pub z: f32,
    /// How far the box extrudes toward the camera from that face.
    pub d: f32,
    pub role: Role,
    /// Which of the two groups this part belongs to - see `Group`.
    pub inst: bool,
}

/// THE COCKPIT SURVIVES THE V TOGGLE, and it does it by being two things.
///
///   CANOPY     the physical shell you are sitting inside: pillars, arch,
///              console lip. First person only. In third person you are
///              looking at the machine from outside it, and a canopy rib
///              across that view is not a cockpit, it is a bug.
///   INSTRUMENT the readouts. Shown in BOTH views, because the interface
///              is the cockpit: a pilot who presses V has not stopped
///              flying the machine and must not lose its gauges.
///
/// A `bool` rather than an enum only because there are exactly two and
/// the field reads as a sentence at the call sites (`pi` = panel,
/// instrument; `pn` = panel, structure).
const fn pn(u0: f32, u1: f32, v0: f32, v1: f32, z: f32, d: f32, role: Role) -> Panel {
    Panel { u0, u1, v0, v1, z, d, role, inst: false }
}

const fn pi(u0: f32, u1: f32, v0: f32, v1: f32, z: f32, d: f32, role: Role) -> Panel {
    Panel { u0, u1, v0, v1, z, d, role, inst: true }
}

/// THE ARMOURED CAB. Square, thick, and closed: a full-width beam over
/// your head, pillars you could not get an arm round, and a console lip
/// deep enough that you look OVER it rather than through it.
///
/// The panels run slightly past +-1.02 so that a window a hair wider
/// than 16:9 still shows structure at the very edge instead of a seam.
pub const HEAVY_SHELL: &[Panel] = &[
    // --- pillars -----------------------------------------------------
    pn(-1.02, -0.86, -1.02, 0.80, Z_SHELL, 0.10, Role::Frame),
    pn(0.86, 1.02, -1.02, 0.80, Z_SHELL, 0.10, Role::Frame),
    // the shadowed inboard face of each pillar, set back a little
    pn(-0.88, -0.84, -1.02, 0.80, Z_SHELL + 0.03, 0.03, Role::Deep),
    pn(0.84, 0.88, -1.02, 0.80, Z_SHELL + 0.03, 0.03, Role::Deep),
    // --- canopy: a beam, then two chamfer steps into each pillar ------
    pn(-1.02, 1.02, 0.88, 1.02, Z_SHELL + 0.02, 0.10, Role::Frame),
    pn(-1.02, -0.68, 0.80, 0.88, Z_SHELL + 0.02, 0.08, Role::Frame),
    pn(-0.86, -0.72, 0.72, 0.80, Z_SHELL + 0.02, 0.08, Role::Frame),
    pn(0.68, 1.02, 0.80, 0.88, Z_SHELL + 0.02, 0.08, Role::Frame),
    pn(0.72, 0.86, 0.72, 0.80, Z_SHELL + 0.02, 0.08, Role::Frame),
    // --- console: a lip across the bottom, risers at both ends --------
    pn(-1.02, 1.02, -1.02, -0.84, Z_SHELL, 0.12, Role::Frame),
    pn(-1.02, -0.56, -0.84, -0.74, Z_SHELL, 0.12, Role::Frame),
    pn(0.56, 1.02, -0.84, -0.74, Z_SHELL, 0.12, Role::Frame),
    // the recess the barrier ladder is sunk into. Sits PROUD of the
    // console's top face: at these v the lip's own top is only ~0.38 m
    // out, so a recess declared any deeper would be swallowed by the
    // console it is supposed to be cut into.
    //
    // Tracks the bar. It was 0.02..0.70 x -0.98..-0.87 around a bar that
    // filled it; the bar is a third of the height now, and a socket with
    // 25 px of black margin round a 12 px readout reads as a slot with
    // something missing from it rather than as an instrument. Snug: a
    // couple of millimetres of surround on every side.
    pn(0.03, 0.675, -0.958, -0.895, 0.380, 0.010, Role::Fascia),
    // --- lit edges: the amber lines that say the frame is powered -----
    // `EDGE_D` deep, not 0.02 - see its note. These four ring the whole
    // view, so any width they gain to parallax is multiplied by four.
    pn(-1.02, 1.02, -0.848, -0.822, Z_EDGE, EDGE_D, Role::Edge),
    pn(-0.862, -0.842, -0.84, 0.80, Z_EDGE, EDGE_D, Role::Edge),
    pn(0.842, 0.862, -0.84, 0.80, Z_EDGE, EDGE_D, Role::Edge),
    pn(-1.02, 1.02, 0.872, 0.894, Z_EDGE + 0.01, EDGE_D, Role::Edge),
    // --- bolt heads down the pillars ----------------------------------
    pn(-0.985, -0.945, -0.66, -0.62, Z_EDGE, 0.03, Role::Stud),
    pn(-0.985, -0.945, -0.28, -0.24, Z_EDGE, 0.03, Role::Stud),
    pn(-0.985, -0.945, 0.10, 0.14, Z_EDGE, 0.03, Role::Stud),
    pn(-0.985, -0.945, 0.48, 0.52, Z_EDGE, 0.03, Role::Stud),
    pn(0.945, 0.985, -0.66, -0.62, Z_EDGE, 0.03, Role::Stud),
    pn(0.945, 0.985, -0.28, -0.24, Z_EDGE, 0.03, Role::Stud),
    pn(0.945, 0.985, 0.10, 0.14, Z_EDGE, 0.03, Role::Stud),
    pn(0.945, 0.985, 0.48, 0.52, Z_EDGE, 0.03, Role::Stud),
    // --- the instrument stack, bolted INBOARD of the left pillar -------
    // Set proud of the frame so it reads as a fitted unit rather than a
    // decal, and placed in the one large region of this HUD that is
    // empty at every resolution: outboard left, above the vitals line.
    //
    // Inboard of u=-0.86 deliberately. The pillar is a 10 cm deep box,
    // so its INNER SIDE FACE sweeps forward from 0.44 m to 0.34 m as it
    // recedes - and anything sharing that screen space at a greater
    // depth is behind a wall. The first placement ran to -1.02 and the
    // pillar ate the outboard third of the panel.
    pi(-0.88, -0.54, -0.52, 0.22, Z_BEZEL, Z_BEZEL_D, Role::Frame),
    pi(-0.86, -0.56, -0.48, 0.18, Z_FASCIA, Z_FASCIA_D, Role::Fascia),
];

/// THE BUBBLE CANOPY. Everything the cab is not: posts instead of
/// pillars, a stepped arch instead of a beam, no console lip at all, and
/// a dash bar that floats rather than being built into a wall. The medic
/// is a machine you can see out of.
pub const MEDIC_SHELL: &[Panel] = &[
    // --- two thin posts, hard against the edge ------------------------
    pn(-1.02, -0.93, -1.02, 0.66, Z_SHELL, 0.08, Role::Frame),
    pn(0.93, 1.02, -1.02, 0.66, Z_SHELL, 0.08, Role::Frame),
    // --- the arch: three steps a side, so it reads as a curve ---------
    pn(-1.02, -0.86, 0.66, 0.78, Z_SHELL + 0.02, 0.06, Role::Frame),
    pn(-0.90, -0.62, 0.78, 0.88, Z_SHELL + 0.02, 0.06, Role::Frame),
    pn(-0.66, 0.66, 0.88, 1.02, Z_SHELL + 0.02, 0.06, Role::Frame),
    pn(0.62, 0.90, 0.78, 0.88, Z_SHELL + 0.02, 0.06, Role::Frame),
    pn(0.86, 1.02, 0.66, 0.78, Z_SHELL + 0.02, 0.06, Role::Frame),
    // --- two cross-braces per post: an OPEN frame, visibly braced -----
    pn(-0.99, -0.80, 0.20, 0.245, Z_SHELL - 0.01, 0.04, Role::Deep),
    pn(-0.99, -0.80, -0.34, -0.295, Z_SHELL - 0.01, 0.04, Role::Deep),
    pn(0.80, 0.99, 0.20, 0.245, Z_SHELL - 0.01, 0.04, Role::Deep),
    pn(0.80, 0.99, -0.34, -0.295, Z_SHELL - 0.01, 0.04, Role::Deep),
    // --- cool-white light down the inboard edge of everything ---------
    pn(-0.928, -0.912, -1.02, 0.66, Z_EDGE, EDGE_D, Role::Edge),
    pn(0.912, 0.928, -1.02, 0.66, Z_EDGE, EDGE_D, Role::Edge),
    pn(-1.02, -0.86, 0.652, 0.668, Z_EDGE + 0.01, EDGE_D, Role::Edge),
    pn(-0.90, -0.62, 0.772, 0.788, Z_EDGE + 0.01, EDGE_D, Role::Edge),
    pn(-0.66, 0.66, 0.872, 0.888, Z_EDGE + 0.01, EDGE_D, Role::Edge),
    pn(0.62, 0.90, 0.772, 0.788, Z_EDGE + 0.01, EDGE_D, Role::Edge),
    pn(0.86, 1.02, 0.652, 0.668, Z_EDGE + 0.01, EDGE_D, Role::Edge),
    // --- the dash bar: a slab hung low and left, not a wall -----------
    // All three are INSTRUMENT parts: the medic's dash is not structure,
    // it is the readout, and it stays through the V toggle.
    pi(-0.94, -0.16, -1.02, -0.85, Z_BEZEL, Z_BEZEL_D, Role::Frame),
    pi(-0.92, -0.18, -1.00, -0.865, Z_FASCIA, Z_FASCIA_D, Role::Fascia),
    pi(-0.94, -0.16, -0.862, -0.846, Z_EDGE, EDGE_D, Role::Edge),
];

/// Which live value a ladder shows.
///
/// Every one of these is an ANALOG read of a field the sim already
/// owns - no strings, no numbers, no rounding. See the module note in
/// `spawn_cockpit` for why that distinction matters here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gauge {
    Hull,
    /// The heavy's power core (`Fighter::armor`, 0..POWER_MAX).
    Power,
    /// Hull-mount heat. NOTE the two chassis store this in ONE field at
    /// TWO scales - see `heat_frac`.
    Heat,
    /// The mech barrier pool.
    Barrier,
    /// The medic's precision-shot wind-up.
    Charge,
}

/// A ladder of `n` lit segments stepping from an origin. `du`/`dv` set
/// the direction: a vertical column steps in v, a horizontal bar in u.
#[derive(Clone, Copy, Debug)]
pub struct Ladder {
    pub id: Gauge,
    pub u: f32,
    pub v: f32,
    pub su: f32,
    pub sv: f32,
    pub du: f32,
    pub dv: f32,
    pub n: u8,
    /// Part of the INSTRUMENT group (survives the V toggle) rather than
    /// the canopy. See `pn`/`pi`.
    pub inst: bool,
}

/// How far the LIT face is inset inside its dark rung, per axis, as a
/// fraction of the rung. The rung is the socket and the lit face is the
/// bulb in it, so the bulb has to be visibly smaller than the hole.
///
/// Named rather than left as literals in the spawn because `ladder_ink`
/// has to compute the same rectangle from a test, and two copies of an
/// inset is exactly how a bound stops bounding what is drawn.
const SEG_INSET_U: f32 = 0.10;
const SEG_INSET_V: f32 = 0.14;

impl Ladder {
    /// The screen rectangle the whole ladder occupies.
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let last = (self.n.max(1) - 1) as f32;
        (
            self.u,
            self.u + self.du * last + self.su,
            self.v,
            self.v + self.dv * last + self.sv,
        )
    }

    /// The fraction of the SCREEN this ladder's lit faces cover with
    /// every segment burning - the gauge's worst case, and the state
    /// most of them are in most of the time.
    ///
    /// The u/v tables run -1..1 in both axes, so the screen is 2x2 = 4
    /// units of area and a rectangle's share of it is `w * h / 4`. Only
    /// the LIT faces count: the dark rungs behind them are near-black
    /// and are what makes an empty cell read as a socket rather than a
    /// hole, so they are not what anybody means by "too orange".
    pub fn ink(&self) -> f32 {
        let w = self.su * (1.0 - 2.0 * SEG_INSET_U);
        let h = self.sv * (1.0 - 2.0 * SEG_INSET_V);
        (w * h * self.n as f32 / 4.0).abs()
    }
}

/// The heavy's three-column instrument stack, plus the barrier ladder
/// sunk into the console lip where a pilot's hand would be.
pub const HEAVY_LADDERS: &[Ladder] = &[
    Ladder { id: Gauge::Hull, u: -0.840, v: -0.34, su: 0.068, sv: 0.044, du: 0.0, dv: 0.062, n: 8, inst: true },
    Ladder { id: Gauge::Power, u: -0.735, v: -0.34, su: 0.068, sv: 0.044, du: 0.0, dv: 0.062, n: 8, inst: true },
    Ladder { id: Gauge::Heat, u: -0.630, v: -0.34, su: 0.068, sv: 0.044, du: 0.0, dv: 0.062, n: 8, inst: true },
    // The BARRIER bar is sunk into the console lip, so it belongs to the
    // canopy: a strip of lit segments floating at the bottom of a
    // third-person frame with no console under it reads as debris. Its
    // pool is still on screen in both views - `shield_readout` carries
    // it on the right edge, which is where §18 put it.
    //
    // 0.042 x 0.038 CELLS, and they were 0.050 x 0.060. At the old size
    // this was ten 32x19 px blocks of full-tint amber in a row 480 px
    // long, directly under the crosshair, and the capture is not
    // ambiguous about the result - it is a row of orange bricks and it
    // is the first thing you see in the frame. A gauge that dominates
    // the view is not more readable for it; a pilot reads a bar by HOW
    // FAR ALONG IT the light stops, and that reading needs length, which
    // this keeps in full, not height, which it does not. Same ten cells,
    // same pitch, same position: a scale instead of a slab. See
    // `CANOPY_GAUGE_INK_MAX`.
    Ladder { id: Gauge::Barrier, u: 0.05, v: -0.945, su: 0.042, sv: 0.038, du: 0.062, dv: 0.0, n: 10, inst: false },
];

/// The medic's dash: three short rows, read left to right. A different
/// shape from the heavy's columns on purpose - a pilot swapping machines
/// should not be able to read the new one out of habit.
pub const MEDIC_LADDERS: &[Ladder] = &[
    Ladder { id: Gauge::Hull, u: -0.885, v: -0.975, su: 0.055, sv: 0.026, du: 0.064, dv: 0.0, n: 10, inst: true },
    Ladder { id: Gauge::Heat, u: -0.885, v: -0.937, su: 0.055, sv: 0.026, du: 0.064, dv: 0.0, n: 10, inst: true },
    Ladder { id: Gauge::Charge, u: -0.885, v: -0.899, su: 0.055, sv: 0.026, du: 0.064, dv: 0.0, n: 10, inst: true },
];

/// A warning lamp. Two only, and both are THRESHOLDS on a gauge already
/// on the panel beside them - a lamp that carried a fact of its own
/// would be a third place for that fact to live.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lamp {
    /// Hull below a quarter.
    Caution,
    /// The mount is venting and will not fire.
    Vent,
}

#[derive(Clone, Copy, Debug)]
pub struct LampSpec {
    pub id: Lamp,
    pub u0: f32,
    pub u1: f32,
    pub v0: f32,
    pub v1: f32,
}

/// On the instrument fascia, under the ladders - NOT on the console lip
/// where they started. Warning lamps are instrumentation, and putting
/// them on the canopy would have meant losing both of them the moment
/// the pilot pressed V.
pub const HEAVY_LAMPS: &[LampSpec] = &[
    LampSpec { id: Lamp::Caution, u0: -0.830, u1: -0.755, v0: -0.455, v1: -0.385 },
    LampSpec { id: Lamp::Vent, u0: -0.715, u1: -0.640, v0: -0.455, v1: -0.385 },
];

pub const MEDIC_LAMPS: &[LampSpec] = &[
    LampSpec { id: Lamp::Caution, u0: -0.145, u1: -0.085, v0: -0.975, v1: -0.925 },
    LampSpec { id: Lamp::Vent, u0: -0.145, u1: -0.085, v0: -0.910, v1: -0.860 },
];

// ---------------------------------------------------------------------
// screen space -> local space
// ---------------------------------------------------------------------

/// Half the visible extent of the viewmodel camera at distance `z`.
pub fn half_extent(z: f32, aspect: f32) -> (f32, f32) {
    let hh = z * (VM_FOV_DEG.to_radians() * 0.5).tan();
    (hh * aspect, hh)
}

/// Turn a screen-fraction rectangle at depth `z` into a box transform.
/// The rectangle is the FAR face; the box grows toward the camera.
pub fn rect_transform(u0: f32, u1: f32, v0: f32, v1: f32, z: f32, d: f32) -> Transform {
    let (hw, hh) = half_extent(z, DESIGN_ASPECT);
    Transform {
        translation: Vec3::new(
            (u0 + u1) * 0.5 * hw,
            (v0 + v1) * 0.5 * hh,
            -(z - d * 0.5),
        ),
        rotation: Quat::IDENTITY,
        scale: Vec3::new(
            ((u1 - u0) * hw).abs().max(1e-4),
            ((v1 - v0) * hh).abs().max(1e-4),
            d.max(1e-4),
        ),
    }
}

fn panel_transform(p: &Panel) -> Transform {
    rect_transform(p.u0, p.u1, p.v0, p.v1, p.z, p.d)
}

/// The screen-fraction rectangle a panel actually COVERS - which is not
/// the rectangle it was authored with, and that gap is what made the
/// heavy cab read as one flat orange.
///
/// A panel's world extent is fixed at its FAR face. Its NEAR face sits
/// `d` closer to the eye, so under a perspective camera that face
/// projects `z / (z - d)` times further from the screen centre, and the
/// side wall joining the two is drawn as well. The silhouette is
/// therefore the convex hull of both faces, and for a strip out at the
/// screen edge the hull can be several times the declared width.
///
/// Pure, and takes only the declared numbers, so the property can be
/// asserted over the whole table without launching the game - which is
/// the only reason this defect is now catchable. It was invisible in the
/// source (`0.02` looks like a hairline) and only measurable in a PNG.
pub fn panel_silhouette(p: &Panel) -> (f32, f32, f32, f32) {
    // guard the degenerate case rather than dividing by ~0: a panel as
    // deep as it is distant is a bug the extrude test already catches.
    let near = (p.z - p.d).max(1e-3);
    let m = p.z / near;
    let hull = |a: f32, b: f32| (a.min(a * m).min(b.min(b * m)), a.max(a * m).max(b.max(b * m)));
    let (u0, u1) = hull(p.u0, p.u1);
    let (v0, v1) = hull(p.v0, p.v1);
    (u0, u1, v0, v1)
}

/// How much wider than authored a panel renders, per axis. 1.0 is a
/// panel that covers exactly what it declared.
pub fn panel_spread(p: &Panel) -> (f32, f32) {
    let (u0, u1, v0, v1) = panel_silhouette(p);
    (
        (u1 - u0) / (p.u1 - p.u0).abs().max(1e-4),
        (v1 - v0) / (p.v1 - p.v0).abs().max(1e-4),
    )
}

/// The lit trim's colour at a given flicker level.
///
/// ONE function, called from both the spawn and `cockpit_sync`, because
/// those two have to agree about the trim and a bare literal in each is
/// exactly how they would stop agreeing - the spawn value would be the
/// one a still capture photographs and the sync value the one the game
/// actually runs at, which is a difference no screenshot could ever
/// show.
pub fn edge_color(tint: [f32; 3], stutter: f32) -> Color {
    srgb(tint, stutter * EDGE_LEVEL)
}

/// How hard a gauge burns, given how alarming it is. `GAUGE_CALM` when
/// it has nothing to say, 1.0 when it is screaming.
///
/// Linear between the two on purpose: `gauge_alarm` already shapes the
/// ramp per gauge (hull reads down, heat reads up), and putting a second
/// curve on top of it would mean two places decide what "half alarming"
/// looks like.
pub fn gauge_level(alarm: f32) -> f32 {
    let a = alarm.clamp(0.0, 1.0);
    GAUGE_CALM + (1.0 - GAUGE_CALM) * a
}

/// THE colour of a gauge segment: the chassis tint, mixed toward
/// `ALARM_TINT` by `alarm`, at `gauge_level(alarm)`, times `flicker`.
///
/// One function, called from both the spawn and `cockpit_sync`, for the
/// reason `edge_color` gives above and one more: the alarm MIX used to
/// live inline in the system, which meant the single most important
/// colour decision in this module - what a gauge in trouble looks like -
/// was unreachable from a test. It is three lines of arithmetic that
/// nothing could call. Now `an_alarming_gauge_outburns_a_calm_one` can.
pub fn gauge_color(tint: [f32; 3], alarm: f32, flicker: f32) -> Color {
    let k = alarm.clamp(0.0, 1.0);
    let mix = [
        tint[0] * (1.0 - k) + ALARM_TINT[0] * k,
        tint[1] * (1.0 - k) + ALARM_TINT[1] * k,
        tint[2] * (1.0 - k) + ALARM_TINT[2] * k,
    ];
    srgb(mix, flicker * gauge_level(k))
}

/// `MAX_SHAKE_M` expressed in the screen fractions the tables use, at
/// the depth the shell sits. The shake moves the whole rig, so this is
/// how far a panel edge can travel toward the centre.
pub fn max_shake_u() -> f32 {
    let (hw, _) = half_extent(Z_SHELL, DESIGN_ASPECT);
    MAX_SHAKE_M / hw
}

/// Worst-case vertical travel: the shake and the pitch lean can peak
/// together, and the lean is far the larger of the two.
pub fn max_shake_v() -> f32 {
    let (_, hh) = half_extent(Z_SHELL, DESIGN_ASPECT);
    (MAX_SHAKE_M + MAX_LEAN_M) / hh
}

// ---------------------------------------------------------------------
// live values
// ---------------------------------------------------------------------

/// The handful of live values a cockpit reads, lifted out of the
/// fighter.
///
/// Extracted so the arithmetic is reachable from a test. `sim::Fighter`
/// has ~90 fields and no `Default`, so a helper that took one could only
/// ever be exercised by launching the game - and the last time a piece
/// of view arithmetic hid inside a Bevy system in this codebase, a 47
/// degree camera error survived for months behind it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vitals {
    pub hull: f32,
    pub hull_max: f32,
    pub power: f32,
    /// ALREADY normalised to 0..1 - see `heat_frac`.
    pub heat: f32,
    pub barrier: f32,
    pub charge: f32,
}

/// Mount heat as a 0..1 fraction.
///
/// `Fighter::gatling_heat` is ONE field carrying TWO scales: the heavy's
/// hull gatling runs it 0..100 (`GATLING_HEAT_PER_SHOT` is 0.9 against a
/// 100.0 cap) and the medic's plasma mounts run it 0..1. That is not
/// this module's to fix - it is in the sim - but every reader has to
/// know, and at least one existing HUD line does not: the heavy's
/// bottom-right corner prints `gatling_heat` raw under a `%` sign, which
/// is right for the heavy and would be 100x wrong for the medic if that
/// branch were ever shared.
pub fn heat_frac(gatling_heat: f32, scout: bool) -> f32 {
    let max = if scout { 1.0 } else { 100.0 };
    (gatling_heat / max).clamp(0.0, 1.0)
}

pub fn vitals_of(f: &sim::Fighter) -> Vitals {
    Vitals {
        hull: f.hull,
        hull_max: f.mech_hull_max(),
        power: f.armor / sim::POWER_MAX,
        heat: heat_frac(f.gatling_heat, f.in_scout_mech()),
        barrier: f.mech_shield_hp / sim::MECH_SHIELD_HP,
        charge: f.plasma_charge_t / sim::PLASMA_CHARGE_S,
    }
}

/// What fraction of a ladder is lit, 0..1.
pub fn gauge_frac(v: &Vitals, id: Gauge) -> f32 {
    let x = match id {
        Gauge::Hull => v.hull / v.hull_max.max(1.0),
        Gauge::Power => v.power,
        Gauge::Heat => v.heat,
        Gauge::Barrier => v.barrier,
        Gauge::Charge => v.charge,
    };
    x.clamp(0.0, 1.0)
}

/// How many of `n` segments a fraction lights.
///
/// Ceiling, not rounding, with an explicit zero: a gauge that shows an
/// empty ladder while the pool still has something in it is the one
/// reading a pilot must never get. The last segment goes dark only when
/// the value is genuinely gone.
pub fn lit_segments(frac: f32, n: u8) -> u8 {
    if frac <= 0.0 {
        return 0;
    }
    ((frac.clamp(0.0, 1.0) * n as f32).ceil() as u8).clamp(1, n)
}

/// The colour a gauge burns at, given its fill. Returns linear RGB
/// multipliers applied to the chassis tint.
///
/// HULL and POWER read DOWN (full is calm, empty is alarming); HEAT
/// reads UP. They therefore cannot share a ramp, and getting that
/// backwards on one of them is the kind of thing nobody notices until a
/// pilot vents a mount believing he was topped up.
pub fn gauge_alarm(id: Gauge, frac: f32) -> f32 {
    match id {
        Gauge::Hull | Gauge::Power | Gauge::Barrier => (1.0 - frac * 2.2).clamp(0.0, 1.0),
        Gauge::Heat => ((frac - 0.55) / 0.45).clamp(0.0, 1.0),
        Gauge::Charge => 0.0,
    }
}

// ---------------------------------------------------------------------
// vibration
// ---------------------------------------------------------------------

/// The machine's own tremor, as an offset for the cockpit shell only.
///
/// This deliberately never touches the camera. The camera's transform
/// feeds `crosshair_aim_dir`, so shaking it would be shaking the AIM -
/// a cosmetic system writing into something the sim reads. Moving the
/// shell instead is also the better read: a rock-steady view with a
/// frame juddering around it says "the machine is vibrating", where a
/// shaking view says "the cameraman is running".
///
/// Layers, in order of how loud they are:
///   idle    a constant high-frequency hum - the reactor never stops
///   stride  footfall spikes while the chassis is actually walking
///   fire    a fast rattle while a mount is cycling
///   burst   the power stride, which is a whole-body shove
///   hit     a short hard jolt when the hull takes damage
///
/// The heavy is slow and heavy; the medic is fast and light, at roughly
/// half the amplitude and twice the rate. Returns (offset m, roll rad),
/// bounded by `MAX_SHAKE_M` (asserted in the tests).
pub fn cockpit_shake(
    t: f32,
    heavy: bool,
    walk: f32,
    fire: f32,
    burst: f32,
    hit: f32,
) -> (Vec3, f32) {
    let (rate, amp) = if heavy { (1.0, 1.0) } else { (1.85, 0.5) };
    let walk = walk.clamp(0.0, 1.0);
    let fire = fire.clamp(0.0, 1.0);
    let burst = burst.clamp(0.0, 1.0);
    let hit = hit.clamp(0.0, 1.0);

    // idle hum: two prime-ish rates so it never settles into a beat
    let mut y = (t * 31.0 * rate).sin() * 0.00055;
    let mut x = (t * 17.3 * rate).sin() * 0.00040;

    // footfall. `.powi(8)` on a sine is a SPIKE, not a wobble - a walker
    // this size does not bounce, it lands.
    let step = (t * 4.6 * rate).sin().abs().powi(8);
    y -= step * 0.0058 * walk;
    x += (t * 2.3 * rate).sin() * 0.0012 * walk;

    // the mount cycling
    y += (t * 47.0 * rate).sin() * 0.0030 * fire;
    x += (t * 61.0 * rate).cos() * 0.0018 * fire;

    // the power stride: low, heavy, and it shoves you back in the seat
    y += (t * 12.0).sin() * 0.0040 * burst;

    // the jolt
    y += (t * 83.0).sin() * 0.0055 * hit;
    x += (t * 71.0).cos() * 0.0040 * hit;

    let roll = (t * 3.7 * rate).sin() * 0.0022 * walk
        + (t * 41.0 * rate).sin() * 0.0030 * fire
        + (t * 67.0).sin() * 0.0060 * hit;

    let o = Vec3::new(x * amp, y * amp, 0.0);
    // A hard clamp rather than a hope: the centre-clearance test is
    // written against MAX_SHAKE_M, so this function is not allowed to
    // exceed it even if a future layer is added carelessly.
    let len = o.length();
    let o = if len > MAX_SHAKE_M { o * (MAX_SHAKE_M / len) } else { o };
    (o, roll * amp)
}

// ---------------------------------------------------------------------
// the rig
// ---------------------------------------------------------------------

struct GaugeInst {
    id: Gauge,
    heavy: bool,
    segs: Vec<Entity>,
    lit: Handle<StandardMaterial>,
}

struct LampInst {
    id: Lamp,
    heavy: bool,
    mat: Handle<StandardMaterial>,
}

struct ScanInst {
    e: Entity,
    /// bottom and top of the travel, in local metres
    y_lo: f32,
    y_hi: f32,
    speed: f32,
    phase: f32,
}

/// Everything `cockpit_sync` needs to find, indexed rather than queried.
///
/// Deliberately the same shape as `VmRig`: a handful of `Entity` lists in
/// a resource. Component marker queries would need three disjoint
/// `Transform`/`Visibility` borrows in one system signature, and the
/// borrow gymnastics are more code than the index.
#[derive(Resource)]
pub struct CockpitRig {
    /// The shake node. Everything hangs off this.
    root: Entity,
    /// Per chassis (0 heavy, 1 medic): the CANOPY group, first person
    /// only, and the INSTRUMENT group, shown in both views. See `pn`/`pi`.
    canopy: [Entity; 2],
    inst: [Entity; 2],
    light: Entity,
    gauges: Vec<GaugeInst>,
    lamps: Vec<LampInst>,
    scans: Vec<ScanInst>,
    /// The lit accent materials, one per chassis, flickered on damage.
    edge: [Handle<StandardMaterial>; 2],
}

/// State carried between frames. Only a damage edge detector and a
/// couple of eased values - nothing the sim could ever read.
#[derive(Default)]
pub struct CockpitMem {
    t: f32,
    prev_hull: f32,
    hit_t: f32,
    fire_t: f32,
}

/// A lamp face: a flat, self-coloured surface the world's light cannot
/// touch. `base_color` is the whole story - see `HEAVY_TINT`.
fn lit(c: [f32; 3], k: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: srgb(c, k),
        unlit: true,
        ..default()
    }
}

/// Build both cockpits under one shake node on the viewmodel camera.
///
/// Called once from `setup` with the viewmodel camera entity. It must be
/// spawned there and not later: `tag_viewmodel_layer` latches after its
/// first sweep, so a late spawn would never be stamped - and this does
/// not rely on that sweep anyway, it stamps every part itself, because a
/// cockpit that silently renders on the WORLD camera would appear as a
/// house-sized box floating at the spawn point.
pub fn spawn_cockpit(
    commands: &mut Commands,
    kit: &ModelKit,
    materials: &mut Assets<StandardMaterial>,
    vm_cam: Entity,
) -> CockpitRig {
    let root = commands
        .spawn((
            Transform::IDENTITY,
            Visibility::Hidden,
            RenderLayers::layer(COCKPIT_LAYER),
        ))
        .id();
    commands.entity(root).set_parent(vm_cam);

    // ---- palettes ----------------------------------------------------
    // Two machines, two material sets, one layout language. The heavy is
    // dark gunmetal plate under amber service lighting; the medic is
    // pale shell under cool white. Nothing is shared but the mesh.
    //
    // The frame tones are deliberately NEUTRAL. The first cut painted
    // the heavy's structure warm khaki and then lit it amber, and the
    // whole cab came out one flat orange: you could not tell the lamp
    // from the wall. A grey frame lets the light BE the colour, which is
    // also the only way the medic's cool white reads as a different
    // machine rather than a different filter.
    //
    // THAT FIX HELD AND THE CAB WENT ORANGE ANYWAY, twice over, and the
    // reason it took a capture to find is that neither cause was in this
    // table. Measured off the level frame of `cockpit`: the structural
    // Frame renders `(0.107, 0.092, 0.077)` - dark, barely warm, exactly
    // what the paragraph above asks for. The orange was
    //   1. `EDGE_D`: the trim strips are BOXES, and at 2 cm deep out by
    //      the screen edge each drew at three times its authored width,
    //      four of them ringing the view; and
    //   2. `EDGE_LEVEL`: that trim burned at full tint, the same value
    //      as a gauge segment reading maximum.
    // A palette can be right and still lose to the geometry that spends
    // it. This is why the two constants exist and why they are asserted.
    let mk = |c: Color, m: f32, r: f32| StandardMaterial {
        base_color: c,
        metallic: m,
        perceptual_roughness: r,
        ..default()
    };
    let (heavy_amber, medic_cyan) = (HEAVY_TINT, MEDIC_TINT);

    // The Edge entries go in through `edge_color`, at `EDGE_LEVEL`
    // rather than at full tint: the trim is a seam, not a readout.
    let trim = |c: [f32; 3]| StandardMaterial {
        base_color: edge_color(c, 1.0),
        unlit: true,
        ..default()
    };
    let heavy_mats = [
        materials.add(mk(Color::srgb(0.155, 0.155, 0.152), 0.30, 0.70)), // Frame
        materials.add(mk(Color::srgb(0.052, 0.052, 0.052), 0.10, 0.85)), // Deep
        materials.add(trim(heavy_amber)),                                // Edge
        materials.add(mk(Color::srgb(0.032, 0.032, 0.030), 0.05, 0.35)), // Fascia
        materials.add(mk(Color::srgb(0.105, 0.105, 0.102), 0.85, 0.38)), // Stud
    ];
    let medic_mats = [
        materials.add(mk(Color::srgb(0.470, 0.492, 0.520), 0.20, 0.50)),
        materials.add(mk(Color::srgb(0.088, 0.098, 0.115), 0.35, 0.60)),
        materials.add(trim(medic_cyan)),
        materials.add(mk(Color::srgb(0.036, 0.042, 0.052), 0.05, 0.35)),
        materials.add(mk(Color::srgb(0.245, 0.265, 0.298), 0.80, 0.36)),
    ];
    let role_i = |r: Role| match r {
        Role::Frame => 0,
        Role::Deep => 1,
        Role::Edge => 2,
        Role::Fascia => 3,
        Role::Stud => 4,
    };
    // the dark backing every gauge segment sits in, so an unlit rung is
    // still a rung rather than a hole. Unlit for the same reason the
    // lamps are: a socket that brightens with the weather is not a
    // socket, and these are inside a machine.
    let seg_off = [
        materials.add(lit(heavy_amber, 0.085)),
        materials.add(lit(medic_cyan, 0.085)),
    ];
    // §20 SCREEN GLOW: the scanline. Thin translucent bars slid up the
    // fascia - the cheapest honest way to say "this is a DISPLAY and not
    // a painted panel" without a shader.
    //
    // Very faint on purpose. The first pass ran them at alpha 0.16 and
    // they came back as three solid tan stripes across the instrument
    // face, hiding the gauges they were supposed to be glazing.
    let scan_mats = [0, 1].map(|ci| {
        let c = if ci == 0 { heavy_amber } else { medic_cyan };
        materials.add(StandardMaterial {
            base_color: Color::srgba(c[0], c[1], c[2], 0.075),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })
    });

    let mut gauges: Vec<GaugeInst> = Vec::new();
    let mut lamps: Vec<LampInst> = Vec::new();
    let mut scans: Vec<ScanInst> = Vec::new();
    let mut canopies = [Entity::PLACEHOLDER; 2];
    let mut insts = [Entity::PLACEHOLDER; 2];

    for (ci, heavy) in [true, false].into_iter().enumerate() {
        // TWO groups per chassis, switched independently: the canopy is
        // physical structure and only exists in first person, the
        // instruments are the interface and survive the V toggle.
        let mut group = || {
            let e = commands
                .spawn((
                    Transform::IDENTITY,
                    Visibility::Hidden,
                    RenderLayers::layer(COCKPIT_LAYER),
                ))
                .id();
            commands.entity(e).set_parent(root);
            e
        };
        let canopy = group();
        let inst = group();
        canopies[ci] = canopy;
        insts[ci] = inst;
        let bin = |is_inst: bool| if is_inst { inst } else { canopy };

        let mats = if heavy { &heavy_mats } else { &medic_mats };
        let panels = if heavy { HEAVY_SHELL } else { MEDIC_SHELL };
        for p in panels {
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(mats[role_i(p.role)].clone()),
                    panel_transform(p),
                    RenderLayers::layer(COCKPIT_LAYER),
                    // §20 NO SHADOWS on any of this. The one directional
                    // light in the game also lights the viewmodel layer,
                    // and a 10 cm deep pillar 40 cm from the lens
                    // presents a nearly edge-on face to it - the first
                    // capture came back with the right-hand pillar
                    // covered in diagonal shadow-map aliasing that read
                    // as a texture nobody had authored. A cockpit
                    // interior has no business casting into the world
                    // either.
                    bevy::pbr::NotShadowCaster,
                    bevy::pbr::NotShadowReceiver,
                ))
                .set_parent(bin(p.inst));
        }

        // ---- gauges --------------------------------------------------
        let ladders = if heavy { HEAVY_LADDERS } else { MEDIC_LADDERS };
        for l in ladders {
            // one lit material per ladder, so a gauge can go to alarm
            // colour without dragging the other three with it.
            //
            // Spawned through `gauge_color` at zero alarm rather than at
            // a bare 1.0: the spawn value is the one a still capture
            // photographs on frame 1 and the sync value is the one the
            // game runs at, and a literal in each is how those two stop
            // agreeing. Same reason `edge_color` exists.
            let lit_mat = materials.add(StandardMaterial {
                base_color: gauge_color(if heavy { heavy_amber } else { medic_cyan }, 0.0, 1.0),
                unlit: true,
                ..default()
            });
            let mut segs = Vec::with_capacity(l.n as usize);
            for k in 0..l.n {
                let (u0, v0) = (l.u + l.du * k as f32, l.v + l.dv * k as f32);
                // the dark rung, always drawn
                commands
                    .spawn((
                        Mesh3d(kit.cube.clone()),
                        MeshMaterial3d(seg_off[ci].clone()),
                        rect_transform(u0, u0 + l.su, v0, v0 + l.sv, Z_RUNG, Z_RUNG_D),
                        RenderLayers::layer(COCKPIT_LAYER),
                        bevy::pbr::NotShadowCaster,
                        bevy::pbr::NotShadowReceiver,
                    ))
                    .set_parent(bin(l.inst));
                // and the lit face over it, shown only up to the level
                let e = commands
                    .spawn((
                        Mesh3d(kit.cube.clone()),
                        MeshMaterial3d(lit_mat.clone()),
                        rect_transform(
                            u0 + l.su * SEG_INSET_U,
                            u0 + l.su * (1.0 - SEG_INSET_U),
                            v0 + l.sv * SEG_INSET_V,
                            v0 + l.sv * (1.0 - SEG_INSET_V),
                            Z_GAUGE,
                            Z_GAUGE_D,
                        ),
                        Visibility::Hidden,
                        RenderLayers::layer(COCKPIT_LAYER),
                        bevy::pbr::NotShadowCaster,
                        bevy::pbr::NotShadowReceiver,
                    ))
                    .set_parent(bin(l.inst))
                    .id();
                segs.push(e);
            }
            gauges.push(GaugeInst { id: l.id, heavy, segs, lit: lit_mat });
        }

        // ---- warning lamps -------------------------------------------
        for ls in if heavy { HEAVY_LAMPS } else { MEDIC_LAMPS } {
            let m = materials.add(lit(LAMP_OFF, 1.0));
            // a bezel, so an unlit lamp is visibly a lamp
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(mats[role_i(Role::Deep)].clone()),
                    rect_transform(
                        ls.u0 - 0.012,
                        ls.u1 + 0.012,
                        ls.v0 - 0.012,
                        ls.v1 + 0.012,
                        Z_RUNG,
                        Z_RUNG_D,
                    ),
                    RenderLayers::layer(COCKPIT_LAYER),
                    bevy::pbr::NotShadowCaster,
                    bevy::pbr::NotShadowReceiver,
                ))
                .set_parent(inst);
            commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(m.clone()),
                    rect_transform(ls.u0, ls.u1, ls.v0, ls.v1, Z_GAUGE, Z_GAUGE_D),
                    RenderLayers::layer(COCKPIT_LAYER),
                    bevy::pbr::NotShadowCaster,
                    bevy::pbr::NotShadowReceiver,
                ))
                .set_parent(inst);
            lamps.push(LampInst { id: ls.id, heavy, mat: m });
        }

        // ---- the scanline over the instrument fascia ------------------
        // The band the bars travel is the fascia panel itself, so a
        // layout change to the panel moves the glow with it rather than
        // leaving it sliding over bare frame.
        //
        // The BIGGEST fascia, not the first one listed. The heavy has
        // two - the instrument panel and the small recess the barrier
        // ladder sits in - and `find` picked whichever happened to be
        // higher up the table, which was the recess: the scanline was
        // running across two centimetres of console lip.
        let band = panels
            .iter()
            .filter(|p| p.role == Role::Fascia && p.inst)
            .max_by(|a, b| {
                let area = |p: &Panel| (p.u1 - p.u0) * (p.v1 - p.v0);
                area(a).total_cmp(&area(b))
            })
            .copied()
            .unwrap_or(pi(-1.0, -0.7, -0.4, 0.2, Z_FASCIA, 0.02, Role::Fascia));
        let (hw, hh) = half_extent(Z_SCAN, DESIGN_ASPECT);
        let (y_lo, y_hi) = (band.v0 * hh, band.v1 * hh);
        for k in 0..3 {
            let e = commands
                .spawn((
                    Mesh3d(kit.cube.clone()),
                    MeshMaterial3d(scan_mats[ci].clone()),
                    Transform {
                        translation: Vec3::new(
                            (band.u0 + band.u1) * 0.5 * hw,
                            y_lo,
                            -Z_SCAN,
                        ),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(
                            ((band.u1 - band.u0) * hw).abs(),
                            (y_hi - y_lo).abs() * 0.055,
                            0.006,
                        ),
                    },
                    RenderLayers::layer(COCKPIT_LAYER),
                    bevy::pbr::NotShadowCaster,
                    bevy::pbr::NotShadowReceiver,
                ))
                .set_parent(inst)
                .id();
            scans.push(ScanInst {
                e,
                y_lo,
                y_hi,
                speed: if heavy { 0.34 } else { 0.55 },
                phase: k as f32 / 3.0,
            });
        }
    }

    // ---- §20 COCKPIT LIGHTING ----------------------------------------
    // A real light source inside the cab, on the viewmodel layer only.
    // It is what makes the hull mount in the lower right pick up the
    // chassis tint instead of sitting under bare daylight, and it is the
    // fastest thing on screen to react to a hit.
    let light = commands
        .spawn((
            PointLight {
                color: Color::srgb(1.0, 0.72, 0.32),
                intensity: 0.0,
                range: 3.0,
                shadows_enabled: false,
                ..default()
            },
            // Above and slightly behind the eye - a service lamp in the
            // canopy roof, not a torch on the console. The first cut had
            // it low and forward, which washed the console lip out to a
            // flat pale slab and put the brightest thing in the frame at
            // the bottom of it.
            Transform::from_xyz(0.0, 0.09, -0.06),
            RenderLayers::layer(COCKPIT_LAYER),
        ))
        .set_parent(root)
        .id();

    CockpitRig {
        root,
        canopy: canopies,
        inst: insts,
        light,
        gauges,
        lamps,
        scans,
        edge: [heavy_mats[2].clone(), medic_mats[2].clone()],
    }
}

/// Drive the whole cockpit: which shell, where it shakes to, what every
/// gauge and lamp reads, and how hard the glass is flickering.
#[allow(clippy::too_many_arguments)]
pub fn cockpit_sync(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    state: Res<State<GameState>>,
    rig: Res<CockpitRig>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut tfs: Query<&mut Transform>,
    mut vis: Query<&mut Visibility>,
    mut lights: Query<&mut PointLight>,
    mut mem: Local<CockpitMem>,
) {
    let dt = time.delta_secs();
    mem.t += dt;
    let t = mem.t;
    let p = &game.sim.fighters[game.sim.player];
    // WHICH CHASSIS, asked through `in_mech`/`in_scout_mech` and never
    // through `armor_set == RobotSuit`. That direct comparison is how the
    // medic pilot fell out of the bottom of half a dozen mech gates
    // earlier in this codebase's life: it is not a test for "in a
    // machine", it is a test for "in THAT machine".
    let heavy_now = p.in_mech() && !p.in_scout_mech();
    let medic_now = p.in_scout_mech();
    // In a chassis, alive, in a match. NOT gated on first person: the
    // INSTRUMENTS survive the V toggle (only the canopy structure does
    // not) - a pilot who pulls the camera back has not stopped flying
    // the machine and must not lose its gauges.
    let on = matches!(state.get(), GameState::Playing)
        && p.alive()
        && (heavy_now || medic_now);
    let first_person = cam_ctl.person_t < 0.5;

    let set_vis = |vis: &mut Query<&mut Visibility>, e: Entity, want: bool| {
        if let Ok(mut v) = vis.get_mut(e) {
            let w = if want { Visibility::Inherited } else { Visibility::Hidden };
            if *v != w {
                *v = w;
            }
        }
    };
    if let Ok(mut v) = vis.get_mut(rig.root) {
        let w = if on { Visibility::Visible } else { Visibility::Hidden };
        if *v != w {
            *v = w;
        }
    }
    for (i, mine) in [heavy_now, medic_now].into_iter().enumerate() {
        set_vis(&mut vis, rig.canopy[i], on && mine && first_person);
        set_vis(&mut vis, rig.inst[i], on && mine);
    }
    if !on {
        // leave the memory clean so re-boarding does not inherit a
        // half-decayed jolt from the last chassis
        mem.hit_t = 0.0;
        mem.fire_t = 0.0;
        mem.prev_hull = p.hull;
        if let Ok(mut l) = lights.get_mut(rig.light) {
            l.intensity = 0.0;
        }
        return;
    }
    let heavy = heavy_now;

    // ---- damage: an edge detector on a value already on screen -------
    // Not a new fact and not a sim hook - the hull number is in the
    // vitals line either way. This only asks whether it went DOWN, which
    // is a question about the last frame, not about the sim.
    if p.hull < mem.prev_hull - 0.01 {
        let bite = ((mem.prev_hull - p.hull) / 60.0).clamp(0.25, 1.0);
        mem.hit_t = mem.hit_t.max(bite);
    }
    mem.prev_hull = p.hull;
    mem.hit_t = (mem.hit_t - dt * 2.2).max(0.0);

    // ---- how hard the machine is working -----------------------------
    let speed = (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt();
    let walk = (speed / 5.0).clamp(0.0, 1.0);
    // the trigger hold timer is the honest "a mount is cycling" signal;
    // ease it so the rattle does not switch on and off with the clock
    let firing = if p.gatling_trigger_t > 0.0 || p.repair_target >= 0 { 1.0 } else { 0.0 };
    mem.fire_t += (firing - mem.fire_t) * (dt * 9.0).min(1.0);
    let burst = if p.stride_t > 0.0 { 1.0 } else { 0.0 };

    let (off, roll) = cockpit_shake(t, heavy, walk, mem.fire_t, burst, mem.hit_t);

    // ---- sealing up / powering down ----------------------------------
    // Boarding is a committed 1.6 s in the sim, and it already has a
    // stage list. The cockpit drops out of the bottom of the frame for
    // the duration rather than snapping into place, so the canopy
    // arriving is the thing that tells you the machine is yours.
    let seal = if p.mech_transition_t > 0.0 {
        let span = if p.mech_exiting { sim::MECH_EXIT_S } else { sim::MECH_ENTER_S };
        (p.mech_transition_t / span.max(0.01)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // ---- the pitch lean ----------------------------------------------
    // A cockpit welded to the camera can never show you more of itself,
    // because your head IS the camera - which is why the first capture
    // pitched 24 degrees down at the console and photographed exactly
    // the same console. In a real machine the cab is bolted to the
    // chassis and only the head turns, so looking down brings the
    // console UP into view.
    //
    // A fraction of that, not all of it: the honest transform would
    // swing the whole frame off screen at full pitch. Clamped hard,
    // because `centre_stays_clear` is written against the sum of this
    // and the shake.
    //
    // Third person has no canopy to reveal, so the lean fades out with
    // the camera pull-back rather than sliding a floating instrument
    // panel around for no reason.
    let lean =
        (cam_ctl.pitch * 0.11).clamp(-MAX_LEAN_M, MAX_LEAN_M) * (1.0 - cam_ctl.person_t);
    if let Ok(mut tf) = tfs.get_mut(rig.root) {
        tf.translation = off + Vec3::new(0.0, lean - seal * 0.34, 0.0);
        tf.rotation = Quat::from_rotation_z(roll);
    }

    // ---- the flicker -------------------------------------------------
    // One number drives every lit surface in the cab. A hit browns the
    // whole frame out for a fraction of a second and it comes back with
    // a stutter, which is the difference between "a light went red" and
    // "something in here just broke".
    let stutter = if mem.hit_t > 0.0 {
        let s = (t * 47.0).sin() * (t * 31.0).cos();
        (1.0 - mem.hit_t * (0.55 + 0.45 * s)).clamp(0.18, 1.0)
    } else {
        // a slow, barely-there breathe so the panels are never dead flat
        1.0 + (t * 1.7).sin() * 0.05
    };
    let base = if heavy { HEAVY_TINT } else { MEDIC_TINT };
    if let Some(m) = mats.get_mut(&rig.edge[if heavy { 0 } else { 1 }]) {
        m.base_color = edge_color(base, stutter);
    }
    if let Ok(mut l) = lights.get_mut(rig.light) {
        l.color = if heavy {
            Color::srgb(1.0, 0.70, 0.30)
        } else {
            Color::srgb(0.55, 0.88, 1.0)
        };
        l.intensity = 1_800.0 * stutter.clamp(0.0, 1.4) * (1.0 - seal);
    }

    // ---- gauges ------------------------------------------------------
    let vit = vitals_of(p);
    for g in &rig.gauges {
        if g.heavy != heavy {
            continue;
        }
        let frac = gauge_frac(&vit, g.id);
        let n = lit_segments(frac, g.segs.len() as u8) as usize;
        for (k, e) in g.segs.iter().enumerate() {
            set_vis(&mut vis, *e, k < n);
        }
        let alarm = gauge_alarm(g.id, frac);
        // an alarming gauge also PULSES. Colour alone is a poor signal
        // on a panel this small and this far into peripheral vision.
        let pulse = if alarm > 0.02 {
            0.65 + 0.35 * (t * (5.0 + alarm * 7.0)).sin()
        } else {
            1.0
        };
        if let Some(m) = mats.get_mut(&g.lit) {
            m.base_color = gauge_color(base, alarm, stutter * pulse);
        }
    }

    // ---- lamps -------------------------------------------------------
    for l in &rig.lamps {
        if l.heavy != heavy {
            continue;
        }
        let (on, col) = match l.id {
            Lamp::Caution => (gauge_frac(&vit, Gauge::Hull) < 0.25, LAMP_CAUTION),
            Lamp::Vent => (p.gatling_vent_t > 0.0, LAMP_VENT),
        };
        if let Some(m) = mats.get_mut(&l.mat) {
            // lamps BLINK. A steady red light is scenery; a blinking one
            // is a thing that started happening. The floor is 0.45 and
            // not zero: a lamp that spends part of its cycle indistin-
            // guishable from an unlit lamp is a lamp a capture can catch
            // dark and a pilot can glance past.
            let b = if on {
                0.45 + 0.55 * ((t * 6.5).sin() * 0.5 + 0.5)
            } else {
                0.0
            };
            m.base_color = srgb(
                [
                    LAMP_OFF[0] + col[0] * b,
                    LAMP_OFF[1] + col[1] * b,
                    LAMP_OFF[2] + col[2] * b,
                ],
                1.0,
            );
        }
    }

    // ---- the scanline ------------------------------------------------
    for s in &rig.scans {
        let span = (s.y_hi - s.y_lo).abs().max(1e-4);
        let f = (t * s.speed + s.phase).rem_euclid(1.0);
        if let Ok(mut tf) = tfs.get_mut(s.e) {
            tf.translation.y = s.y_lo + f * span;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_rect() -> Vec<(&'static str, f32, f32, f32, f32)> {
        let mut out = Vec::new();
        for (name, panels) in [("heavy", HEAVY_SHELL), ("medic", MEDIC_SHELL)] {
            for p in panels {
                out.push((name, p.u0, p.u1, p.v0, p.v1));
            }
        }
        for (name, ls) in [("heavy", HEAVY_LADDERS), ("medic", MEDIC_LADDERS)] {
            for l in ls {
                let (u0, u1, v0, v1) = l.bounds();
                out.push((name, u0, u1, v0, v1));
            }
        }
        for (name, ls) in [("heavy", HEAVY_LAMPS), ("medic", MEDIC_LAMPS)] {
            for l in ls {
                out.push((name, l.u0 - 0.012, l.u1 + 0.012, l.v0 - 0.012, l.v1 + 0.012));
            }
        }
        out
    }

    /// THE requirement: a cockpit may not cover the middle of the frame.
    ///
    /// The crosshair, the precision charge marks and `mech_hud_sync`'s
    /// target bracket all live at the centre, and a strut over any of
    /// them is not a style disagreement - it is a pilot who cannot see
    /// what he is shooting. Checked against the shake bound too, since
    /// the whole rig moves.
    #[test]
    fn centre_stays_clear() {
        let (mu, mv) = (max_shake_u(), max_shake_v());
        let (su, sv) = (CENTRE_SAFE_U + mu, CENTRE_SAFE_V + mv);
        for (name, u0, u1, v0, v1) in every_rect() {
            let hits = u0 < su && u1 > -su && v0 < sv && v1 > -sv;
            assert!(
                !hits,
                "{name} cockpit part ({u0},{u1})x({v0},{v1}) enters the \
                 centre safe box (+-{su:.3}, +-{sv:.3})"
            );
        }
    }

    /// Every declared rectangle has to be a rectangle. A transposed pair
    /// would spawn a box with a negative scale, which renders inside-out
    /// and is very hard to spot in a screenshot.
    #[test]
    fn rects_are_well_formed() {
        for (name, u0, u1, v0, v1) in every_rect() {
            assert!(u1 > u0, "{name}: u range inverted ({u0}..{u1})");
            assert!(v1 > v0, "{name}: v range inverted ({v0}..{v1})");
        }
    }

    /// The two machines must not be the same cockpit. Compare the
    /// LAYOUTS, not the colours: a palette swap was the failure mode
    /// this whole module exists to avoid, and a test that only checked
    /// materials would pass on exactly that.
    #[test]
    fn the_two_cockpits_are_different_machines() {
        assert_ne!(
            HEAVY_SHELL.len(),
            MEDIC_SHELL.len(),
            "different structures should not have the same part count"
        );
        // the heavy has a console lip across the bottom; the medic does
        // not have anything spanning the middle of the lower edge
        let spans_bottom = |ps: &[Panel]| {
            ps.iter()
                .any(|p| p.u0 < -0.3 && p.u1 > 0.3 && p.v0 < -0.95 && p.v1 < -0.6)
        };
        assert!(spans_bottom(HEAVY_SHELL), "the heavy needs its console lip");
        assert!(
            !spans_bottom(MEDIC_SHELL),
            "the medic must stay open across the bottom - a lip there is \
             the heavy's cab with new paint"
        );
        // and the gauges the two machines SHARE are read in different
        // directions. Restricted to the common pair on purpose: the
        // heavy's barrier ladder is a bar sunk into the console lip and
        // the medic has no barrier at all, so a blanket "all columns"
        // rule would only be describing this week's part list.
        let common = [Gauge::Hull, Gauge::Heat];
        for id in common {
            let h = HEAVY_LADDERS.iter().find(|l| l.id == id).expect("heavy gauge");
            let m = MEDIC_LADDERS.iter().find(|l| l.id == id).expect("medic gauge");
            assert!(
                h.dv > 0.0 && h.du == 0.0,
                "the heavy reads {id:?} as a column"
            );
            assert!(
                m.du > 0.0 && m.dv == 0.0,
                "the medic reads {id:?} as a row"
            );
        }
    }

    /// The pillars have to reach the screen edge. A cockpit frame that
    /// stops at 0.9 is a floating bar, not a window.
    #[test]
    fn the_shell_reaches_the_edges() {
        for (name, ps) in [("heavy", HEAVY_SHELL), ("medic", MEDIC_SHELL)] {
            assert!(
                ps.iter().any(|p| p.u0 <= -1.0),
                "{name}: nothing reaches the left edge"
            );
            assert!(
                ps.iter().any(|p| p.u1 >= 1.0),
                "{name}: nothing reaches the right edge"
            );
            assert!(
                ps.iter().any(|p| p.v1 >= 1.0),
                "{name}: nothing reaches the top edge"
            );
            assert!(
                ps.iter().any(|p| p.v0 <= -1.0),
                "{name}: nothing reaches the bottom edge"
            );
        }
    }

    /// THE FLAT-ORANGE DEFECT, as a property.
    ///
    /// A lit trim strip is the loudest material in the cab and it is
    /// supposed to be an accent - a line. What it actually covers is the
    /// convex hull of its far and near faces (`panel_silhouette`), and
    /// out at |u| ~ 0.9 a 2 cm deep box triples its own width. Four of
    /// those ring the view, so the "accent" became the wall.
    ///
    /// Asserted per axis, because a strip is thin in exactly one of them
    /// and that is the axis that can run away: the top rail is 1.02 wide
    /// (spread 1.03, harmless) and 0.022 tall (spread 2.9 at the old
    /// depth, which is the whole bug).
    #[test]
    fn no_lit_trim_renders_far_wider_than_it_was_authored() {
        let mut checked = 0;
        for (name, ps) in [("heavy", HEAVY_SHELL), ("medic", MEDIC_SHELL)] {
            for p in ps.iter().filter(|p| p.role == Role::Edge) {
                checked += 1;
                let (su, sv) = panel_spread(p);
                assert!(
                    su <= EDGE_MAX_SPREAD && sv <= EDGE_MAX_SPREAD,
                    "{name} trim ({},{})x({},{}) at depth {} covers {su:.2}x its \
                     authored width and {sv:.2}x its height (limit \
                     {EDGE_MAX_SPREAD}). A trim strip that draws three times \
                     its own width is not an accent, it is the wall.",
                    p.u0,
                    p.u1,
                    p.v0,
                    p.v1,
                    p.d
                );
            }
        }
        assert!(checked >= 12, "expected both shells' trim, saw {checked}");
    }

    /// `panel_silhouette` has to describe the picture, not the table.
    ///
    /// Pinned against numbers taken off a PNG rather than off this
    /// function: `handback/brief-vii/cockpit/01-cockpit-level.png`,
    /// scanline y=300, showed the heavy's left pillar accent as a solid
    /// amber run from x=77 to x=127 of 1600 - u -0.904 to -0.841. The
    /// old panel was `(-0.862, -0.842)` at z 0.415, d 0.02.
    #[test]
    fn panel_silhouette_matches_what_the_capture_measured() {
        let old = pn(-0.862, -0.842, -0.84, 0.80, 0.415, 0.02, Role::Edge);
        let (u0, u1, _, _) = panel_silhouette(&old);
        assert!(
            (u0 - -0.9056).abs() < 0.002,
            "outer edge should land where the capture found it: {u0}"
        );
        assert!((u1 - -0.842).abs() < 0.002, "inner edge is the far face: {u1}");
        let (su, _) = panel_spread(&old);
        assert!(su > 3.0, "the old trim covered >3x its width, got {su:.2}");
        // and a zero-depth panel covers exactly what it declared
        let flat = pn(-0.862, -0.842, -0.84, 0.80, 0.415, 0.0, Role::Edge);
        let (fu, _) = panel_spread(&flat);
        assert!((fu - 1.0).abs() < 1e-3, "a flat panel cannot spread: {fu}");
    }

    /// The second half of "tell the lamp from the wall".
    ///
    /// The trim and a gauge segment at maximum were the same material
    /// value, and there is an order of magnitude more trim than gauge on
    /// the screen. A seam in the structure must sit clearly below a
    /// readout at full, or the readout is not reading anything.
    #[test]
    fn lit_trim_never_burns_as_hard_as_a_gauge() {
        for tint in [HEAVY_TINT, MEDIC_TINT] {
            let trim = edge_color(tint, 1.0).to_srgba();
            // A CALM gauge, not a full-tint one. This test used to
            // compare the trim against `srgb(tint, 1.0)`, which after
            // `GAUGE_CALM` is a colour nothing on the panel ever burns:
            // an ordering held against a hypothetical value is not an
            // ordering. The claim is about the two things that are
            // actually on the screen together.
            let gauge = gauge_color(tint, 0.0, 1.0).to_srgba();
            let lum = |c: bevy::color::Srgba| 0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue;
            assert!(
                lum(trim) <= 0.7 * lum(gauge),
                "trim {:?} is not clearly below a calm gauge {:?}",
                trim,
                gauge
            );
            // ...but it is still LIT. A trim line dimmed into the frame
            // is a cab that reads dead, which is the other way to fail.
            assert!(
                lum(trim) >= 0.2 * lum(gauge),
                "trim {trim:?} has been dimmed out of existence"
            );
        }
        // the flicker still owns the whole range - a hit must be able to
        // brown the trim out and bring it back
        let dark = edge_color(HEAVY_TINT, 0.18).to_srgba();
        let bright = edge_color(HEAVY_TINT, 1.05).to_srgba();
        assert!(bright.red > dark.red * 3.0, "the hit flicker got flattened");
    }

    /// THE OTHER SIDE OF THE SAME NUMBER.
    ///
    /// `GAUGE_CALM` exists to stop the readouts being the loudest thing
    /// in the cab, and the way to get that wrong is to keep going: a
    /// gauge dimmed toward its own dark backing stops being a lit
    /// instrument and starts being a sticker. So the calm level is
    /// pinned from below as well - clearly over the trim it must
    /// out-read, and far over the unlit rung it sits in.
    #[test]
    fn a_calm_gauge_is_still_lit_instrumentation() {
        let lum = |c: bevy::color::Srgba| 0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue;
        for tint in [HEAVY_TINT, MEDIC_TINT] {
            let calm = gauge_color(tint, 0.0, 1.0).to_srgba();
            let trim = edge_color(tint, 1.0).to_srgba();
            let rung = srgb(tint, 0.085).to_srgba(); // the `seg_off` backing
            assert!(
                lum(calm) >= 1.35 * lum(trim),
                "a calm gauge {calm:?} no longer out-reads the trim {trim:?} - \
                 the readout has been dimmed into the seams"
            );
            assert!(
                lum(calm) >= 5.0 * lum(rung),
                "a calm gauge {calm:?} is barely brighter than the unlit rung \
                 {rung:?} it sits in, so a lit cell no longer reads as lit"
            );
        }
    }

    /// AN ALARM GAINS VALUE, NOT JUST HUE.
    ///
    /// `ALARM_TINT`'s note records the first pass shipping an alarm that
    /// read as "the amber one, but slightly more orange". It fixed the
    /// hue and left the VALUE identical - full-tint either way - so on a
    /// panel this small the only signal was the pulse. `gauge_level`
    /// closes that: calm sits at `GAUGE_CALM`, alarming climbs to 1.0.
    ///
    /// Checked on the composed colour rather than on `gauge_level` alone,
    /// because the mix and the level are applied together and it is the
    /// product that reaches a pixel.
    #[test]
    fn an_alarming_gauge_outburns_a_calm_one() {
        let lum = |c: bevy::color::Srgba| 0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue;
        for tint in [HEAVY_TINT, MEDIC_TINT] {
            let calm = gauge_color(tint, 0.0, 1.0).to_srgba();
            let loud = gauge_color(tint, 1.0, 1.0).to_srgba();
            // the RED channel is the alarm's own channel in both cabs -
            // it is the one `ALARM_TINT` drives to 1.0 - so it must rise
            // whichever tint it started from
            assert!(
                loud.red > calm.red + 0.15,
                "{tint:?}: alarm {loud:?} is not visibly hotter than calm {calm:?}"
            );
            // and it must not merely be a different colour at the same
            // brightness, which is the failure this test exists for
            assert!(
                loud.red / calm.red.max(1e-4) >= 1.2,
                "{tint:?}: the alarm changed hue and kept the value"
            );
            // the calm end really is calm
            assert!(
                lum(calm) < lum(gauge_color(tint, 0.0, 1.0 / GAUGE_CALM).to_srgba()),
                "GAUGE_CALM is not actually holding anything back"
            );
        }
        // monotone the whole way, so a gauge sliding into trouble
        // brightens continuously rather than stepping at a threshold
        let mut prev = -1.0;
        for i in 0..=20 {
            let a = i as f32 / 20.0;
            let v = gauge_level(a);
            assert!(v > prev, "gauge_level is not monotone at alarm {a}");
            prev = v;
        }
        assert!((gauge_level(1.0) - 1.0).abs() < 1e-6, "an alarm must reach full");
        assert!((gauge_level(0.0) - GAUGE_CALM).abs() < 1e-6);
        // clamped, because `gauge_alarm` is the only caller today and a
        // future one might not clamp
        assert!((gauge_level(4.0) - 1.0).abs() < 1e-6);
        assert!((gauge_level(-1.0) - GAUGE_CALM).abs() < 1e-6);
    }

    /// THE BARRIER BAR IS IN THE SIGHT LINE AND THE STACK IS NOT.
    ///
    /// Position is half of how loud a thing is, and this table is the
    /// only place that fact can be checked. A gauge in the CANOPY group
    /// is by construction structure-mounted - the barrier bar is sunk
    /// into the console lip directly below the crosshair - while the
    /// instrument gauges are outboard on their own fascia in peripheral
    /// vision. So the cap applies to the first and not the second.
    ///
    /// Fails on last pass's table: 10 cells of 0.050 x 0.060 inset to
    /// 0.040 x 0.0432 is 0.432% of the screen against a 0.300% cap.
    #[test]
    fn a_canopy_gauge_stays_out_of_the_sight_line() {
        for (name, ls) in [("heavy", HEAVY_LADDERS), ("medic", MEDIC_LADDERS)] {
            for l in ls.iter().filter(|l| !l.inst) {
                assert!(
                    l.ink() <= CANOPY_GAUGE_INK_MAX,
                    "{name}: the {:?} gauge is mounted on the canopy, in the \
                     pilot's sight line, and covers {:.3}% of the screen in lit \
                     tint against a {:.3}% cap",
                    l.id,
                    l.ink() * 100.0,
                    CANOPY_GAUGE_INK_MAX * 100.0
                );
                // ...and it is still a gauge you can read across the
                // room. The cap must be paid for in HEIGHT, not in
                // length: a bar is read by how far along it the light
                // stops, so shortening it would be spending the one
                // dimension that carries the number.
                let (u0, u1, ..) = l.bounds();
                assert!(
                    (u1 - u0).abs() >= 0.55,
                    "{name}: the {:?} bar is only {:.2} of the screen long - a \
                     segmented bar shortened is a bar you cannot read",
                    l.id,
                    (u1 - u0).abs()
                );
                assert!(
                    l.n >= 8,
                    "{name}: {:?} is down to {} cells; a bar with too few \
                     stops cannot show you where the level is",
                    l.id,
                    l.n
                );
            }
        }
        // the exemption is real and deliberate: the outboard stack is
        // allowed to be bigger than this, and if it ever came in under
        // the cap the asymmetry above would be testing nothing
        let stack: f32 = HEAVY_LADDERS.iter().filter(|l| l.inst).map(|l| l.ink()).sum();
        assert!(
            stack > CANOPY_GAUGE_INK_MAX,
            "the instrument stack is now under the canopy cap, so this test \
             no longer distinguishes the two mountings"
        );
    }

    /// `ink` has to agree with the rectangle the spawner actually draws,
    /// or the cap above is bounding a number nobody renders.
    ///
    /// Recomputed here from the ladder's own fields the long way round -
    /// per segment, from the same `SEG_INSET_*` the spawn uses - so the
    /// two disagree if anybody changes the inset in one place.
    #[test]
    fn ladder_ink_measures_what_the_spawner_draws() {
        for l in HEAVY_LADDERS.iter().chain(MEDIC_LADDERS) {
            let mut area = 0.0;
            for k in 0..l.n {
                let (u0, v0) = (l.u + l.du * k as f32, l.v + l.dv * k as f32);
                let (a, b) = (u0 + l.su * SEG_INSET_U, u0 + l.su * (1.0 - SEG_INSET_U));
                let (c, d) = (v0 + l.sv * SEG_INSET_V, v0 + l.sv * (1.0 - SEG_INSET_V));
                area += (b - a).abs() * (d - c).abs();
            }
            assert!(
                (l.ink() - area / 4.0).abs() < 1e-6,
                "{:?}: ink() says {:.5}, the drawn faces come to {:.5}",
                l.id,
                l.ink(),
                area / 4.0
            );
        }
        // and the inset is a real inset - a lit face that filled its
        // rung would leave no socket round it
        assert!(SEG_INSET_U > 0.0 && SEG_INSET_V > 0.0);
        assert!(SEG_INSET_U < 0.5 && SEG_INSET_V < 0.5);
    }

    /// A box declared at its FAR face and extruded toward the camera has
    /// to end up NEARER than that face - if the sign is ever flipped the
    /// clearance test above stops bounding anything.
    #[test]
    fn panels_extrude_toward_the_camera() {
        for p in HEAVY_SHELL.iter().chain(MEDIC_SHELL) {
            let tf = panel_transform(p);
            let near = tf.translation.z + tf.scale.z * 0.5;
            let far = tf.translation.z - tf.scale.z * 0.5;
            assert!(
                (far + p.z).abs() < 1e-5,
                "far face should sit at the declared depth: {far} vs {}",
                -p.z
            );
            assert!(near > far, "the box must extrude toward the eye");
            assert!(near < 0.0, "the whole box must stay in front of the eye");
        }
    }

    /// The defect a capture caught and nothing else could: an
    /// instrument bezel deep enough to swallow the gauges it surrounds.
    ///
    /// The layers of the panel stack go bezel, fascia, rung, lit face,
    /// scanline, front to back. Each has a declared far face and a
    /// depth, and each one's NEAR face must still be BEHIND the next
    /// layer's far face - otherwise the outer box's front surface
    /// crosses in front of what it contains, and the inner layers are
    /// simply gone.
    #[test]
    fn panel_stack_layers_do_not_bury_each_other() {
        let layers = [
            ("bezel", Z_BEZEL, Z_BEZEL_D),
            ("fascia", Z_FASCIA, Z_FASCIA_D),
            ("rung", Z_RUNG, Z_RUNG_D),
            ("lit face", Z_GAUGE, Z_GAUGE_D),
        ];
        for w in layers.windows(2) {
            let ((an, az, ad), (bn, bz, _)) = (w[0], w[1]);
            let a_near = az - ad;
            assert!(
                a_near > bz,
                "{an}'s near face ({a_near}) is in front of {bn}'s far \
                 face ({bz}) - {bn} would be invisible"
            );
        }
        let (_, lz, ld) = layers[layers.len() - 1];
        assert!(lz - ld > Z_SCAN, "the scanline must stay in front of the gauges");
    }

    /// The OTHER thing the first capture caught. A pillar is a 10 cm
    /// deep box hard against the screen edge, so its inner side face
    /// sweeps forward from 0.44 m to 0.34 m as it recedes across the
    /// screen. Anything drawn in that screen band at a greater depth is
    /// behind a wall - which is what happened to two of the heavy's
    /// three ladders, sitting perfectly correctly inside a pillar.
    #[test]
    fn no_instrument_hides_inside_a_pillar() {
        for (name, ps, ls) in [
            ("heavy", HEAVY_SHELL, HEAVY_LADDERS),
            ("medic", MEDIC_SHELL, MEDIC_LADDERS),
        ] {
            // a pillar: structural, reaches an edge, and is tall
            let tall = |p: &&Panel| p.role == Role::Frame && (p.v1 - p.v0) > 1.0;
            let left = ps
                .iter()
                .filter(tall)
                .filter(|p| p.u0 <= -1.0)
                .map(|p| p.u1)
                .fold(f32::NEG_INFINITY, f32::max);
            let right = ps
                .iter()
                .filter(tall)
                .filter(|p| p.u1 >= 1.0)
                .map(|p| p.u0)
                .fold(f32::INFINITY, f32::min);
            assert!(left.is_finite() && right.is_finite(), "{name}: no pillars found");
            for l in ls {
                let (u0, u1, ..) = l.bounds();
                assert!(
                    u0 >= left,
                    "{name}: the {:?} ladder starts at {u0}, outboard of the \
                     left pillar's inner face at {left}",
                    l.id
                );
                assert!(
                    u1 <= right,
                    "{name}: the {:?} ladder ends at {u1}, outboard of the \
                     right pillar's inner face at {right}",
                    l.id
                );
            }
        }
    }

    /// THE INTERFACE SURVIVES THE V TOGGLE.
    ///
    /// The owner's line is that the cockpit should feel like being
    /// inside the machine in BOTH camera modes. The physical canopy
    /// cannot be in third person - you are outside the machine looking
    /// at it - so what has to carry across is the INSTRUMENTATION. This
    /// pins that split in the tables, where it is decided, rather than
    /// trusting a visibility line in a system nothing can call.
    #[test]
    fn every_chassis_keeps_a_readout_in_third_person() {
        for (name, ps, ls, lamps) in [
            ("heavy", HEAVY_SHELL, HEAVY_LADDERS, HEAVY_LAMPS),
            ("medic", MEDIC_SHELL, MEDIC_LADDERS, MEDIC_LAMPS),
        ] {
            // a live gauge
            assert!(
                ls.iter().filter(|l| l.inst).count() >= 2,
                "{name}: at least two gauges must survive the V toggle"
            );
            // something to print them on
            assert!(
                ps.iter().any(|p| p.inst && p.role == Role::Fascia),
                "{name}: the instrument group has no panel to sit on"
            );
            // and both warning lamps, which are the whole point of a
            // lamp - you need it most when you are not looking at it
            assert_eq!(
                lamps.len(),
                2,
                "{name}: both warning lamps belong to the instruments"
            );
            // the canopy must still be a canopy - structure that only
            // makes sense from inside
            assert!(
                ps.iter().any(|p| !p.inst && (p.u0 <= -1.0 || p.u1 >= 1.0)),
                "{name}: nothing structural left in the canopy group"
            );
            // and every instrument must sit clear of the screen edges,
            // because in third person there is no frame around it and a
            // panel bleeding off the edge reads as a rendering fault
            for p in ps.iter().filter(|p| p.inst) {
                assert!(
                    p.u0 > -1.0 && p.u1 < 1.0,
                    "{name}: an instrument part runs to the screen edge \
                     ({}..{}) with no canopy to justify it",
                    p.u0,
                    p.u1
                );
            }
        }
    }

    /// A gauge has to be printed ON something. A ladder floating off its
    /// fascia reads as debris stuck to the glass.
    #[test]
    fn every_ladder_sits_on_a_fascia() {
        for (name, ps, ls) in [
            ("heavy", HEAVY_SHELL, HEAVY_LADDERS),
            ("medic", MEDIC_SHELL, MEDIC_LADDERS),
        ] {
            for l in ls {
                let (u0, u1, v0, v1) = l.bounds();
                let on = ps.iter().any(|p| {
                    p.role == Role::Fascia
                        && p.u0 <= u0 + 1e-4
                        && p.u1 >= u1 - 1e-4
                        && p.v0 <= v0 + 1e-4
                        && p.v1 >= v1 - 1e-4
                });
                assert!(
                    on,
                    "{name}: the {:?} ladder ({u0}..{u1} x {v0}..{v1}) is not \
                     inside any instrument panel",
                    l.id
                );
            }
        }
    }

    #[test]
    fn a_ladder_lights_from_empty_to_full() {
        assert_eq!(lit_segments(0.0, 10), 0, "empty is empty");
        assert_eq!(lit_segments(1.0, 10), 10, "full is full");
        // the one that matters: a pool with anything left in it must
        // never read as empty
        assert_eq!(lit_segments(0.001, 10), 1, "a sliver still lights a rung");
        assert_eq!(lit_segments(0.5, 10), 5);
        let mut prev = 0;
        for i in 0..=100 {
            let n = lit_segments(i as f32 / 100.0, 10);
            assert!(n >= prev, "a ladder must never go down as the value goes up");
            prev = n;
        }
    }

    /// The heat field carries two scales in one `f32`. Both chassis have
    /// to land on the same 0..1 here or one of them shows a bar that is
    /// pinned at either end for the whole match.
    #[test]
    fn heat_reads_the_same_on_both_chassis() {
        assert!(
            (heat_frac(50.0, false) - 0.5).abs() < 1e-4,
            "the heavy's 0..100 heat must normalise, got {}",
            heat_frac(50.0, false)
        );
        assert!(
            (heat_frac(0.5, true) - 0.5).abs() < 1e-4,
            "the medic's 0..1 heat must normalise, got {}",
            heat_frac(0.5, true)
        );
        // and the trap this exists to catch: reading the heavy's number
        // on the medic's scale pins the bar at full for the whole match
        assert!(
            (heat_frac(50.0, true) - 1.0).abs() < 1e-4,
            "cross-wiring the scales should be visibly, not subtly, wrong"
        );
    }

    /// A ladder must not report a number the pilot's own vitals line
    /// disagrees with. Both read the SAME fields; this pins the mapping.
    #[test]
    fn gauges_read_the_fields_they_claim_to() {
        let v = Vitals {
            hull: 150.0,
            hull_max: 600.0,
            power: 0.4,
            heat: 0.75,
            barrier: 0.9,
            charge: 0.2,
        };
        assert!((gauge_frac(&v, Gauge::Hull) - 0.25).abs() < 1e-5);
        assert!((gauge_frac(&v, Gauge::Power) - 0.4).abs() < 1e-5);
        assert!((gauge_frac(&v, Gauge::Heat) - 0.75).abs() < 1e-5);
        assert!((gauge_frac(&v, Gauge::Barrier) - 0.9).abs() < 1e-5);
        assert!((gauge_frac(&v, Gauge::Charge) - 0.2).abs() < 1e-5);
        // a hull pool the sim has already emptied must not wrap or spike
        let dead = Vitals { hull: -12.0, ..v };
        assert_eq!(gauge_frac(&dead, Gauge::Hull), 0.0);
    }

    /// Falling pools and rising ones must not share an alarm ramp.
    #[test]
    fn alarms_point_the_right_way() {
        assert!(gauge_alarm(Gauge::Hull, 1.0) < 0.01, "a full hull is calm");
        assert!(gauge_alarm(Gauge::Hull, 0.1) > 0.5, "a spent hull is not");
        assert!(gauge_alarm(Gauge::Heat, 0.0) < 0.01, "a cold mount is calm");
        assert!(gauge_alarm(Gauge::Heat, 1.0) > 0.9, "a maxed mount is not");
        assert!(
            gauge_alarm(Gauge::Barrier, 1.0) < gauge_alarm(Gauge::Barrier, 0.0),
            "the barrier reads like the hull, not like the heat"
        );
    }

    /// The shake is the one part of this that can reach the crosshair,
    /// so its bound is not advisory.
    #[test]
    fn the_shake_stays_inside_its_bound() {
        let mut worst = 0.0_f32;
        let mut worst_roll = 0.0_f32;
        for i in 0..4000 {
            let t = i as f32 * 0.0037;
            for heavy in [true, false] {
                // every layer at maximum at once - a state no player can
                // reach, which is the point of checking it
                let (o, r) = cockpit_shake(t, heavy, 1.0, 1.0, 1.0, 1.0);
                worst = worst.max(o.length());
                worst_roll = worst_roll.max(r.abs());
            }
        }
        assert!(
            worst <= MAX_SHAKE_M + 1e-5,
            "shake reached {worst} m against a declared bound of {MAX_SHAKE_M}"
        );
        assert!(worst > MAX_SHAKE_M * 0.2, "a bound nothing approaches is not a bound");
        assert!(worst_roll < 0.05, "roll should be a tremor, not a barrel roll");
    }

    /// A parked, undamaged machine still has to be ALIVE - a dead-still
    /// cockpit is a painted backdrop.
    #[test]
    fn the_machine_is_never_completely_still() {
        let mut moved = 0.0_f32;
        for i in 0..500 {
            let t = i as f32 * 0.017;
            let (o, _) = cockpit_shake(t, true, 0.0, 0.0, 0.0, 0.0);
            moved = moved.max(o.length());
        }
        assert!(moved > 1e-4, "the idle hum went missing: peak {moved}");
    }

    /// The two machines must not shake alike either.
    #[test]
    fn the_two_cockpits_do_not_shake_alike() {
        let mut differed = false;
        for i in 0..500 {
            let t = i as f32 * 0.011;
            let (a, _) = cockpit_shake(t, true, 1.0, 0.0, 0.0, 0.0);
            let (b, _) = cockpit_shake(t, false, 1.0, 0.0, 0.0, 0.0);
            if (a - b).length() > 1e-4 {
                differed = true;
            }
        }
        assert!(differed, "the light chassis judders exactly like the heavy one");
    }

    /// The shell has to sit NEARER than every hull mount, or the machine
    /// draws its own gun in front of its own canopy.
    #[test]
    fn the_canopy_sits_in_front_of_the_mounts() {
        // the nearest mount carry in `setup` is the rocket pod at 0.46
        const NEAREST_MOUNT: f32 = 0.46;
        for p in HEAVY_SHELL.iter().chain(MEDIC_SHELL) {
            let near = p.z - p.d;
            assert!(
                near < NEAREST_MOUNT,
                "a cockpit part at {near} m would sit behind the hull mounts"
            );
        }
    }
}
